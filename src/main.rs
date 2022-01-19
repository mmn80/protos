use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};
use rand::{thread_rng, Rng};

use protos::{
    camera::MainCameraPlugin,
    light::{MainLightsPlugin, INFINITE_TEMP_COLOR},
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
        .add_plugin(SidePanelPlugin)
        .add_plugin(MainLightsPlugin::default())
        .add_plugin(MainCameraPlugin)
        .add_startup_system(setup)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1024.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // objects
    let mesh = meshes.add(Mesh::from(shape::Capsule {
        depth: 2.,
        ..Default::default()
    }));
    let mut rng = thread_rng();
    let mats = {
        let mut mats = vec![];
        for _ in 1..10 {
            mats.push(
                materials.add(
                    Color::rgb(
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                    )
                    .into(),
                ),
            );
        }
        mats
    };
    for x in (-500..500).step_by(10) {
        for z in (-500..500).step_by(10) {
            commands
                .spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: mats[rng.gen_range(0..mats.len())].clone(),
                    transform: Transform::from_xyz(x as f32, 1.5, z as f32),
                    ..Default::default()
                })
                .insert_bundle(PickableBundle::default());
        }
    }
}
