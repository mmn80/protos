use bevy::{
    app::AppExit,
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_xpbd_3d::prelude::*;

use protos::{
    ai::{building::BuildingPlugin, swarm::SwarmPlugin, terrain::TerrainPlugin},
    anim::{fox::FoxPlugin, joint::JointPlugin, rig::RigPlugin},
    camera::MainCameraPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    ui::{
        basic_materials::BasicMaterialsPlugin, selection::SelectionPlugin,
        side_panel::SidePanelPlugin, transform_gizmo::TransformGizmoPlugin,
    },
};

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(INFINITE_TEMP_COLOR))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Prototype".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..Default::default()
                    }),
                }),
        )
        .add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            EguiPlugin,
            DefaultInspectorConfigPlugin,
        ))
        .add_plugins((
            SidePanelPlugin,
            SelectionPlugin,
            BasicMaterialsPlugin,
            TransformGizmoPlugin,
            MainLightsPlugin,
            MainCameraPlugin,
            RigPlugin,
            JointPlugin,
            TerrainPlugin,
            FoxPlugin,
            BuildingPlugin,
            SwarmPlugin,
        ))
        .add_systems(Update, exit_system)
        .run();
}

fn exit_system(keyboard: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard.just_released(KeyCode::Q) && keyboard.pressed(KeyCode::ControlLeft) {
        exit.send(AppExit);
    }
}
