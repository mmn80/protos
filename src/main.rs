use bevy::prelude::*;
use rand::{thread_rng, Rng};

use protos::{camera::MainCameraPlugin, light::MainLightsPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb_u8(148, 177, 255)))
        .add_plugins(DefaultPlugins)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(MainLightsPlugin::default())
        .add_plugin(MainCameraPlugin::default())
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
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: mats[rng.gen_range(0..mats.len())].clone(),
                transform: Transform::from_xyz(x as f32, 1.5, z as f32),
                ..Default::default()
            });
        }
    }
}
