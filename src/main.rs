use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use big_brain::prelude::*;

use protos::{
    ai::{
        fast_unit::FastUnitPlugin, fast_unit_index::FastUnitIndexPlugin, ground::GroundPlugin,
        slow_unit::SlowUnitPlugin,
    },
    camera::MainCameraPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    ui::{multi_select::MultiSelectPlugin, side_panel::SidePanelPlugin},
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
        .add_plugin(MultiSelectPlugin)
        .add_plugin(MainLightsPlugin::default())
        .add_plugin(MainCameraPlugin)
        .add_plugin(GroundPlugin)
        .add_plugin(SlowUnitPlugin)
        .add_plugin(FastUnitIndexPlugin)
        .add_plugin(FastUnitPlugin)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
