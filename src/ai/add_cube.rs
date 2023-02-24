use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use crate::{
    camera::{MainCamera, ScreenPosition},
    ui::{
        basic_materials::BasicMaterialsRes,
        selection::Selectable,
        side_panel::{SidePanelState, UiMode},
    },
};

use super::{joints::KinematicHinge, terrain::TerrainRes};

pub struct AddCubePlugin;

impl Plugin for AddCubePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(add_cube_ui).add_system(shoot_balls);
    }
}

#[derive(PartialEq, Eq)]
enum AddCubeUiState {
    None,
    PickP0,
    PickP1,
    PickP2,
    PickHeight,
}

impl Default for AddCubeUiState {
    fn default() -> Self {
        AddCubeUiState::None
    }
}

#[derive(Default)]
struct AddCubeLocal {
    state: AddCubeUiState,
    attach: Option<Entity>,
    cube: Option<Entity>,
    p0: Option<Vec3>,
    p0_n: Option<Vec3>,
    p1: Option<Vec3>,
    p2: Option<Vec3>,
    height: Option<f32>,
}

const CUBE_INIT_LEN: f32 = 0.1;

fn add_cube_ui(
    mut state: Local<AddCubeLocal>,
    mut meshes: ResMut<Assets<Mesh>>,
    ui: Res<SidePanelState>,
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    terrain: Res<TerrainRes>,
    materials: Res<BasicMaterialsRes>,
    q_camera: Query<&MainCamera>,
    mut q_trans: Query<&mut Transform>,
    q_gtrans: Query<&GlobalTransform>,
    mut cmd: Commands,
) {
    if ui.mode == UiMode::AddCube {
        if state.state == AddCubeUiState::None {
            state.state = AddCubeUiState::PickP0;
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

        if ui.mouse_over {
            return;
        }

        if let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) {
            let ray_p = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());

            if state.state == AddCubeUiState::PickP0 {
                if mouse.just_pressed(MouseButton::Left) {
                    let material = materials.ui_transparent.clone();
                    if let Some((attach, hit)) = rapier.cast_ray_and_get_normal(
                        ray.origin,
                        ray.direction,
                        1000.,
                        false,
                        QueryFilter::new().exclude_sensors(),
                    ) {
                        let p0_n = hit.normal.normalize();
                        let p0 = hit.point;

                        state.attach = Some(attach);
                        state.p0 = Some(p0);
                        state.p0_n = Some(p0_n);
                        state.state = AddCubeUiState::PickP1;

                        let dir_x = p0_n.any_orthonormal_vector();
                        state.cube = Some(
                            cmd.spawn((
                                PbrBundle {
                                    transform: Transform::from_translation(
                                        p0 + (CUBE_INIT_LEN / 2.) * p0_n,
                                    )
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
                            ))
                            .id(),
                        );
                    }
                }
            } else if state.state == AddCubeUiState::PickP1 {
                let p0 = state.p0.unwrap();
                let p0_n = state.p0_n.unwrap();
                if let Some(toi) = ray_toi_with_halfspace(&p0.into(), &p0_n.into(), &ray_p) {
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
                        state.state = AddCubeUiState::PickP2;
                    }
                }
            } else if state.state == AddCubeUiState::PickP2 {
                let p0 = state.p0.unwrap();
                let p0_n = state.p0_n.unwrap();
                if let Some(toi) = ray_toi_with_halfspace(&p0.into(), &p0_n.into(), &ray_p) {
                    let cube_tr = q_trans.get(state.cube.unwrap()).unwrap();
                    let p1 = state.p1.unwrap();
                    let p2 = ray.origin + toi * ray.direction;
                    let len = (p2 - p1).dot(cube_tr.back());
                    state.p2 = Some(p1 + len * cube_tr.back());
                    if mouse.just_pressed(MouseButton::Left) {
                        state.state = AddCubeUiState::PickHeight;
                    }
                }
            } else if state.state == AddCubeUiState::PickHeight {
                let cube_tr = q_trans.get(state.cube.unwrap()).unwrap();
                if mouse.just_pressed(MouseButton::Left) {
                    let material = materials.salmon.clone();
                    let attach = state.attach.unwrap();
                    let attach_gtr = q_gtrans.get(attach).unwrap();
                    let attach_inv = attach_gtr.affine().inverse();
                    let new_cube_tr = Transform::from_translation(
                        attach_inv.transform_point3(cube_tr.translation),
                    )
                    .with_rotation(attach_inv.to_scale_rotation_translation().1 * cube_tr.rotation);
                    let s = cube_tr.scale;

                    cmd.entity(attach).with_children(|children| {
                        let mut new_cube = children.spawn((
                            PbrBundle {
                                transform: new_cube_tr,
                                mesh: meshes.add(Mesh::from(shape::Box::new(s.x, s.y, s.z))),
                                material,
                                ..default()
                            },
                            Selectable,
                            ScreenPosition::default(),
                            RigidBody::KinematicPositionBased,
                            Collider::cuboid(s.x / 2., s.y / 2., s.z / 2.),
                        ));
                        if terrain.ground.unwrap() != attach {
                            let (p0, p1) = (state.p0.unwrap(), state.p1.unwrap());
                            let anchor = (p0 + p1) / 2.;
                            let new_cube_inv = new_cube_tr.compute_affine().inverse();
                            let anchor_attach = attach_inv.transform_point3(anchor);
                            let anchor_new_cube = new_cube_inv.transform_point3(anchor_attach);
                            let hinge = new_cube_inv
                                .transform_vector3(attach_inv.transform_vector3(p1 - p0));

                            new_cube.insert(KinematicHinge {
                                target_angle: 0.,
                                axis: new_cube_tr.right(),
                                anchor: anchor_new_cube,
                                length: hinge.length(),
                                start_dir_up: new_cube_tr.up(),
                                speed: 0.01,
                            });
                        }
                    });

                    clear_ui_state(&mut state, &mut cmd);
                } else {
                    let p2 = state.p2.unwrap();
                    if let (Some(toi0), Some(toi1)) = (
                        ray_toi_with_halfspace(&p2.into(), &cube_tr.right().into(), &ray_p),
                        ray_toi_with_halfspace(&p2.into(), &cube_tr.back().into(), &ray_p),
                    ) {
                        let i0 = ray.origin + toi0 * ray.direction;
                        let i1 = ray.origin + toi1 * ray.direction;
                        let p2_y = cube_tr.up().dot(p2);
                        let y0 = cube_tr.up().dot(i0);
                        let y1 = cube_tr.up().dot(i1);
                        state.height = Some(((y0 + y1) / 2. - p2_y).max(CUBE_INIT_LEN));
                    }
                }
            }
        }
    } else if state.state != AddCubeUiState::None {
        clear_ui_state(&mut state, &mut cmd);
    }
}

