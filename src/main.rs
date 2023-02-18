use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::prelude::*;

use protos::{
    ai::{platform::PlatformPlugin, terrain::TerrainPlugin},
    camera::MainCameraPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    ui::{lines::LinesPlugin, selection::SelectionPlugin, side_panel::SidePanelPlugin},
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(INFINITE_TEMP_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Prototypes".to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            enabled: false,
            ..Default::default()
        })
        .add_plugin(EguiPlugin)
        .add_plugin(SidePanelPlugin)
        .add_plugin(SelectionPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(MainLightsPlugin::default())
        .add_plugin(MainCameraPlugin)
        .add_plugin(TerrainPlugin)
        .add_plugin(PlatformPlugin)
        .add_system(bevy::window::close_on_esc)
        .run();
}
