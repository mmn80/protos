use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use crate::{
    camera::{MainCamera, ScreenPosition},
    ui::{
        selection::{Selectable, Selected},
        side_panel::SidePanelState,
    },
};

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
    PickAttachP0,
    PickAttachP1,
    PickLength,
}

#[derive(Resource)]
struct AddPlatformUiRes {
    pub platform_ui_mat: Option<Handle<StandardMaterial>>,
    pub platform_mat: Option<Handle<StandardMaterial>>,
    pub state: AddPlatformUiState,
    pub attach_p0: Option<Vec3>,
    pub attach_p0_normal: Option<Vec3>,
    pub attach_p1: Option<Vec3>,
    pub length: Option<f32>,
    pub platform: Option<Entity>,
    pub ground: Option<Entity>,
}

impl Default for AddPlatformUiRes {
    fn default() -> Self {
        Self {
            platform_ui_mat: None,
            platform_mat: None,
            state: AddPlatformUiState::PickAttachP0,
            attach_p0: None,
            attach_p0_normal: None,
            attach_p1: None,
            length: None,
            platform: None,
            ground: None,
        }
    }
}

fn setup_platform_ui(
    mut ui: ResMut<AddPlatformUiRes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    ui.platform_ui_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.9, 0.5, 0.4),
        emissive: Color::rgb(0.5, 0.9, 0.5),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }));
    ui.platform_mat = Some(materials.add(StandardMaterial {
        base_color: Color::SALMON,
        metallic: 0.2,
        perceptual_roughness: 0.8,
        reflectance: 0.5,
        ..default()
    }));
}

const PLATFORM_INIT_LEN: f32 = 0.1;

fn add_platform_ui(
    mut ui_global: ResMut<SidePanelState>,
    mut ui: ResMut<AddPlatformUiRes>,
    input_mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    camera_q: Query<&MainCamera>,
    mut tr_q: Query<&mut Transform>,
    gl_tr_q: Query<&GlobalTransform>,
    q_parent: Query<&Parent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if ui_global.add_platform {
        if input_mouse.just_pressed(MouseButton::Right) {
            clear_ui_state(&mut ui, &mut commands);
            ui_global.add_platform = false;
            return;
        }

        if let (Some(ground_ent), Some(platform_ent)) = (ui.ground, ui.platform) {
            if let (Ok(mut platform_mut), Ok(ground), Some(p0), Some(p1)) = (
                tr_q.get_mut(platform_ent),
                gl_tr_q.get(ground_ent),
                ui.attach_p0,
                ui.attach_p1,
            ) {
                let scale_y = ui.length.unwrap_or(PLATFORM_INIT_LEN);
                let inverse = ground.affine().inverse();
                let p0_ground = inverse.transform_point3(p0);
                let p1_ground = inverse.transform_point3(p1);
                let scale = platform_mut.rotation * (p1_ground - p0_ground);
                platform_mut.scale = Vec3::new(scale.x.abs(), scale_y, scale.z.abs());
                platform_mut.translation =
                    (p0_ground + p1_ground + scale_y * platform_mut.up()) / 2.;
            }
        }

        if let Ok(Some(ray)) = camera_q.get_single().map(|c| c.mouse_ray.clone()) {
            let ray_p = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());
            if ui.state == AddPlatformUiState::PickAttachP0 {
                if input_mouse.just_pressed(MouseButton::Left) {
                    let material = ui.platform_ui_mat.clone();
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
                        ui.ground = Some(ground);
                        ui.attach_p0 = Some(p0);
                        ui.attach_p0_normal = Some(p0_n);
                        ui.state = AddPlatformUiState::PickAttachP1;

                        let ground_tr = gl_tr_q.get(ground).unwrap();
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
                        ui.platform = Some(commands.entity(ground).add_children(|parent| {
                            parent
                                .spawn((
                                    PbrBundle {
                                        transform: Transform::from_translation(
                                            ground_p0 + (PLATFORM_INIT_LEN / 2.) * dir_y,
                                        )
                                        .with_rotation(Quat::from_mat3(&Mat3::from_cols(
                                            dir_x,
                                            dir_y,
                                            dir_x.cross(dir_y).normalize(),
                                        )))
                                        .with_scale(Vec3::new(0., PLATFORM_INIT_LEN, 0.)),
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
            } else if ui.state == AddPlatformUiState::PickAttachP1 {
                let center = ui.attach_p0.unwrap();
                let normal = ui.attach_p0_normal.unwrap();
                if let Some(toi) = ray_toi_with_halfspace(&center.into(), &normal.into(), &ray_p) {
                    ui.attach_p1 = Some(ray.origin + toi * ray.direction);
                    if input_mouse.just_pressed(MouseButton::Left) {
                        ui.state = AddPlatformUiState::PickLength;
                    }
                }
            } else if ui.state == AddPlatformUiState::PickLength {
                if let (Some(ground), Ok(platform)) = (ui.ground, gl_tr_q.get(ui.platform.unwrap()))
                {
                    if input_mouse.just_pressed(MouseButton::Left) {
                        let material = ui.platform_mat.clone();
                        let (scale, rotation) = {
                            let srt = platform.to_scale_rotation_translation();
                            (srt.0, srt.1)
                        };
                        commands.entity(ground).with_children(|parent| {
                            parent
                                .spawn(PbrBundle {
                                    transform: Transform::from_translation(platform.translation())
                                        .with_rotation(rotation),
                                    mesh: meshes.add(Mesh::from(shape::Box::new(
                                        scale.x, scale.y, scale.z,
                                    ))),
                                    material: material.unwrap(),
                                    ..default()
                                })
                                .insert((
                                    Selectable,
                                    ScreenPosition::default(),
                                    Selected,
                                    RigidBody::Fixed,
                                ))
                                .with_children(|parent| {
                                    parent
                                        .spawn(Collider::cuboid(
                                            scale.x / 2.,
                                            scale.y / 2.,
                                            scale.z / 2.,
                                        ))
                                        .insert(TransformBundle::from(Transform::IDENTITY));
                                });
                        });
                        clear_ui_state(&mut ui, &mut commands);
                    } else {
                        let p1 = ui.attach_p1.unwrap();
                        if let (Some(toi0), Some(toi1)) = (
                            ray_toi_with_halfspace(&p1.into(), &platform.right().into(), &ray_p),
                            ray_toi_with_halfspace(&p1.into(), &platform.back().into(), &ray_p),
                        ) {
                            let i0 = ray.origin + toi0 * ray.direction;
                            let i1 = ray.origin + toi1 * ray.direction;
                            let p1_y = platform.up().dot(p1);
                            let y0 = platform.up().dot(i0);
                            let y1 = platform.up().dot(i1);
                            ui.length = Some(((y0 + y1) / 2. - p1_y).max(PLATFORM_INIT_LEN));
                        }
                    }
                }
            }
        }
    }
}

fn clear_ui_state(ui: &mut ResMut<AddPlatformUiRes>, commands: &mut Commands) {
    ui.state = AddPlatformUiState::PickAttachP0;
    ui.attach_p0 = None;
    ui.attach_p0_normal = None;
    ui.attach_p1 = None;
    ui.length = None;
    if let Some(ent) = ui.platform {
        commands.entity(ent).despawn_recursive();
    }
    ui.platform = None;
    ui.ground = None;
}
