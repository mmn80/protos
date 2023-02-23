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
        app.add_system(add_cube_ui).add_system(shoot_balls);
    }
}

#[derive(PartialEq, Eq)]
enum AddCubeUiState {
    None,
    PickAttachP0,
    PickAttachP1,
    PickAttachP2,
    PickLength,
}

impl Default for AddCubeUiState {
    fn default() -> Self {
        AddCubeUiState::None
    }
}

#[derive(Default)]
struct AddCubeUiRes {
    state: AddCubeUiState,
    attach_p0: Option<Vec3>,
    attach_p0_normal: Option<Vec3>,
    attach_p1: Option<Vec3>,
    attach_p2: Option<Vec3>,
    length: Option<f32>,
    cube: Option<Entity>,
}

const CUBE_INIT_LEN: f32 = 0.1;

fn add_cube_ui(
    ui: Res<SidePanelState>,
    mut state: Local<AddCubeUiRes>,
    materials: Res<BasicMaterialsRes>,
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    q_camera: Query<&MainCamera>,
    mut q_trans: Query<&mut Transform>,
    q_global_trans: Query<&GlobalTransform>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if ui.mode == UiMode::AddCube {
        if state.state == AddCubeUiState::None {
            state.state = AddCubeUiState::PickAttachP0;
        }

        if let Some(cube) = state.cube {
            if let (Ok(mut cube_trans), Some(p0), Some(p0_n), Some(p1)) = (
                q_trans.get_mut(cube),
                state.attach_p0,
                state.attach_p0_normal,
                state.attach_p1,
            ) {
                let dir_x = (p1 - p0).normalize();
                cube_trans.rotation =
                    Quat::from_mat3(&Mat3::from_cols(dir_x, p0_n, dir_x.cross(p0_n).normalize()));
                let length = state.length.unwrap_or(CUBE_INIT_LEN);
                cube_trans.translation = (p0 + p1 + length * cube_trans.up()) / 2.;
                let p2 = state
                    .attach_p2
                    .unwrap_or(p1 + cube_trans.back() * CUBE_INIT_LEN);
                cube_trans.scale = Vec3::new(
                    (p1 - p0).length(),
                    length,
                    (p2 - p1).dot(cube_trans.back()).abs() * 2.,
                );
            }
        }

        if ui.mouse_over {
            return;
        }

        if let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) {
            let ray_p = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());
            if state.state == AddCubeUiState::PickAttachP0 {
                if mouse.just_pressed(MouseButton::Left) {
                    let material = materials.ui_transparent.clone();
                    if let Some((attach, hit)) = rapier.cast_ray_and_get_normal(
                        ray.origin,
                        ray.direction,
                        1000.,
                        false,
                        QueryFilter::new(),
                    ) {
                        let p0_n = hit.normal.normalize();
                        let p0 = hit.point;

                        state.attach_p0 = Some(p0);
                        state.attach_p0_normal = Some(p0_n);
                        state.state = AddCubeUiState::PickAttachP1;

                        let attach_tr = q_global_trans.get(attach).unwrap();
                        let dir_x = {
                            let dir_x = p0_n.cross(attach_tr.back());
                            if dir_x.length() < 0.01 {
                                p0_n.cross(attach_tr.up())
                            } else {
                                dir_x
                            }
                            .normalize()
                        };
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
            } else if state.state == AddCubeUiState::PickAttachP1 {
                let p0 = state.attach_p0.unwrap();
                let normal = state.attach_p0_normal.unwrap();
                if let Some(toi) = ray_toi_with_halfspace(&p0.into(), &normal.into(), &ray_p) {
                    state.attach_p1 = Some(ray.origin + toi * ray.direction);
                    if mouse.just_pressed(MouseButton::Left) {
                        state.state = AddCubeUiState::PickAttachP2;
                    }
                }
            } else if state.state == AddCubeUiState::PickAttachP2 {
                let p0 = state.attach_p0.unwrap();
                let normal = state.attach_p0_normal.unwrap();
                if let Some(toi) = ray_toi_with_halfspace(&p0.into(), &normal.into(), &ray_p) {
                    state.attach_p2 = Some(ray.origin + toi * ray.direction);
                    if mouse.just_pressed(MouseButton::Left) {
                        state.state = AddCubeUiState::PickLength;
                    }
                }
            } else if state.state == AddCubeUiState::PickLength {
                if let Ok(cube) = q_global_trans.get(state.cube.unwrap()) {
                    if mouse.just_pressed(MouseButton::Left) {
                        let material = materials.salmon.clone();
                        let (scale, rotation) = {
                            let srt = cube.to_scale_rotation_translation();
                            (srt.0, srt.1)
                        };
                        cmd.spawn((
                            PbrBundle {
                                transform: Transform::from_translation(cube.translation())
                                    .with_rotation(rotation),
                                mesh: meshes
                                    .add(Mesh::from(shape::Box::new(scale.x, scale.y, scale.z))),
                                material,
                                ..default()
                            },
                            Selectable,
                            ScreenPosition::default(),
                            RigidBody::KinematicPositionBased,
                            Collider::cuboid(scale.x / 2., scale.y / 2., scale.z / 2.),
                        ));
                        clear_ui_state(&mut state, &mut cmd);
                    } else {
                        let p2 = state.attach_p2.unwrap();
                        if let (Some(toi0), Some(toi1)) = (
                            ray_toi_with_halfspace(&p2.into(), &cube.right().into(), &ray_p),
                            ray_toi_with_halfspace(&p2.into(), &cube.back().into(), &ray_p),
                        ) {
                            let i0 = ray.origin + toi0 * ray.direction;
                            let i1 = ray.origin + toi1 * ray.direction;
                            let p2_y = cube.up().dot(p2);
                            let y0 = cube.up().dot(i0);
                            let y1 = cube.up().dot(i1);
                            state.length = Some(((y0 + y1) / 2. - p2_y).max(CUBE_INIT_LEN));
                        }
                    }
                }
            }
        }
    } else if state.state != AddCubeUiState::None {
        clear_ui_state(&mut state, &mut cmd);
    }
}

fn clear_ui_state(state: &mut AddCubeUiRes, cmd: &mut Commands) {
    state.state = AddCubeUiState::None;
    state.attach_p0 = None;
    state.attach_p0_normal = None;
    state.attach_p1 = None;
    state.attach_p2 = None;
    state.length = None;
    if let Some(cube) = state.cube {
        cmd.entity(cube).despawn_recursive();
    }
    state.cube = None;
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
            if let Some(ray) = camera.mouse_ray {
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
