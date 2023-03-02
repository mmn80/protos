use std::f32::consts::PI;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiSettings};
use bevy_rapier3d::render::DebugRenderContext;

use crate::ai::kinematic_joints::{
    KinematicJointType, RevoluteJoint, RevoluteJointCommand, SphericalJoint, SphericalJointCommand,
};

use super::selection::Selected;

pub struct SidePanelPlugin;

impl Plugin for SidePanelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SidePanelState::default())
            .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            //.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
            //.add_plugin(bevy::wgpu::diagnostic::WgpuResourceDiagnosticsPlugin::default())
            //.add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
            //.add_plugin(bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<Mesh>::default())
            .insert_resource(SidePanelState::default())
            .add_startup_system(configure_egui)
            .add_system(update_side_panel)
            .add_system(inspector_ui);
    }
}

fn configure_egui(_egui_ctx: ResMut<EguiContext>, mut egui_settings: ResMut<EguiSettings>) {
    egui_settings.scale_factor = 1.0;
}

#[derive(PartialEq)]
pub enum UiMode {
    Select,
    AddCube,
    ShootBalls,
}

#[derive(Resource)]
pub struct SidePanelState {
    pub mouse_over: bool,
    pub mode: UiMode,
    pub rapier_debug_enabled: bool,
    pub revolute_target_angle: i16,
    pub spherical_target_angle_x: i16,
    pub spherical_target_angle_y: i16,
    pub spherical_target_angle_z: i16,
    pub joint_stop_at_collisions: bool,
    pub add_joint_type: KinematicJointType,
    pub selected_show_inspector: bool,
    pub selected_show_names: bool,
    pub selected_show_move_gizmo: bool,
}

impl Default for SidePanelState {
    fn default() -> Self {
        Self {
            mouse_over: false,
            mode: UiMode::Select,
            rapier_debug_enabled: false,
            revolute_target_angle: 0,
            spherical_target_angle_x: 0,
            spherical_target_angle_y: 0,
            spherical_target_angle_z: 0,
            joint_stop_at_collisions: false,
            add_joint_type: KinematicJointType::Revolute,
            selected_show_names: true,
            selected_show_inspector: true,
            selected_show_move_gizmo: false,
        }
    }
}

const SIDE_PANEL_WIDTH: f32 = 250.;
const INSPECTOR_WIDTH: f32 = 300.;

