use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContext, EguiContexts, EguiSettings};
use bevy_rapier3d::render::DebugRenderContext;

use crate::anim::rig::{KiRevoluteJoint, KiSphericalJoint};

use super::{
    add_cube::{add_cube_ui, AddCubeUiState},
    selection::{selection_ui, Selected, SelectionUiState},
};

pub struct SidePanelPlugin;

impl Plugin for SidePanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SidePanel>()
            .init_resource::<SidePanel>()
            .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            //.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
            //.add_plugin(bevy::wgpu::diagnostic::WgpuResourceDiagnosticsPlugin::default())
            //.add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
            //.add_plugin(bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<Mesh>::default())
            .init_resource::<SidePanel>()
            .add_startup_system(configure_egui)
            .add_systems((main_panel, inspector_panel));
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
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct SidePanel {
    pub mouse_over: bool,
    pub show_resources: bool,
    pub show_assets: bool,
    pub show_world: bool,
    pub mode: UiMode,
    pub rapier_debug_enabled: bool,
    pub panel_width: f32,
    pub inspector_width: f32,
}

impl Default for SidePanel {
    fn default() -> Self {
        Self {
            mouse_over: false,
            show_resources: false,
            show_assets: false,
            show_world: false,
            mode: UiMode::Select,
            rapier_debug_enabled: false,
            panel_width: 0.0,
            inspector_width: 0.0,
        }
    }
}

fn main_panel(
    mut egui_ctx: EguiContexts,
    keyboard: Res<Input<KeyCode>>,
    diagnostics: Res<Diagnostics>,
    mut panel: ResMut<SidePanel>,
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
        panel.mode = UiMode::Select;
    }

    let selected: Vec<_> = q_selected.iter().collect();

    panel.panel_width = egui::SidePanel::left("side_panel")
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
                    ui.checkbox(&mut panel.rapier_debug_enabled, "Debug render");
                    debug_render_ctx.enabled = panel.rapier_debug_enabled;

                    add_cube_ui(ui, &mut panel, add_cube_state);
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
