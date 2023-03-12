use bevy::{app::AppExit, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_rapier3d::prelude::*;

use protos::{
    ai::{building::BuildingPlugin, terrain::TerrainPlugin},
    anim::{auto_collider::AutoColliderPlugin, fox::FoxPlugin, joint::JointPlugin, rig::RigPlugin},
    camera::MainCameraPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    mesh::lines::LinesPlugin,
    ui::{
        add_cube::AddCubePlugin, basic_materials::BasicMaterialsPlugin, selection::SelectionPlugin,
        side_panel::SidePanelPlugin, transform_gizmo::TransformGizmoPlugin,
    },
};

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(INFINITE_TEMP_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Prototype".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            enabled: false,
            ..Default::default()
        })
        .add_plugin(EguiPlugin)
        .add_plugin(DefaultInspectorConfigPlugin)
        .add_plugin(SidePanelPlugin)
        .add_plugin(SelectionPlugin)
        .add_plugin(BasicMaterialsPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(TransformGizmoPlugin)
        .add_plugin(MainLightsPlugin)
        .add_plugin(MainCameraPlugin)
        .add_plugin(RigPlugin)
        .add_plugin(JointPlugin)
        .add_plugin(AutoColliderPlugin)
        .add_plugin(TerrainPlugin)
        .add_plugin(AddCubePlugin)
        .add_plugin(FoxPlugin)
        .add_plugin(BuildingPlugin)
        .add_system(exit_system)
        .run();
}

fn exit_system(keyboard: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard.just_released(KeyCode::Q) && keyboard.pressed(KeyCode::LControl) {
        exit.send(AppExit);
    }
}
