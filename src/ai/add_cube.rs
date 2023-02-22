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

pub struct AddCubePlugin;

impl Plugin for AddCubePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AddCubeUiRes::default())
            .add_system(add_cube_ui)
            .add_system(shoot_balls);
    }
}

#[derive(PartialEq)]
enum AddCubeUiState {
    None,
    PickAttachP0,
    PickAttachP1,
    PickLength,
}

#[derive(Resource)]
struct AddCubeUiRes {
    pub state: AddCubeUiState,
    pub attach_p0: Option<Vec3>,
    pub attach_p0_normal: Option<Vec3>,
    pub attach_p1: Option<Vec3>,
    pub length: Option<f32>,
    pub cube: Option<Entity>,
    pub ground: Option<Entity>,
}

impl Default for AddCubeUiRes {
    fn default() -> Self {
        Self {
            state: AddCubeUiState::None,
            attach_p0: None,
            attach_p0_normal: None,
            attach_p1: None,
            length: None,
            cube: None,
            ground: None,
        }
    }
}

const CUBE_INIT_LEN: f32 = 0.1;

fn add_cube_ui(
    ui: Res<SidePanelState>,
    mut res: ResMut<AddCubeUiRes>,
    materials: Res<BasicMaterialsRes>,
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    q_camera: Query<&MainCamera>,
    mut q_tr: Query<&mut Transform>,
    q_gl_tr: Query<&GlobalTransform>,
    q_parent: Query<&Parent>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if ui.mode == UiMode::AddCube {
        if res.state == AddCubeUiState::None {
            res.state = AddCubeUiState::PickAttachP0;
        }

        if let (Some(ground_ent), Some(cube_ent)) = (res.ground, res.cube) {
            if let (Ok(mut cube_mut), Ok(ground), Some(p0), Some(p1)) = (
                q_tr.get_mut(cube_ent),
                q_gl_tr.get(ground_ent),
                res.attach_p0,
                res.attach_p1,
            ) {
                let scale_y = res.length.unwrap_or(CUBE_INIT_LEN);
                let inverse = ground.affine().inverse();
                let p0_ground = inverse.transform_point3(p0);
                let p1_ground = inverse.transform_point3(p1);
                let scale = cube_mut.rotation * (p1_ground - p0_ground);
                cube_mut.scale = Vec3::new(scale.x.abs(), scale_y, scale.z.abs());
                cube_mut.translation = (p0_ground + p1_ground + scale_y * cube_mut.up()) / 2.;
            }
        }

        if ui.mouse_over {
            return;
        }

        if let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) {
            let ray_p = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());
            if res.state == AddCubeUiState::PickAttachP0 {
                if mouse.just_pressed(MouseButton::Left) {
                    let material = materials.ui_transparent.clone();
                    if let (Some(material), Some((attach_ent, hit))) = (
                        material,
                        rapier.cast_ray_and_get_normal(
                            ray.origin,
                            ray.direction,
                            1000.,
                            false,
                            QueryFilter::new(),
                        ),
                    ) {
                        let p0_n = hit.normal.normalize();
                        let p0 = hit.point;
                        let ground = {
                            if let Some(ground) = q_parent.iter_ancestors(attach_ent).last() {
                                ground
                            } else {
                                attach_ent
                            }
                        };
                        res.ground = Some(ground);
                        res.attach_p0 = Some(p0);
                        res.attach_p0_normal = Some(p0_n);
                        res.state = AddCubeUiState::PickAttachP1;

                        let ground_tr = q_gl_tr.get(ground).unwrap();
                        let ground_inv = ground_tr.affine().inverse();
                        let ground_p0 = ground_inv.transform_point3(p0);
                        let dir_y = ground_inv.transform_vector3(p0_n).normalize();
                        let dir_x = {
                            let dir_x = dir_y.cross(ground_tr.back());
                            if dir_x.length() < 0.01 {
                                dir_y.cross(ground_tr.up())
                            } else {
                                dir_x
                            }
                            .normalize()
                        };
                        res.cube = Some(cmd.entity(ground).add_children(|parent| {
                            parent
                                .spawn((
                                    PbrBundle {
                                        transform: Transform::from_translation(
                                            ground_p0 + (CUBE_INIT_LEN / 2.) * dir_y,
                                        )
                                        .with_rotation(Quat::from_mat3(&Mat3::from_cols(
                                            dir_x,
                                            dir_y,
                                            dir_x.cross(dir_y).normalize(),
                                        )))
                                        .with_scale(Vec3::new(0., CUBE_INIT_LEN, 0.)),
                                        mesh: meshes.add(Mesh::from(shape::Box::new(1., 1., 1.))),
                                        material: material.clone(),
                                        ..default()
                                    },
                                    NotShadowCaster,
                                    NotShadowReceiver,
                                ))
                                .id()
                        }));
                    }
                }
            } else if res.state == AddCubeUiState::PickAttachP1 {
                let center = res.attach_p0.unwrap();
                let normal = res.attach_p0_normal.unwrap();
                if let Some(toi) = ray_toi_with_halfspace(&center.into(), &normal.into(), &ray_p) {
                    res.attach_p1 = Some(ray.origin + toi * ray.direction);
                    if mouse.just_pressed(MouseButton::Left) {
                        res.state = AddCubeUiState::PickLength;
                    }
                }
            } else if res.state == AddCubeUiState::PickLength {
                if let (Some(ground), Ok(cube)) = (res.ground, q_gl_tr.get(res.cube.unwrap())) {
                    if mouse.just_pressed(MouseButton::Left) {
                        let material = materials.salmon.clone();
                        let (scale, rotation) = {
                            let srt = cube.to_scale_rotation_translation();
                            (srt.0, srt.1)
                        };
                        cmd.entity(ground).with_children(|parent| {
                            parent
                                .spawn((
                                    PbrBundle {
                                        transform: Transform::from_translation(cube.translation())
                                            .with_rotation(rotation),
                                        mesh: meshes.add(Mesh::from(shape::Box::new(
                                            scale.x, scale.y, scale.z,
                                        ))),
                                        material: material.unwrap(),
                                        ..default()
                                    },
                                    Selectable,
                                    ScreenPosition::default(),
                                    RigidBody::KinematicPositionBased,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        TransformBundle::from(Transform::IDENTITY),
                                        Collider::cuboid(scale.x / 2., scale.y / 2., scale.z / 2.),
                                    ));
                                });
                        });
                        clear_ui_state(&mut res, &mut cmd);
                    } else {
                        let p1 = res.attach_p1.unwrap();
                        if let (Some(toi0), Some(toi1)) = (
                            ray_toi_with_halfspace(&p1.into(), &cube.right().into(), &ray_p),
                            ray_toi_with_halfspace(&p1.into(), &cube.back().into(), &ray_p),
                        ) {
                            let i0 = ray.origin + toi0 * ray.direction;
                            let i1 = ray.origin + toi1 * ray.direction;
                            let p1_y = cube.up().dot(p1);
                            let y0 = cube.up().dot(i0);
                            let y1 = cube.up().dot(i1);
                            res.length = Some(((y0 + y1) / 2. - p1_y).max(CUBE_INIT_LEN));
                        }
                    }
                }
            }
        }
    } else if res.state != AddCubeUiState::None {
        clear_ui_state(&mut res, &mut cmd);
    }
}

fn clear_ui_state(res: &mut ResMut<AddCubeUiRes>, cmd: &mut Commands) {
    res.state = AddCubeUiState::None;
    res.attach_p0 = None;
    res.attach_p0_normal = None;
    res.attach_p1 = None;
    res.length = None;
    if let Some(ent) = res.cube {
        cmd.entity(ent).despawn_recursive();
    }
    res.cube = None;
    res.ground = None;
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
        if let Ok(camera) = q_camera.get_single() {
            if let (Some(ray), Some(mat)) = (camera.mouse_ray, materials.gold.clone()) {
                if mouse.just_pressed(MouseButton::Left) {
                    cmd.spawn((
                        PbrBundle {
                            transform: Transform::from_translation(ray.origin),
                            mesh: meshes.add(Mesh::from(shape::Icosphere {
                                radius: 1.,
                                subdivisions: 20,
                            })),
                            material: mat,
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
                            coefficient: 0.5,
                            combine_rule: CoefficientCombineRule::Average,
                        },
                    ));
                }
            }
        }

        for (ball, ball_tr) in &q_balls {
            if ball_tr.translation().y < -10. {
                cmd.entity(ball).despawn_recursive();
            }
        }
    }
}
