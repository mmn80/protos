use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use crate::{camera::MainCamera, ui::side_panel::SidePanelState};

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AddPlatformUiRes::default())
            .add_startup_system(setup_platform_ui)
            .add_system(add_platform_ui);
    }
}

#[derive(PartialEq)]
enum AddPlatformUiState {
    SelectingRectStart,
    SelectingRectEnd,
    SelectingDepth,
}

#[derive(Resource)]
struct AddPlatformUiRes {
    pub platform_ui_mat: Option<Handle<StandardMaterial>>,
    pub state: AddPlatformUiState,
    pub attach_p0: Option<Vec3>,
    pub attach_p0_normal: Option<Vec3>,
    pub attach_p0_tangent: Option<Vec3>,
    pub attach_p1: Option<Vec3>,
    pub length: Option<f32>,
    pub platform: Option<Entity>,
}

impl Default for AddPlatformUiRes {
    fn default() -> Self {
        Self {
            platform_ui_mat: None,
            state: AddPlatformUiState::SelectingRectStart,
            attach_p0: None,
            attach_p0_normal: None,
            attach_p0_tangent: None,
            attach_p1: None,
            length: None,
            platform: None,
        }
    }
}

fn setup_platform_ui(
    mut ui: ResMut<AddPlatformUiRes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    ui.platform_ui_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.9, 0.5, 0.2),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        // unlit: true,
        ..default()
    }));
}

fn add_platform_ui(
    ui_global: Res<SidePanelState>,
    mut ui: ResMut<AddPlatformUiRes>,
    input_mouse: Res<Input<MouseButton>>,
    rapier_ctx: Res<RapierContext>,
    camera_q: Query<&MainCamera>,
    mut platform_q: Query<&mut Transform>,
    transform_q: Query<&GlobalTransform>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if ui_global.add_platform {
        if input_mouse.just_pressed(MouseButton::Right) {
            ui.state = AddPlatformUiState::SelectingRectStart;
            ui.attach_p0 = None;
            ui.attach_p0_normal = None;
            ui.attach_p1 = None;
            ui.length = None;
            if let Some(ent) = ui.platform {
                commands.entity(ent).despawn_recursive();
            }
            ui.platform = None;
            return;
        }
        let ray = {
            if let Ok(Some(ray)) = camera_q.get_single().map(|c| c.mouse_ray.clone()) {
                ray
            } else {
                return;
            }
        };
        let material = {
            if let Some(material) = &ui.platform_ui_mat {
                material.clone()
            } else {
                return;
            }
        };
        if let Some(ent) = ui.platform {
            if let (Ok(mut transform), Some(p0), Some(p1), Some(normal)) = (
                platform_q.get_mut(ent),
                ui.attach_p0,
                ui.attach_p1,
                ui.attach_p0_normal,
            ) {
                let scale_y = ui.length.unwrap_or(0.1);
                let p0_local = transform.rotation * p0;
                let p1_local = transform.rotation * p1;
                let dp = p1_local - p0_local;
                transform.scale = Vec3::new(dp.x.abs(), scale_y, dp.z.abs());
                transform.translation = (p0 + p1 + scale_y * normal) / 2.;
            }
        }
        if ui.state == AddPlatformUiState::SelectingRectStart {
            if input_mouse.just_pressed(MouseButton::Left) {
                if let Some((entity, intersection)) = rapier_ctx.cast_ray_and_get_normal(
                    ray.origin,
                    ray.direction,
                    1000.,
                    false,
                    QueryFilter::new(),
                ) {
                    let p0_n = intersection.normal.normalize();
                    let p0 = intersection.point;
                    println!("Rect started at point {} with normal {}", p0, p0_n);
                    if let Ok(transform) = transform_q.get(entity) {
                        ui.attach_p0 = Some(p0);
                        ui.attach_p0_normal = Some(p0_n);
                        ui.state = AddPlatformUiState::SelectingRectEnd;
                        let right = {
                            if transform.forward().dot(p0_n) < 0.9 {
                                p0_n.cross(-transform.forward())
                            } else if transform.up().dot(p0_n) < 0.9 {
                                p0_n.cross(transform.up())
                            } else if transform.right().dot(p0_n) < 0.9 {
                                p0_n.cross(transform.right())
                            } else {
                                panic!("imposibru!!!")
                            }
                        };
                        ui.attach_p0_tangent = Some(right);
                        ui.platform = Some(
                            commands
                                .spawn(PbrBundle {
                                    transform: Transform::from_translation(p0 + 0.05 * p0_n)
                                        .with_rotation(Quat::from_mat3(&Mat3::from_cols(
                                            right,
                                            p0_n,
                                            right.cross(p0_n).normalize(),
                                        )))
                                        .with_scale(Vec3::new(0., 0.1, 0.)),
                                    mesh: meshes.add(Mesh::from(shape::Box::new(1., 1., 1.))),
                                    material,
                                    ..default()
                                })
                                .id(),
                        );
                    }
                }
            }
        } else if ui.state == AddPlatformUiState::SelectingRectEnd {
            let center = ui.attach_p0.unwrap();
            let normal = ui.attach_p0_normal.unwrap();
            let ray_parry = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());
            if let Some(toi) = ray_toi_with_halfspace(&center.into(), &normal.into(), &ray_parry) {
                let p1 = ray.origin + toi * ray.direction;
                ui.attach_p1 = Some(p1);

                if input_mouse.just_pressed(MouseButton::Left) {
                    println!("Rect completed at point {}", ui.attach_p1.unwrap());
                    ui.state = AddPlatformUiState::SelectingDepth;
                }
            }
        }
    }
}
