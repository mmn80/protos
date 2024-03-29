use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContext, EguiContexts, EguiSettings};
use bevy_xpbd_3d::prelude::PhysicsDebugConfig;

use crate::{
    ai::swarm::InitSwarmEvent,
    anim::rig::{KiRevoluteJoint, KiSphericalJoint},
};

use super::selection::{selection_ui, Selected, SelectionUiState};

pub struct SidePanelPlugin;

impl Plugin for SidePanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SidePanel>()
            .init_resource::<SidePanel>()
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            //.add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
            //.add_plugins(wgpu::WgpuResourceDiagnosticsPlugin::default())
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
            //.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin::default())
            .init_resource::<SidePanel>()
            .add_systems(Startup, configure_egui)
            .add_systems(Update, (main_panel, inspector_panel));
    }
}

fn configure_egui(mut _contexts: EguiContexts, mut egui_settings: ResMut<EguiSettings>) {
    egui_settings.scale_factor = 1.0;
}

#[derive(PartialEq, Reflect)]
pub enum UiMode {
    Select,
    AddCube,
    ShootBalls,
    AddFox,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct SidePanel {
    pub mouse_over: bool,
    pub show_resources: bool,
    pub show_assets: bool,
    pub show_world: bool,
    pub mode: UiMode,
    pub physics_debug_enabled: bool,
    pub panel_width: f32,
    pub inspector_width: f32,
}

impl Default for SidePanel {
    fn default() -> Self {
        Self {
            mouse_over: false,
            show_resources: false,
            show_assets: false,
            show_world: true,
            mode: UiMode::Select,
            physics_debug_enabled: false,
            panel_width: 0.0,
            inspector_width: 0.0,
        }
    }
}

fn main_panel(
    mut egui_ctx: EguiContexts,
    keyboard: Res<Input<KeyCode>>,
    diagnostics: Res<DiagnosticsStore>,
    mut panel: ResMut<SidePanel>,
    mut sel_state: ResMut<SelectionUiState>,
    mut physics_debug_config: ResMut<PhysicsDebugConfig>,
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
    mut ev_init_swarm: EventWriter<InitSwarmEvent>,
    cmd: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        panel.mode = UiMode::Select;
    }

    let selected: Vec<_> = q_selected.iter().collect();

    panel.panel_width = egui::SidePanel::left("side_panel")
        .show(egui_ctx.ctx_mut(), |ui| {
            let fps = diagnostics
                .get(FrameTimeDiagnosticsPlugin::FPS)
                .map(|d| d.smoothed().unwrap_or(0.) as u32)
                .unwrap_or(0);
            let entities = diagnostics
                .get_measurement(bevy::diagnostic::EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .map(|d| d.value as u32)
                .unwrap_or(0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("FPS: {fps}"));
                    ui.separator();
                    ui.label(format!("entity: {entities}"));
                });
            });

            egui::CollapsingHeader::new("Global settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut panel.show_resources, "Show resources");
                    ui.checkbox(&mut panel.show_assets, "Show assets");
                    ui.checkbox(&mut panel.show_world, "Show world");
                });

            selection_ui(ui, &mut sel_state, selected, cmd);

            egui::CollapsingHeader::new("Physics")
                .default_open(true)
                .show(ui, |ui| {
                    ui.checkbox(&mut panel.physics_debug_enabled, "Debug render");
                    physics_debug_config.enabled = panel.physics_debug_enabled;

                    if ui.button("Toggle swarm").clicked() {
                        ev_init_swarm.send(InitSwarmEvent);
                    }
                });

            egui::CollapsingHeader::new("World")
                .default_open(true)
                .show(ui, |ui| {
                    ui_mode_toggle(ui, &mut panel, UiMode::ShootBalls, "Shoot balls");
                    ui_mode_toggle(ui, &mut panel, UiMode::AddFox, "Add fox");
                });
        })
        .response
        .rect
        .width();

    panel.mouse_over = true;
    if let Ok(window) = q_window.get_single() {
        if let Some(mouse_pos) = window.cursor_position() {
            panel.mouse_over = mouse_pos.x <= panel.panel_width
                || mouse_pos.x >= window.width() - panel.inspector_width;
        }
    }
}

pub fn ui_mode_toggle(ui: &mut egui::Ui, panel: &mut SidePanel, mode: UiMode, text: &str) {
    let mut val = panel.mode == mode;
    let val1 = val;
    ui.toggle_value(&mut val, text);
    if val {
        panel.mode = mode;
    } else if val1 {
        panel.mode = UiMode::Select
    };
}

fn inspector_panel(world: &mut World) {
    let (show_resources, show_assets, show_world) = {
        let panel = world.resource_mut::<SidePanel>();
        (panel.show_resources, panel.show_assets, panel.show_world)
    };

    let selected = {
        if world.resource::<SelectionUiState>().show_inspector {
            let mut q_selected = world.query_filtered::<Entity, With<Selected>>();
            q_selected.iter(&world).next()
        } else {
            None
        }
    };

    if selected.is_none() && !show_resources && !show_assets && !show_world {
        world.resource_mut::<SidePanel>().inspector_width = 0.;
        return;
    }

    let mut egui_ctx = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    world.resource_mut::<SidePanel>().inspector_width = egui::SidePanel::right("inspector")
        .show(egui_ctx.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if show_resources {
                    egui::CollapsingHeader::new("Resources")
                        .default_open(true)
                        .show(ui, |ui| {
                            bevy_inspector_egui::bevy_inspector::ui_for_resources(world, ui);
                        });
                }

                if show_assets {
                    egui::CollapsingHeader::new("Assets")
                        .default_open(true)
                        .show(ui, |ui| {
                            bevy_inspector_egui::bevy_inspector::ui_for_all_assets(world, ui);
                        });
                }

                if show_world {
                    egui::CollapsingHeader::new("World")
                        .default_open(true)
                        .show(ui, |ui| {
                            bevy_inspector_egui::bevy_inspector::ui_for_world_entities(world, ui);
                        });
                }

                if let Some(entity) = selected {
                    egui::CollapsingHeader::new("Selected")
                        .default_open(true)
                        .show(ui, |ui| {
                            bevy_inspector_egui::bevy_inspector::ui_for_entity_with_children(
                                world, entity, ui,
                            );
                        });
                }
            });
        })
        .response
        .rect
        .width();
}
