use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_inspector_egui::egui;
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use crate::{
    anim::{
        auto_collider::{AutoCollider, AutoColliderMesh, AutoColliderRoot},
        rig::{KiBone, KiJointType, KiRevoluteJoint, KiRoot, KiSphericalJoint},
    },
    camera::{MainCamera, ScreenPosition},
};

use super::{
    basic_materials::BasicMaterials,
    selection::Selectable,
    side_panel::{ui_mode_toggle, SidePanel, UiMode},
};

pub struct AddCubePlugin;

impl Plugin for AddCubePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AddCubeUiState>()
            .add_systems((update_add_cube, shoot_balls));
    }
}

#[derive(Resource)]
pub struct AddCubeUiState {
    pub joint_type: KiJointType,
}

impl Default for AddCubeUiState {
    fn default() -> Self {
        Self {
            joint_type: KiJointType::Revolute,
        }
    }
}

#[derive(PartialEq, Eq)]
enum AddCubeModeState {
    None,
    PickP0,
    PickP1,
    PickP2,
    PickHeight,
}

impl Default for AddCubeModeState {
    fn default() -> Self {
        AddCubeModeState::None
    }
}

#[derive(Component)]
struct AddCubeSelector;

#[derive(Default)]
struct AddCubeLocal {
    state: AddCubeModeState,
    attach: Option<Entity>,
    cube: Option<Entity>,
    p0: Option<Vec3>,
    p0_n: Option<Vec3>,
    p1: Option<Vec3>,
    p2: Option<Vec3>,
    height: Option<f32>,
}

const CUBE_INIT_LEN: f32 = 0.1;

