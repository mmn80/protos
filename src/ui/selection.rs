use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::egui;
use bevy_rapier3d::prelude::*;

use crate::{
    anim::{
        auto_collider::{AutoCollider, AutoColliderMesh},
        joint::{RevoluteJointCommand, SphericalJointCommand},
        rig::{KiRevoluteJoint, KiSphericalJoint},
    },
    camera::{MainCamera, ScreenPosition},
};

use super::{
    basic_materials::BasicMaterials,
    side_panel::{SidePanel, UiMode},
    transform_gizmo::{AddTransformGizmo, HasTransformGizmo, RemoveTransformGizmo},
};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Selectable>()
            .register_type::<Selected>()
            .init_resource::<SelectionUiState>()
            .init_resource::<SelectionRect>()
            .add_event::<DeselectedEvent>()
            .add_startup_system(setup)
            .add_system(
                update_selected
                    .in_base_set(CoreSet::PreUpdate)
                    .after(crate::camera::update_screen_position),
            )
            .add_systems((
                update_selected_names,
                update_select_ui_rect,
                update_move_gizmo,
            ));
    }
}

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        background_color: Color::NONE.into(),
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: Color::rgba(0.1, 0.8, 0.1, 0.1).into(),
                visibility: Visibility::Hidden,
                ..default()
            },
            SelectionRectUiNode,
        ));
    });
    cmd.insert_resource(LoadedFont(asset_server.load("fonts/FiraMono-Medium.ttf")));
}

#[derive(Clone, Component, Debug, Reflect)]
pub struct Selectable {
    pub selected: Entity,
    pub mesh: Option<Entity>,
}

impl Selectable {
    pub fn new(selected: Entity, mesh: Option<Entity>) -> Self {
        Self { selected, mesh }
    }
}

#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct Selected {
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Entity>,
}

#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct SelectionRectUiNode;

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct SelectionRect {
    pub clear_previous: bool,
    pub begin: Option<Vec2>,
    pub end: Option<Vec2>,
}

impl SelectionRect {
    pub fn get_rect(&self) -> Option<Rect> {
        if let (Some(begin), Some(end)) = (self.begin, self.end) {
            Some(Rect::from_corners(begin, end))
        } else {
            None
        }
    }
}

struct DeselectedEvent(Entity);

fn update_selected(
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    panel: Res<SidePanel>,
    materials: Res<BasicMaterials>,
    mut selection_rect: ResMut<SelectionRect>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&MainCamera>,
    q_selectable: Query<&Selectable>,
    q_selected: Query<(Entity, &Selected)>,
    q_screen_pos: Query<&ScreenPosition>,
    q_sensor: Query<&Sensor>,
    mut q_material: Query<&mut Handle<StandardMaterial>>,
    mut ev_deselected: EventWriter<DeselectedEvent>,
    mut cmd: Commands,
) {
    if panel.mouse_over || panel.mode != UiMode::Select {
        return;
    };

    let mut processed_single = false;

    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) {
            if let Some((hit_ent, _)) =
                rapier.cast_ray(ray.origin, ray.direction, 1000., false, QueryFilter::new())
            {
                if !q_sensor.contains(hit_ent) {
                    let shift = keyboard.pressed(KeyCode::LShift);
                    let mut sel_ent = None;
                    let mut to_remove = vec![];
                    if let Ok(selectable) = q_selectable.get(hit_ent) {
                        processed_single = true;
                        sel_ent = Some(selectable.selected);
                        if !q_selected.contains(selectable.selected) {
                            let mut material = None;
                            if let Some(mesh_ent) = selectable.mesh {
                                if let Ok(mut mat) = q_material.get_mut(mesh_ent) {
                                    material = Some(mat.clone());
                                    *mat = materials.ui_transparent.clone();
                                }
                            }
                            cmd.entity(selectable.selected).insert(Selected {
                                material,
                                mesh: selectable.mesh,
                            });
                        } else if shift {
                            to_remove.push(selectable.selected);
                        }
                    }
                    if !shift {
                        for (selected, _) in &q_selected {
                            let mut remove = true;
                            if let Some(sel_ent) = sel_ent {
                                remove = sel_ent != selected;
                            }
                            if remove {
                                to_remove.push(selected);
                            }
                        }
                    }
                    for deselected in to_remove {
                        let Ok((_, selected)) = q_selected.get(deselected) else { continue };
                        if let Some(material) = selected.material.clone() {
                            if let Some(mesh_ent) = selected.mesh {
                                if let Ok(mut mat) = q_material.get_mut(mesh_ent) {
                                    *mat = material;
                                }
                            }
                        }
                        cmd.entity(deselected).remove::<Selected>();
                        ev_deselected.send(DeselectedEvent(deselected));
                    }
                } else {
                    processed_single = true;
                }
            }
        }
    }

    if !processed_single {
        let do_select_rect = {
            selection_rect.clear_previous = !keyboard.pressed(KeyCode::LShift);
            let Ok(window) = q_window.get_single() else { return };
            let mouse_pos = window.cursor_position();
            if mouse.just_pressed(MouseButton::Left) {
                selection_rect.begin = mouse_pos.clone();
                selection_rect.end = selection_rect.begin;
            } else if selection_rect.begin.is_some() {
                if mouse.pressed(MouseButton::Left) && mouse_pos.is_some() {
                    selection_rect.end = Some(mouse_pos.unwrap());
                } else if !mouse.just_released(MouseButton::Left) || mouse_pos.is_none() {
                    selection_rect.begin = None;
                    selection_rect.end = None;
                }
            }
            if mouse.just_released(MouseButton::Left) {
                selection_rect.get_rect()
            } else {
                None
            }
        };

        let Some(rect) = do_select_rect else { return };
        for selectable in &q_selectable {
            let Ok(ScreenPosition {
                position,
                camera_dist: _,
            }) = q_screen_pos.get(selectable.selected) else { continue };
            if position.x > rect.min.x
                && position.x < rect.max.x
                && position.y < rect.max.y
                && position.y > rect.min.y
            {
                cmd.entity(selectable.selected).insert(Selected {
                    material: None,
                    mesh: None,
                });
            } else if selection_rect.clear_previous {
                cmd.entity(selectable.selected).remove::<Selected>();
                ev_deselected.send(DeselectedEvent(selectable.selected));
            }
        }
        selection_rect.begin = None;
        selection_rect.end = None;
    }
}

