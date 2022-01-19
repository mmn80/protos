use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiSettings};
use bevy_mod_picking::Selection;

pub struct SidePanelPlugin;

impl Plugin for SidePanelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiState::default())
            .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            //.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
            //.add_plugin(bevy::wgpu::diagnostic::WgpuResourceDiagnosticsPlugin::default())
            //.add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
            //.add_plugin(bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<Mesh>::default())
            .insert_resource(UiState::default())
            .add_startup_system(configure_egui)
            .add_system(update_side_panel);
    }
}

fn configure_egui(egui_ctx: ResMut<EguiContext>, mut egui_settings: ResMut<EguiSettings>) {
    egui_ctx.ctx().set_visuals(egui::Visuals {
        window_corner_radius: 0.0,
        ..Default::default()
    });
    egui_settings.scale_factor = 1.0;
}

#[derive(Default)]
pub struct UiState {
    pub random_walk_selected: bool,
    pub random_walk_all: bool,
}

fn update_side_panel(
    egui_ctx: ResMut<EguiContext>,
    diagnostics: Res<Diagnostics>,
    mut state: ResMut<UiState>,
    query: Query<(Entity, &Selection, &Transform)>,
) {
    egui::SidePanel::left("side_panel")
        .default_width(250.0)
        .show(egui_ctx.ctx(), |ui| {
            let fps = diagnostics
                .get_measurement(FrameTimeDiagnosticsPlugin::FPS)
                .map(|d| d.value.round() as u32)
                .unwrap_or(0);
            let frame = diagnostics
                .get_measurement(FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                .map(|d| d.value as u32)
                .unwrap_or(0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.label(format!("Frame: {}", frame));
                    ui.separator();
                    ui.label(format!("FPS: {}", fps));
                });
            });

            egui::CollapsingHeader::new("Selection")
                .default_open(true)
                .show(ui, |ui| {
                    ui.colored_label(egui::Color32::DARK_GREEN, "Selected objects:");
                    for (entity, selection, transform) in query.iter() {
                        if selection.selected() {
                            ui.label(format!("- {:?}: {}", entity, transform.translation));
                        }
                    }
                    ui.add_space(10.);
                    ui.checkbox(&mut state.random_walk_selected, "Random walk (selected)");
                    ui.checkbox(&mut state.random_walk_all, "Random walk (all)");
                });
        });
}
