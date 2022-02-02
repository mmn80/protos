use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiSettings};

use crate::ai::ground::GroundMaterials;

use super::multi_select::Selected;

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

fn configure_egui(egui_ctx: ResMut<EguiContext>, mut egui_settings: ResMut<EguiSettings>) {
    egui_ctx.ctx().set_visuals(egui::Visuals {
        window_corner_radius: 0.0,
        ..Default::default()
    });
    egui_settings.scale_factor = 1.0;
}

pub struct SidePanelState {
    pub random_walk_selected: bool,
    pub random_walk_all: bool,
    pub ground_brush_size: u8,
    pub ground_material: GroundMaterials,
    pub spawn_building: bool,
}

impl Default for SidePanelState {
    fn default() -> Self {
        Self {
            random_walk_selected: Default::default(),
            random_walk_all: Default::default(),
            ground_brush_size: 1,
            ground_material: Default::default(),
            spawn_building: false,
        }
    }
}

fn update_side_panel(
    egui_ctx: ResMut<EguiContext>,
    diagnostics: Res<Diagnostics>,
    mut state: ResMut<SidePanelState>,
    query: Query<(&Name, &Transform), With<Selected>>,
) {
    egui::SidePanel::left("side_panel")
        .default_width(200.0)
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
                    ui.checkbox(&mut state.random_walk_selected, "Random walk (selected)");
                    ui.checkbox(&mut state.random_walk_all, "Random walk (all)");

                    if !query.is_empty() {
                        ui.add_space(10.);
                        ui.colored_label(egui::Color32::DARK_GREEN, "Selected objects:");
                        for (name, transform) in query.iter() {
                            let pos = transform.translation;
                            ui.label(format!("- {}: {:.1},{:.1}", name.as_str(), pos.x, pos.z));
                        }
                    }
                });

            egui::CollapsingHeader::new("Ground painter")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut state.ground_brush_size, 1..=32));
                    ui.radio_value(&mut state.ground_material, GroundMaterials::None, "None");
                    ui.radio_value(&mut state.ground_material, GroundMaterials::Grass, "Grass");
                    ui.radio_value(&mut state.ground_material, GroundMaterials::Road, "Road");
                });

            egui::CollapsingHeader::new("Buildings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut state.spawn_building, "Spawn building");
                });
        });
}
