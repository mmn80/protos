use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::egui;
use bevy_rapier3d::prelude::*;

use crate::{
    anim::{
        joint::{RevoluteJointCommand, SphericalJointCommand},
        rig::{KiRevoluteJoint, KiSphericalJoint},
    },
    camera::{MainCamera, ScreenPosition},
};

use super::{
    basic_materials::{BasicMaterials, FlipMaterial, RevertFlipMaterial},
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
            .add_systems(
                (update_selected_from_click, update_selected_from_rect)
                    .in_base_set(CoreSet::PreUpdate)
                    .after(crate::camera::update_screen_position),
            )
            .add_systems((
                update_selected_names,
                update_select_ui_rect,
                update_selected_move_gizmos,
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
    mesh: Option<Entity>,
}

#[derive(Clone, Component, Debug, Default, Reflect)]
pub struct SelectionRectUiNode;

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct SelectionRect {
    pub rect: Option<Rect>,
}

impl SelectionRect {
    fn get_fixed_rect(rect: Rect) -> Rect {
        Rect {
            min: rect.min.min(rect.max),
            max: rect.min.max(rect.max),
        }
    }
}

struct DeselectedEvent(Entity);

fn update_selected_from_click(
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    panel: Res<SidePanel>,
    materials: Res<BasicMaterials>,
    q_camera: Query<&MainCamera>,
    q_selectable: Query<&Selectable>,
    q_selected: Query<(Entity, &Selected)>,
    q_sensor: Query<&Sensor>,
    mut ev_deselected: EventWriter<DeselectedEvent>,
    mut cmd: Commands,
) {
    if panel.mouse_over || panel.mode != UiMode::Select || !mouse.just_pressed(MouseButton::Left) {
        return;
    };
    let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else { return };
    let Some((hit_ent, _)) =
        rapier.cast_ray(ray.origin, ray.direction, 1000., false, QueryFilter::new()) else { return };
    if q_sensor.contains(hit_ent) {
        return;
    }

    let shift = keyboard.pressed(KeyCode::LShift);
    let mut sel_ent = None;
    let mut to_deselect = vec![];
    if let Ok(selectable) = q_selectable.get(hit_ent) {
        sel_ent = Some(selectable.selected);
        if !q_selected.contains(selectable.selected) {
            cmd.entity(selectable.selected).insert(Selected {
                mesh: selectable.mesh,
            });

            let mesh = selectable.mesh.unwrap_or(selectable.selected);
            cmd.entity(mesh)
                .insert(FlipMaterial::new(&materials.ui_transparent));
        } else if shift {
            to_deselect.push(selectable.selected);
        }
    }
    if !shift {
        for (selected, _) in &q_selected {
            let mut remove = true;
            if let Some(sel_ent) = sel_ent {
                remove = sel_ent != selected;
            }
            if remove {
                to_deselect.push(selected);
            }
        }
    }
    for deselected in to_deselect {
        if let Ok((selected_ent, selected)) = q_selected.get(deselected) {
            let mesh = selected.mesh.unwrap_or(selected_ent);
            cmd.entity(mesh).insert(RevertFlipMaterial);

            cmd.entity(deselected).remove::<Selected>();
            ev_deselected.send(DeselectedEvent(deselected));
        }
    }
}

fn update_selected_from_rect(
    panel: Res<SidePanel>,
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    materials: Res<BasicMaterials>,
    rapier: Res<RapierContext>,
    mut selection_rect: ResMut<SelectionRect>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<&MainCamera>,
    q_selectable: Query<&Selectable>,
    q_selected: Query<&Selected>,
    q_screen_pos: Query<&ScreenPosition>,
    q_sensor: Query<&Sensor>,
    mut ev_deselected: EventWriter<DeselectedEvent>,
    mut cmd: Commands,
) {
    if panel.mouse_over || panel.mode != UiMode::Select {
        return;
    };
    if mouse.just_pressed(MouseButton::Left) {
        let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else { return };
        if let Some((entity, _)) =
            rapier.cast_ray(ray.origin, ray.direction, 1000., false, QueryFilter::new())
        {
            if q_selectable.contains(entity) || q_sensor.contains(entity) {
                return;
            }
        }
    }
    let Ok(window) = q_window.get_single() else { return };
    let Some(mouse_pos) = window.cursor_position() else { return };
    if mouse.just_pressed(MouseButton::Left) {
        selection_rect.rect = Some(Rect::from_corners(mouse_pos, mouse_pos));
        return;
    }
    let Some(mut sel_rect) = selection_rect.rect else { return };
    if mouse.pressed(MouseButton::Left) {
        sel_rect.max = mouse_pos;
        selection_rect.rect = Some(sel_rect);
        return;
    }
    let sel_rect = SelectionRect::get_fixed_rect(sel_rect);

    for selectable in &q_selectable {
        let Ok(ScreenPosition {
                position,
                camera_dist: _,
            }) = q_screen_pos.get(selectable.selected) else { continue };
        if sel_rect.contains(*position) {
            if !q_selected.contains(selectable.selected) {
                cmd.entity(selectable.selected).insert(Selected {
                    mesh: selectable.mesh,
                });

                let mesh = selectable.mesh.unwrap_or(selectable.selected);
                cmd.entity(mesh)
                    .insert(FlipMaterial::new(&materials.ui_transparent));
            }
        } else if !keyboard.pressed(KeyCode::LShift) {
            if q_selected.contains(selectable.selected) {
                let mesh = selectable.mesh.unwrap_or(selectable.selected);
                cmd.entity(mesh).insert(RevertFlipMaterial);

                cmd.entity(selectable.selected).remove::<Selected>();
                ev_deselected.send(DeselectedEvent(selectable.selected));
            }
        }
    }
    selection_rect.rect = None;
}

fn update_select_ui_rect(
    selection_rect: Res<SelectionRect>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_style: Query<(&mut Style, &mut Visibility), With<SelectionRectUiNode>>,
) {
    let Ok(window) = q_window.get_single() else { return };
    let window_height = window.height();
    let rect = selection_rect
        .rect
        .map(|r| SelectionRect::get_fixed_rect(r));
    for (mut style, mut visibility) in &mut q_style {
        if let Some(rect) = rect {
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

    for DeselectedEvent(deselected) in ev_deselected.iter() {
        if let Ok((_, _, UnitNameUiNodeRef(ui_node))) = moved_q.get(*deselected) {
            cmd.entity(*ui_node).despawn_recursive();
            cmd.entity(*deselected).remove::<UnitNameUiNodeRef>();
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

fn update_selected_move_gizmos(
    selection: Res<SelectionUiState>,
    mut ev_deselected: EventReader<DeselectedEvent>,
    q_selected_gizmo: Query<Entity, (With<HasTransformGizmo>, With<Selected>)>,
    q_selected_no_gizmo: Query<Entity, (Without<HasTransformGizmo>, With<Selected>)>,
    mut cmd: Commands,
) {
    if selection.show_move_gizmo {
        for selected in &q_selected_no_gizmo {
            cmd.entity(selected).add(AddTransformGizmo);
        }
        for DeselectedEvent(deselected) in ev_deselected.iter() {
            cmd.entity(*deselected).add(RemoveTransformGizmo);
        }
    } else {
        for selected in &q_selected_gizmo {
            cmd.entity(selected).add(RemoveTransformGizmo);
        }
    }
}