fn update_side_panel(
    mut egui_ctx: ResMut<EguiContext>,
    windows: Res<Windows>,
    keyboard: Res<Input<KeyCode>>,
    diagnostics: Res<Diagnostics>,
    mut state: ResMut<SidePanelState>,
    mut debug_render_ctx: ResMut<DebugRenderContext>,
    q_selected: Query<
        (
            Entity,
            Option<&Name>,
            Option<&RevoluteJoint>,
            Option<&SphericalJoint>,
        ),
        With<Selected>,
    >,
    mut cmd: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        state.mode = UiMode::Select;
    }

    state.mouse_over = true;
    if let Some(window) = windows.get_primary() {
        if let Some(mouse_pos) = window.cursor_position() {
            state.mouse_over = mouse_pos.x <= SIDE_PANEL_WIDTH;
            if !state.mouse_over && state.selected_show_inspector && !q_selected.is_empty() {
                state.mouse_over = mouse_pos.x >= window.width() - INSPECTOR_WIDTH;
            }
        }
    }

    let selected: Vec<_> = q_selected.iter().collect();

    egui::SidePanel::left("side_panel")
        .exact_width(SIDE_PANEL_WIDTH)
        .show(egui_ctx.ctx_mut(), |ui| {
            let fps = diagnostics
                .get_measurement(FrameTimeDiagnosticsPlugin::FPS)
                .map(|d| d.value.round() as u32)
                .unwrap_or(0);
            let frame = diagnostics
                .get_measurement(FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                .map(|d| d.value as u32)
                .unwrap_or(0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Frame: {}", frame));
                    ui.separator();
                    ui.label(format!("FPS: {}", fps));
                });
            });

            egui::CollapsingHeader::new("Selection")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut state.selected_show_names, "Show names");
                    ui.checkbox(&mut state.selected_show_inspector, "Show inspector");
                    ui.checkbox(&mut state.selected_show_move_gizmo, "Show move gizmo");

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
                                    egui::Slider::new(&mut state.revolute_target_angle, -180..=180)
                                        .text("angle"),
                                );
                                ui.checkbox(
                                    &mut state.joint_stop_at_collisions,
                                    "Stop at collisions",
                                );
                                if ui.button("Add joint target").clicked() {
                                    cmd.entity(*ent).insert(RevoluteJointCommand::new(
                                        state.revolute_target_angle as f32 * PI / 180.,
                                        0.01,
                                        state.joint_stop_at_collisions,
                                    ));
                                }
                            });
                        } else if let (ent, _, None, Some(_)) = single {
                            ui.group(|ui| {
                                ui.strong("Spherical joint");
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.spherical_target_angle_x,
                                        -180..=180,
                                    )
                                    .text("angle x"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.spherical_target_angle_z,
                                        -180..=180,
                                    )
                                    .text("angle z"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut state.spherical_target_angle_y,
                                        -180..=180,
                                    )
                                    .text("angle y"),
                                );
                                ui.checkbox(
                                    &mut state.joint_stop_at_collisions,
                                    "Stop at collisions",
                                );
                                if ui.button("Add joint target").clicked() {
                                    cmd.entity(*ent).insert(SphericalJointCommand::new_euler(
                                        state.spherical_target_angle_x as f32 * PI / 180.,
                                        state.spherical_target_angle_z as f32 * PI / 180.,
                                        state.spherical_target_angle_y as f32 * PI / 180.,
                                        0.02,
                                        state.joint_stop_at_collisions,
                                    ));
                                }
                            });
                        }
                    }
                });

            egui::CollapsingHeader::new("Physics")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut state.rapier_debug_enabled, "Debug render");
                    debug_render_ctx.enabled = state.rapier_debug_enabled;

                    state_toggle(ui, &mut state, UiMode::AddCube, "Add cube");

                    if state.mode == UiMode::AddCube {
                        ui.indent(10, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Joint type:");
                                ui.selectable_value(
                                    &mut state.add_joint_type,
                                    KinematicJointType::Revolute,
                                    "Revolute",
                                );
                                ui.selectable_value(
                                    &mut state.add_joint_type,
                                    KinematicJointType::Spherical,
                                    "Spherical",
                                );
                            });
                        });
                    }

                    state_toggle(ui, &mut state, UiMode::ShootBalls, "Shoot balls");
                });
        });
}

fn state_toggle(ui: &mut egui::Ui, state: &mut SidePanelState, mode: UiMode, text: &str) {
    let mut val = state.mode == mode;
    let val1 = val;
    ui.toggle_value(&mut val, text);
    if val {
        state.mode = mode;
    } else if val1 {
        state.mode = UiMode::Select
    };
}

fn inspector_ui(world: &mut World) {
    let egui_context = world
        .resource_mut::<bevy_egui::EguiContext>()
        .ctx_mut()
        .clone();

    {
        let state = world.resource::<SidePanelState>();
        if !state.selected_show_inspector {
            return;
        }
    }

    let selected = {
        let mut q_selected = world.query_filtered::<(Entity, Option<&Name>), With<Selected>>();
        q_selected.iter(&world).next().map(|(entity, name)| {
            (
                entity,
                if let Some(name) = name {
                    format!("Inspector: {}", name.as_str())
                } else {
                    format!("Inspector: {:?}", entity)
                },
            )
        })
    };

    if let Some((entity, name)) = selected {
        egui::SidePanel::right("inspector")
            .exact_width(INSPECTOR_WIDTH)
            .show(&egui_context, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(name);
                    bevy_inspector_egui::bevy_inspector::ui_for_entity_with_children(
                        world, entity, ui,
                    );
                });
            });
    }
}