fn update_select_ui_rect(
    selection_rect: Res<SelectionRect>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_style: Query<(&mut Style, &mut Visibility), With<SelectionRectUiNode>>,
) {
    let Ok(window) = q_window.get_single() else { return };
    let window_height = window.height();
    for (mut style, mut visibility) in &mut q_style {
        if let Some(rect) = selection_rect.get_rect() {
            style.size.width = Val::Px(rect.width());
            style.size.height = Val::Px(rect.height());
            style.position.left = Val::Px(rect.min.x);
            style.position.right = Val::Px(rect.max.x);
            style.position.bottom = Val::Px(window_height - rect.min.y);
            style.position.top = Val::Px(window_height - rect.max.y);
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

#[derive(Resource)]
pub struct SelectionUiState {
    pub show_inspector: bool,
    pub show_names: bool,
    pub show_move_gizmo: bool,
    pub revolute_target_angle: i16,
    pub spherical_target_angle_x: i16,
    pub spherical_target_angle_y: i16,
    pub spherical_target_angle_z: i16,
    pub joint_stop_at_collisions: bool,
}

impl Default for SelectionUiState {
    fn default() -> Self {
        Self {
            show_inspector: true,
            show_names: true,
            show_move_gizmo: true,
            revolute_target_angle: 0,
            spherical_target_angle_x: 0,
            spherical_target_angle_y: 0,
            spherical_target_angle_z: 0,
            joint_stop_at_collisions: false,
        }
    }
}

#[derive(Resource)]
struct LoadedFont(Handle<Font>);

#[derive(Clone, Component, Debug, Default)]
pub struct UnitNameUiNode;

#[derive(Clone, Component, Debug)]
pub struct UnitNameUiNodeRef(Entity);

fn update_selected_names(
    panel: Res<SelectionUiState>,
    loaded_font: Res<LoadedFont>,
    added_q: Query<(Entity, &Name, &ScreenPosition), Added<Selected>>,
    moved_q: Query<(Entity, &ScreenPosition, &UnitNameUiNodeRef)>,
    mut nodes_q: Query<(&mut Transform, &mut Style), With<UnitNameUiNode>>,
    mut ev_deselected: EventReader<DeselectedEvent>,
    mut cmd: Commands,
) {
    if panel.show_names {
        let text_alignment = TextAlignment::Center;
        let text_style = TextStyle {
            font: loaded_font.0.clone(),
            font_size: 20.0,
            color: Color::SILVER,
        };

        for (entity, name, screen_pos) in &added_q {
            let cam_fact = 1. / screen_pos.camera_dist;
            let text_ent = cmd
                .spawn((
                    TextBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                left: Val::Px(screen_pos.position.x - 50. - 200. * cam_fact),
                                right: Val::Auto,
                                top: Val::Auto,
                                bottom: Val::Px(screen_pos.position.y - 3000. * cam_fact),
                            },
                            ..default()
                        },
                        text: Text::from_section(name.to_string(), text_style.clone())
                            .with_alignment(text_alignment.clone()),
                        transform: Transform::from_scale(Vec3::ONE * (50. * cam_fact)),
                        ..default()
                    },
                    UnitNameUiNode,
                ))
                .id();
            cmd.entity(entity).insert(UnitNameUiNodeRef(text_ent));
        }
    }

    for (entity, screen_pos, UnitNameUiNodeRef(ui_node)) in &moved_q {
        if panel.show_names {
            if let Ok((mut transform, mut style)) = nodes_q.get_mut(*ui_node) {
                let cam_fact = 1. / screen_pos.camera_dist;
                style.position.left = Val::Px(screen_pos.position.x - 50. - 200. * cam_fact);
                style.position.bottom = Val::Px(screen_pos.position.y - 3000. * cam_fact);
                transform.scale = Vec3::ONE * (50. * cam_fact);
            }
        } else {
            cmd.entity(*ui_node).despawn_recursive();
            cmd.entity(entity).remove::<UnitNameUiNodeRef>();
        }
    }

    for DeselectedEvent(unit_ent) in ev_deselected.iter() {
        if let Ok((_, _, UnitNameUiNodeRef(ui_node))) = moved_q.get(*unit_ent) {
            cmd.entity(*ui_node).despawn_recursive();
            cmd.entity(*unit_ent).remove::<UnitNameUiNodeRef>();
        }
    }
}

