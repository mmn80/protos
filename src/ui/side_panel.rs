use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiSettings};
use bevy_rapier3d::render::DebugRenderContext;

use crate::anim::rig::{KiRevoluteJoint, KiSphericalJoint};

use super::{
    add_cube::{add_cube_ui, AddCubeUiState},
    selection::{selection_ui, Selected, SelectionUiState, INSPECTOR_WIDTH},
};

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
    AddCube,
    ShootBalls,
}

#[derive(Resource)]
pub struct SidePanelState {
    pub mouse_over: bool,
    pub mode: UiMode,
    pub rapier_debug_enabled: bool,
}

impl Default for SidePanelState {
    fn default() -> Self {
        Self {
            mouse_over: false,
            mode: UiMode::Select,
            rapier_debug_enabled: false,
        }
    }
}

const SIDE_PANEL_WIDTH: f32 = 250.;

fn update_side_panel(
    mut egui_ctx: ResMut<EguiContext>,
    windows: Res<Windows>,
    keyboard: Res<Input<KeyCode>>,
    diagnostics: Res<Diagnostics>,
    mut state: ResMut<SidePanelState>,
    sel_state: ResMut<SelectionUiState>,
    add_cube_state: ResMut<AddCubeUiState>,
    mut debug_render_ctx: ResMut<DebugRenderContext>,
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

    state.mouse_over = true;
    if let Some(window) = windows.get_primary() {
        if let Some(mouse_pos) = window.cursor_position() {
            state.mouse_over = mouse_pos.x <= SIDE_PANEL_WIDTH;
            if !state.mouse_over && sel_state.show_inspector && !q_selected.is_empty() {
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

            selection_ui(ui, sel_state, selected, cmd);

            egui::CollapsingHeader::new("Physics")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut state.rapier_debug_enabled, "Debug render");
                    debug_render_ctx.enabled = state.rapier_debug_enabled;

                    add_cube_ui(ui, state, add_cube_state);
                });
        });
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