fn clear_ui_state(state: &mut AddCubeLocal, cmd: &mut Commands) {
    state.state = AddCubeUiState::None;
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

#[derive(Component)]
pub struct ShootyBall;

fn shoot_balls(
    ui: Res<SidePanelState>,
    materials: Res<BasicMaterialsRes>,
    mouse: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_camera: Query<&MainCamera>,
    q_balls: Query<(Entity, &GlobalTransform), With<ShootyBall>>,
    mut cmd: Commands,
) {
    if ui.mode == UiMode::ShootBalls && !ui.mouse_over {
        if let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) {
            if mouse.just_pressed(MouseButton::Left) {
                cmd.spawn((
                    PbrBundle {
                        transform: Transform::from_translation(ray.origin),
                        mesh: meshes.add(Mesh::from(shape::Icosphere {
                            radius: 1.,
                            subdivisions: 20,
                        })),
                        material: materials.gold.clone(),
                        ..default()
                    },
                    ShootyBall,
                    Selectable,
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
                    Collider::ball(0.5),
                    ColliderMassProperties::Density(0.8),
                    Friction {
                        coefficient: 0.8,
                        combine_rule: CoefficientCombineRule::Average,
                    },
                    Restitution {
                        coefficient: 0.3,
                        combine_rule: CoefficientCombineRule::Average,
                    },
                ));
            }
        }

        for (ball, ball_tr) in &q_balls {
            if ball_tr.translation().y < -50. {
                cmd.entity(ball).despawn_recursive();
            }
        }
    }
}