pub fn selection_ui(
    ui: &mut egui::Ui,
    selection: &mut SelectionUiState,
    selected: Vec<(
        Entity,
        Option<&Name>,
        Option<&KiRevoluteJoint>,
        Option<&KiSphericalJoint>,
    )>,
    mut cmd: Commands,
) {
    egui::CollapsingHeader::new("Selection")
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(&mut selection.show_names, "Show names");
            ui.checkbox(&mut selection.show_inspector, "Show inspector");
            ui.checkbox(&mut selection.show_move_gizmo, "Show move gizmo");

            if !selected.is_empty() {
                ui.add_space(10.);
                ui.colored_label(
                    egui::Color32::DARK_GREEN,
                    format!("{} objects selected:", selected.len()),
                );
                for (ent, name, _, _) in selected.iter().take(20) {
                    if let Some(name) = name {
                        ui.label(format!("- {}", name.as_str()));
                    } else {
                        ui.label(format!("- {:?}", ent));
                    }
                }
                if selected.len() > 20 {
                    ui.label("...");
                }
            }

            if selected.len() == 1 {
                let single = selected.first().unwrap();
                if let (ent, _, Some(_), None) = single {
                    ui.group(|ui| {
                        ui.strong("Revolute joint");
                        ui.add(
                            egui::Slider::new(&mut selection.revolute_target_angle, -180..=180)
                                .text("angle"),
                        );
                        ui.checkbox(
                            &mut selection.joint_stop_at_collisions,
                            "Stop at collisions",
                        );
                        if ui.button("Add joint target").clicked() {
                            cmd.entity(*ent).insert(RevoluteJointCommand::new(
                                selection.revolute_target_angle as f32 * PI / 180.,
                                0.01,
                                selection.joint_stop_at_collisions,
                            ));
                        }
                    });
                } else if let (ent, _, None, Some(_)) = single {
                    ui.group(|ui| {
                        ui.strong("Spherical joint");
                        ui.add(
                            egui::Slider::new(&mut selection.spherical_target_angle_x, -180..=180)
                                .text("angle x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut selection.spherical_target_angle_z, -180..=180)
                                .text("angle z"),
                        );
                        ui.add(
                            egui::Slider::new(&mut selection.spherical_target_angle_y, -180..=180)
                                .text("angle y"),
                        );
                        ui.checkbox(
                            &mut selection.joint_stop_at_collisions,
                            "Stop at collisions",
                        );
                        if ui.button("Add joint target").clicked() {
                            cmd.entity(*ent).insert(SphericalJointCommand::new_euler(
                                selection.spherical_target_angle_x as f32 * PI / 180.,
                                selection.spherical_target_angle_z as f32 * PI / 180.,
                                selection.spherical_target_angle_y as f32 * PI / 180.,
                                0.02,
                                selection.joint_stop_at_collisions,
                            ));
                        }
                    });
                }
            }
        });
}

fn update_move_gizmo(
    selection: Res<SelectionUiState>,
    mut ev_deselected: EventReader<DeselectedEvent>,
    q_selected_gizmo: Query<
        (Entity, Option<&AutoCollider>),
        (With<Selected>, With<HasTransformGizmo>),
    >,
    q_selected_no_gizmo: Query<
        (Entity, Option<&AutoCollider>),
        (With<Selected>, Without<HasTransformGizmo>),
    >,
    q_ac_mesh: Query<With<HasTransformGizmo>, With<AutoColliderMesh>>,
    mut cmd: Commands,
) {
    if selection.show_move_gizmo {
        for (entity, maybe_ac) in &q_selected_no_gizmo {
            let mut selected = entity;
            if let Some(ac) = maybe_ac {
                if q_ac_mesh.contains(ac.mesh) {
                    selected = ac.mesh;
                }
            }
            cmd.entity(selected).add(AddTransformGizmo);
        }
        for DeselectedEvent(deselected) in ev_deselected.iter() {
            cmd.entity(*deselected).add(RemoveTransformGizmo);
        }
    } else {
        for (entity, maybe_ac) in &q_selected_gizmo {
            let mut selected = entity;
            if let Some(ac) = maybe_ac {
                if q_ac_mesh.contains(ac.mesh) {
                    selected = ac.mesh;
                }
            }
            cmd.entity(selected).add(RemoveTransformGizmo);
        }
    }
}
