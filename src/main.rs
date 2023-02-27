use bevy::{app::AppExit, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_rapier3d::prelude::*;

use protos::{
    ai::{
        add_cube::AddCubePlugin, building::BuildingPlugin, joints::JointsPlugin,
        kinematic_rig::KinematicRigPlugin, terrain::TerrainPlugin,
    },
    camera::MainCameraPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
    mesh::lines::LinesPlugin,
    ui::{
        basic_materials::BasicMaterialsPlugin, handle_gizmo::HandleGizmoPlugin,
        move_gizmo::MoveGizmoPlugin, selection::SelectionPlugin, side_panel::SidePanelPlugin,
    },
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(INFINITE_TEMP_COLOR))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Prototype".to_string(),
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
        .add_plugin(DefaultInspectorConfigPlugin)
        .add_plugin(SidePanelPlugin)
        .add_plugin(SelectionPlugin)
        .add_plugin(BasicMaterialsPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(HandleGizmoPlugin)
        .add_plugin(MoveGizmoPlugin)
        .add_plugin(MainLightsPlugin)
        .add_plugin(MainCameraPlugin)
        .add_plugin(KinematicRigPlugin)
        .add_plugin(JointsPlugin)
        .add_plugin(TerrainPlugin)
        .add_plugin(AddCubePlugin)
        .add_plugin(BuildingPlugin)
        .add_system(exit_system)
        .run();
}

fn exit_system(keyboard: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard.just_released(KeyCode::Q) && keyboard.pressed(KeyCode::LControl) {
        exit.send(AppExit);
    }
}
