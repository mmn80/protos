use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext, EguiSettings};

use super::selection::Selected;
use crate::ai::{
    fast_unit::{Awake, Sleeping},
    ground::GroundMaterials,
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

fn configure_egui(egui_ctx: ResMut<EguiContext>, mut egui_settings: ResMut<EguiSettings>) {
    egui_ctx.ctx().set_visuals(egui::Visuals {
        window_corner_radius: 0.0,
        ..Default::default()
    });
    egui_settings.scale_factor = 1.0;
}

pub struct SidePanelState {
    pub ai_active_selected: bool,
    pub ai_active_all: bool,
    pub selected_show_names: bool,
    pub selected_show_path: bool,
    pub ground_brush_size: u8,
    pub ground_material: GroundMaterials,
    pub spawn_building: bool,
}

impl Default for SidePanelState {
    fn default() -> Self {
        Self {
            ai_active_selected: false,
            ai_active_all: false,
            selected_show_names: true,
            selected_show_path: true,
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
    selected_q: Query<(&Name, Option<&Awake>, Option<&Sleeping>), With<Selected>>,
    awake_q: Query<With<Awake>>,
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
                    ui.checkbox(&mut state.ai_active_selected, "Ai active (selected)");
                    ui.checkbox(&mut state.ai_active_all, "Ai active (all)");
                    ui.checkbox(&mut state.selected_show_names, "Show names (selected)");
                    ui.checkbox(&mut state.selected_show_path, "Show paths (selected)");

                    ui.colored_label(
                        egui::Color32::DARK_GREEN,
                        format!("{} objects awake", awake_q.iter().count()),
                    );

                    let selected: Vec<_> = selected_q.iter().collect();
                    if !selected.is_empty() {
                        ui.add_space(10.);
                        ui.colored_label(
                            egui::Color32::DARK_GREEN,
                            format!("{} objects selected:", selected.len()),
                        );
                        for (name, awake, sleeping) in selected.iter().take(20) {
                            let status = {
                                if let Some(awake) = awake {
                                    format!(
                                        "A: {}/{}s",
                                        (std::time::Instant::now() - awake.since).as_secs(),
                                        awake.duration as u64
                                    )
                                } else if let Some(sleeping) = sleeping {
                                    format!(
                                        "S: {}/{}s",
                                        (std::time::Instant::now() - sleeping.since).as_secs(),
                                        sleeping.duration as u64
                                    )
                                } else {
                                    format!("?")
                                }
                            };
                            ui.label(format!("- {}: {}", name.as_str(), status));
                        }
                        if selected.len() > 20 {
                            ui.label("...");
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
