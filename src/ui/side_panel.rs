use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContexts, EguiSettings};
use bevy_rapier3d::render::DebugRenderContext;

use crate::anim::rig::{KiRevoluteJoint, KiSphericalJoint};

use super::{
    add_cube::{add_cube_ui, AddCubeUiState},
    selection::{selection_ui, Selected, SelectionUiState},
};

pub struct SidePanelPlugin;

impl Plugin for SidePanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SidePanelState>()
            .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            //.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
            //.add_plugin(bevy::wgpu::diagnostic::WgpuResourceDiagnosticsPlugin::default())
            //.add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
            //.add_plugin(bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<Mesh>::default())
            .init_resource::<SidePanelState>()
            .add_startup_system(configure_egui)
            .add_system(update_side_panel);
    }
}

fn configure_egui(mut _contexts: EguiContexts, mut egui_settings: ResMut<EguiSettings>) {
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
    pub panel_width: f32,
    pub inspector_width: f32,
}

impl Default for SidePanelState {
    fn default() -> Self {
        Self {
            mouse_over: false,
            mode: UiMode::Select,
            rapier_debug_enabled: false,
            panel_width: 0.0,
            inspector_width: 0.0,
        }
    }
}

fn update_side_panel(
    mut egui_ctx: EguiContexts,
    keyboard: Res<Input<KeyCode>>,
    diagnostics: Res<Diagnostics>,
    mut state: ResMut<SidePanelState>,
    mut sel_state: ResMut<SelectionUiState>,
    add_cube_state: ResMut<AddCubeUiState>,
    mut debug_render_ctx: ResMut<DebugRenderContext>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_selected: Query<
        (
            Entity,
            Option<&Name>,
            Option<&KiRevoluteJoint>,
            Option<&KiSphericalJoint>,
        ),
        With<Selected>,
    >,
    cmd: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        state.mode = UiMode::Select;
    }

    let selected: Vec<_> = q_selected.iter().collect();

    state.panel_width = egui::SidePanel::left("side_panel")
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

            selection_ui(ui, &mut sel_state, selected, cmd);

            egui::CollapsingHeader::new("Physics")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut state.rapier_debug_enabled, "Debug render");
                    debug_render_ctx.enabled = state.rapier_debug_enabled;

                    add_cube_ui(ui, &mut state, add_cube_state);
                });
        })
        .response
        .rect
        .width();

    state.mouse_over = true;
    if let Ok(window) = q_window.get_single() {
        if let Some(mouse_pos) = window.cursor_position() {
            state.mouse_over = mouse_pos.x <= state.panel_width;
            if !state.mouse_over && sel_state.show_inspector && !q_selected.is_empty() {
                state.mouse_over = mouse_pos.x >= window.width() - state.inspector_width;
            }
        }
    }
}

pub fn ui_mode_toggle(ui: &mut egui::Ui, state: &mut SidePanelState, mode: UiMode, text: &str) {
    let mut val = state.mode == mode;
    let val1 = val;
    ui.toggle_value(&mut val, text);
    if val {
        state.mode = mode;
    } else if val1 {
        state.mode = UiMode::Select
    };
}
