use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use big_brain::prelude::*;

use protos::{
    ai::{
        fast_unit::FastUnitPlugin, fast_unit_index::FastUnitIndexPlugin, ground::GroundPlugin,
        pathfind::PathfindingPlugin, slow_unit::SlowUnitPlugin, velocity::VelocityPlugin,
    },
    camera::MainCameraPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    ui::{selection::SelectionPlugin, side_panel::SidePanelPlugin},
};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Prototypes".to_string(),
            ..default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(INFINITE_TEMP_COLOR))
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(BigBrainPlugin)
        .add_plugin(SidePanelPlugin)
        .add_plugin(SelectionPlugin)
        .add_plugin(MainLightsPlugin::default())
        .add_plugin(MainCameraPlugin)
        .add_plugin(GroundPlugin)
        .add_plugin(SlowUnitPlugin)
        .add_plugin(FastUnitIndexPlugin)
        .add_plugin(VelocityPlugin)
        .add_plugin(PathfindingPlugin)
        .add_plugin(FastUnitPlugin)
        .add_system(bevy::window::close_on_esc)
        .run();
}