fn update_add_cube(
    mut state: Local<AddCubeLocal>,
    mut meshes: ResMut<Assets<Mesh>>,
    panel: Res<SidePanel>,
    cube_ui: Res<AddCubeUiState>,
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    materials: Res<BasicMaterials>,
    q_camera: Query<&MainCamera>,
    mut q_trans: Query<&mut Transform, With<AddCubeSelector>>,
    q_gtrans: Query<&GlobalTransform>,
    q_coll: Query<(&AutoCollider, &Parent)>,
    q_parent: Query<&Parent>,
    mut cmd: Commands,
) {
    if panel.mode != UiMode::AddCube {
        if state.state != AddCubeModeState::None {
            clear_ui_state(&mut state, &mut cmd);
        }
        return;
    }

    if state.state == AddCubeModeState::None {
        state.state = AddCubeModeState::PickP0;
    }

    if let Some(cube) = state.cube {
        if let (Ok(mut cube_tr), Some(p0), Some(p0_n), Some(p1)) =
            (q_trans.get_mut(cube), state.p0, state.p0_n, state.p1)
        {
            let height = state.height.unwrap_or(CUBE_INIT_LEN);
            let dir_x = (p1 - p0).normalize();
            cube_tr.rotation =
                Quat::from_mat3(&Mat3::from_cols(dir_x, p0_n, dir_x.cross(p0_n).normalize()));
            let p2 = state.p2.unwrap_or(p1 + cube_tr.back() * CUBE_INIT_LEN);
            cube_tr.translation = (p0 + p2 + height * cube_tr.up()) / 2.;
            cube_tr.scale = Vec3::new((p1 - p0).length(), height, (p2 - p1).length());
        }
    }

    if panel.mouse_over {
        return;
    }

    let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else { return };
    let ray_p = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());

    if state.state == AddCubeModeState::PickP0 {
        if mouse.just_pressed(MouseButton::Left) {
            let material = materials.ui_transparent.clone();
            let Some((attach, hit)) = rapier.cast_ray_and_get_normal(
                ray.origin,
                ray.direction,
                1000.,
                false,
                QueryFilter::new().exclude_sensors(),
            ) else { return };

            let p0_n = hit.normal.normalize();
            let p0 = hit.point;

            state.attach = Some(attach);
            state.p0 = Some(p0);
            state.p0_n = Some(p0_n);
            state.state = AddCubeModeState::PickP1;

            let dir_x = p0_n.any_orthonormal_vector();
            state.cube = Some(
                cmd.spawn((
                    PbrBundle {
                        transform: Transform::from_translation(p0 + (CUBE_INIT_LEN / 2.) * p0_n)
                            .with_rotation(Quat::from_mat3(&Mat3::from_cols(
                                dir_x,
                                p0_n,
                                dir_x.cross(p0_n).normalize(),
                            )))
                            .with_scale(Vec3::new(0., CUBE_INIT_LEN, 0.)),
                        mesh: meshes.add(Mesh::from(shape::Box::new(1., 1., 1.))),
                        material: material.clone(),
                        ..default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                    AddCubeSelector,
                ))
                .id(),
            );
        }
    } else if state.state == AddCubeModeState::PickP1 {
        let p0 = state.p0.unwrap();
        let p0_n = state.p0_n.unwrap();
        let Some(toi) = ray_toi_with_halfspace(&p0.into(), &p0_n.into(), &ray_p) else { return };
        let mut p1 = ray.origin + toi * ray.direction;
        if let Ok(attach_gtr) = q_gtrans.get(state.attach.unwrap()) {
            let dp = p1 - p0;
            let dp_len = dp.length();
            for snap in [attach_gtr.up(), attach_gtr.right(), attach_gtr.back()] {
                let len = dp.dot(snap);
                if dp_len - len.abs() < 0.1 {
                    p1 = p0 - len * p0_n.cross(p0_n.cross(snap)).normalize();
                    break;
                }
            }
        }
        state.p1 = Some(p1);
        if mouse.just_pressed(MouseButton::Left) {
            state.state = AddCubeModeState::PickP2;
        }
    } else if state.state == AddCubeModeState::PickP2 {
        let p0 = state.p0.unwrap();
        let p0_n = state.p0_n.unwrap();
        let Some(toi) = ray_toi_with_halfspace(&p0.into(), &p0_n.into(), &ray_p) else { return };
        let cube_tr = q_trans.get(state.cube.unwrap()).unwrap();
        let p1 = state.p1.unwrap();
        let p2 = ray.origin + toi * ray.direction;
        let len = (p2 - p1).dot(cube_tr.back());
        state.p2 = Some(p1 + len * cube_tr.back());
        if mouse.just_pressed(MouseButton::Left) {
            state.state = AddCubeModeState::PickHeight;
        }
    } else if state.state == AddCubeModeState::PickHeight {
        let cube_tr = q_trans.get(state.cube.unwrap()).unwrap();
        if mouse.just_pressed(MouseButton::Left) {
            let material = materials.salmon.clone();

            let (coll_ent, mesh_ent, is_root) = {
                if let Ok((coll, colp_p)) = q_coll.get(state.attach.unwrap()) {
                    let parent_ent = q_parent.get(coll.mesh).unwrap().get();
                    let parent_inv = q_gtrans.get(parent_ent).unwrap().affine().inverse();
                    let (p0, p1, p2) = (state.p0.unwrap(), state.p1.unwrap(), state.p2.unwrap());
                    let sz = cube_tr.scale;

                    let new_bone_tr =
                        Transform::from_translation(parent_inv.transform_point3((p0 + p2) / 2.))
                            .with_rotation(
                                parent_inv.to_scale_rotation_translation().1 * cube_tr.rotation,
                            );

                    let bone_ent = cmd
                        .spawn((
                            SpatialBundle::from(new_bone_tr),
                            ScreenPosition::default(),
                            KiBone::new(sz.y),
                        ))
                        .id();
                    cmd.entity(parent_ent).add_child(bone_ent);

                    let mesh_ent = cmd
                        .spawn((PbrBundle {
                            transform: Transform::from_xyz(0., sz.y / 2., 0.),
                            mesh: meshes.add(Mesh::from(shape::Box::new(sz.x, sz.y, sz.z))),
                            material,
                            ..default()
                        },))
                        .id();
                    cmd.entity(bone_ent).add_child(mesh_ent);

                    if cube_ui.joint_type == KiJointType::Revolute {
                        let hinge = new_bone_tr
                            .compute_affine()
                            .inverse()
                            .transform_vector3(parent_inv.transform_vector3(p1 - p0));
                        cmd.entity(bone_ent).insert(KiRevoluteJoint {
                            length: hinge.length(),
                            start_dir: new_bone_tr.up(),
                            show_mesh: true,
                        });
                    } else if cube_ui.joint_type == KiJointType::Spherical {
                        cmd.entity(bone_ent).insert(KiSphericalJoint {
                            show_mesh: true,
                            start_rot: new_bone_tr.rotation,
                        });
                    }

                    let coll_ent = cmd
                        .spawn((
                            SpatialBundle::from(Transform::from_translation(Vec3::ZERO)),
                            Collider::cuboid(sz.x / 2., sz.y / 2., sz.z / 2.),
                            ColliderDisabled,
                            Selectable::new(bone_ent, Some(mesh_ent)),
                        ))
                        .id();
                    cmd.entity(colp_p.get()).add_child(coll_ent);

                    (coll_ent, mesh_ent, false)
                } else {
                    let new_obj = cmd
                        .spawn((
                            SpatialBundle::from(
                                Transform::from_translation(cube_tr.translation)
                                    .with_rotation(cube_tr.rotation),
                            ),
                            RigidBody::Dynamic,
                            ScreenPosition::default(),
                            KiRoot,
                            AutoColliderRoot,
                        ))
                        .id();
                    let sz = cube_tr.scale;

                    let mesh_ent = cmd
                        .spawn((PbrBundle {
                            transform: Transform::IDENTITY,
                            mesh: meshes.add(Mesh::from(shape::Box::new(sz.x, sz.y, sz.z))),
                            material,
                            ..default()
                        },))
                        .id();
                    cmd.entity(new_obj).add_child(mesh_ent);

                    let coll_ent = cmd
                        .spawn((
                            SpatialBundle::from(Transform::from_translation(Vec3::ZERO)),
                            Collider::cuboid(sz.x / 2., sz.y / 2., sz.z / 2.),
                            Selectable::new(new_obj, Some(mesh_ent)),
                        ))
                        .id();
                    cmd.entity(new_obj).add_child(coll_ent);

                    (coll_ent, mesh_ent, true)
                }
            };

            cmd.entity(coll_ent).insert(AutoCollider {
                mesh: mesh_ent,
                update_transform: !is_root,
            });
            cmd.entity(mesh_ent)
                .insert(AutoColliderMesh { collider: coll_ent });

            clear_ui_state(&mut state, &mut cmd);
        } else {
            let p2 = state.p2.unwrap();
            let (Some(toi0), Some(toi1)) = (
                ray_toi_with_halfspace(&p2.into(), &cube_tr.right().into(), &ray_p),
                ray_toi_with_halfspace(&p2.into(), &cube_tr.back().into(), &ray_p),
            ) else { return };
            let i0 = ray.origin + toi0 * ray.direction;
            let i1 = ray.origin + toi1 * ray.direction;
            let p2_y = cube_tr.up().dot(p2);
            let y0 = cube_tr.up().dot(i0);
            let y1 = cube_tr.up().dot(i1);
            state.height = Some(((y0 + y1) / 2. - p2_y).max(CUBE_INIT_LEN));
        }
    }
}

