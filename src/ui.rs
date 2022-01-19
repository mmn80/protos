use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiSettings};

pub struct SidePanelPlugin;

impl Plugin for SidePanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(configure_egui)
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

fn update_side_panel(egui_ctx: ResMut<EguiContext>, diagnostics: Res<Diagnostics>) {
    egui::SidePanel::left("side_panel")
        .default_width(250.0)
        .show(egui_ctx.ctx(), |ui| {
            let fps = diagnostics
                .get_measurement(FrameTimeDiagnosticsPlugin::FPS)
                .map(|d| d.value)
                .unwrap_or(0.);
            let frame = diagnostics
                .get_measurement(FrameTimeDiagnosticsPlugin::FRAME_COUNT)
                .map(|d| d.value)
                .unwrap_or(0.);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.label(format!("Frame: {}", frame));
                    ui.separator();
                    ui.label(format!("FPS: {:.1}", fps));
                });
            });
        });
}
