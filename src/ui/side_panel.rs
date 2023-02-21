use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiSettings};
use bevy_rapier3d::render::DebugRenderContext;

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
            .add_system(update_side_panel);
    }
}

fn configure_egui(_egui_ctx: ResMut<EguiContext>, mut egui_settings: ResMut<EguiSettings>) {
    egui_settings.scale_factor = 1.0;
}

#[derive(PartialEq)]
pub enum UiMode {
    Select,
    AddPlatform,
    ShootBalls,
}

#[derive(Resource)]
pub struct SidePanelState {
    pub mode: UiMode,
    pub rapier_debug_enabled: bool,
    pub selected_show_names: bool,
    pub selected_show_move_gizmo: bool,
    pub selected_show_path: bool,
}

impl Default for SidePanelState {
    fn default() -> Self {
        Self {
            mode: UiMode::Select,
            rapier_debug_enabled: false,
            selected_show_names: true,
            selected_show_move_gizmo: true,
            selected_show_path: true,
        }
    }
}

fn update_side_panel(
    mut egui_ctx: ResMut<EguiContext>,
    keyboard: Res<Input<KeyCode>>,
    diagnostics: Res<Diagnostics>,
    mut state: ResMut<SidePanelState>,
    selected_q: Query<&Name, With<Selected>>,
    mut debug_render_ctx: ResMut<DebugRenderContext>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        state.mode = UiMode::Select;
    }

    egui::SidePanel::left("side_panel")
        .default_width(200.0)
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
                    ui.checkbox(&mut state.selected_show_move_gizmo, "Show move gizmos");
                    ui.checkbox(&mut state.selected_show_path, "Show paths");

                    let selected: Vec<_> = selected_q.iter().collect();
                    if !selected.is_empty() {
                        ui.add_space(10.);
                        ui.colored_label(
                            egui::Color32::DARK_GREEN,
                            format!("{} objects selected:", selected.len()),
                        );
                        for name in selected.iter().take(20) {
                            ui.label(format!("- {}", name.as_str()));
                        }
                        if selected.len() > 20 {
                            ui.label("...");
                        }
                    }
                });

            egui::CollapsingHeader::new("Physics")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut state.rapier_debug_enabled, "Debug render");
                    debug_render_ctx.enabled = state.rapier_debug_enabled;
                });

            egui::CollapsingHeader::new("Ui mode")
                .default_open(true)
                .show(ui, |ui| {
                    ui.selectable_value(&mut state.mode, UiMode::Select, "Select");
                    ui.selectable_value(&mut state.mode, UiMode::AddPlatform, "Add platform");
                    ui.selectable_value(&mut state.mode, UiMode::ShootBalls, "Shoot ballz");
                });
        });
}