fn clear_ui_state(state: &mut AddCubeLocal, cmd: &mut Commands) {
    state.state = AddCubeModeState::None;
    state.attach = None;
    if let Some(cube) = state.cube {
        cmd.entity(cube).despawn_recursive();
    }
    state.cube = None;
    state.p0 = None;
    state.p0_n = None;
    state.p1 = None;
    state.p2 = None;
    state.height = None;
}

pub fn add_cube_ui(ui: &mut egui::Ui, panel: &mut SidePanel, mut cube: ResMut<AddCubeUiState>) {
    ui_mode_toggle(ui, panel, UiMode::AddCube, "Add cube");

    if panel.mode == UiMode::AddCube {
        ui.indent(10, |ui| {
            ui.horizontal(|ui| {
                ui.label("Joint type:");
                ui.selectable_value(&mut cube.joint_type, KiJointType::Revolute, "Revolute");
                ui.selectable_value(&mut cube.joint_type, KiJointType::Spherical, "Spherical");
            });
        });
    }

    ui_mode_toggle(ui, panel, UiMode::ShootBalls, "Shoot balls");
}

#[derive(Component)]
pub struct ShootyBall;

fn shoot_balls(
    panel: Res<SidePanel>,
    materials: Res<BasicMaterials>,
    mouse: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_camera: Query<&MainCamera>,
    q_balls: Query<(Entity, &GlobalTransform), With<ShootyBall>>,
    mut cmd: Commands,
) {
    for (ball, ball_tr) in &q_balls {
        if ball_tr.translation().y < -50. {
            cmd.entity(ball).despawn_recursive();
        }
    }

    if panel.mode != UiMode::ShootBalls
        || panel.mouse_over
        || !mouse.just_pressed(MouseButton::Left)
    {
        return;
    };

    let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else { return };
    let ball = cmd
        .spawn((
            PbrBundle {
                transform: Transform::from_translation(ray.origin),
                mesh: meshes.add(
                    Mesh::try_from(shape::Icosphere {
                        radius: 1.,
                        subdivisions: 20,
                    })
                    .unwrap(),
                ),
                material: materials.gold.clone(),
                ..default()
            },
            ShootyBall,
            ScreenPosition::default(),
            RigidBody::Dynamic,
            Damping {
                linear_damping: 0.,
                angular_damping: 0.,
            },
            Velocity {
                linvel: 30. * ray.direction,
                angvel: Vec3::ZERO,
            },
            Collider::ball(1.0),
            ColliderMassProperties::Density(0.8),
            Friction {
                coefficient: 0.8,
                combine_rule: CoefficientCombineRule::Average,
            },
            Restitution {
                coefficient: 0.3,
                combine_rule: CoefficientCombineRule::Average,
            },
        ))
        .id();
    cmd.entity(ball).insert(Selectable::new(ball, Some(ball)));
}
