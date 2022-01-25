use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use big_brain::prelude::*;

use protos::{
    camera::MainCameraPlugin,
    fast_unit::FastUnitPlugin,
    fast_unit_index::FastUnitIndexPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    slow_unit::SlowUnitPlugin,
    ui::SidePanelPlugin,
};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Prototypes".to_string(),
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(INFINITE_TEMP_COLOR))
        .add_plugins(DefaultPlugins)
        .add_plugin(DefaultPickingPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(BigBrainPlugin)
        .add_plugin(SidePanelPlugin)
        .add_plugin(MainLightsPlugin::default())
        .add_plugin(MainCameraPlugin)
        .add_plugin(SlowUnitPlugin)
        .add_plugin(FastUnitIndexPlugin)
        .add_plugin(FastUnitPlugin)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
