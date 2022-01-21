use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};
use big_brain::prelude::*;
use rand::{thread_rng, Rng};

use protos::{
    ai::{AiPlugin, Drunk, Idle, RandomMove, Velocity},
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
        .add_plugin(AiPlugin)
        .add_startup_system(setup)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // units
    let units = {
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
        let mut units = vec![];
        for x in (-500..500).step_by(10) {
            for z in (-500..500).step_by(10) {
                units.push(
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: mesh.clone(),
                            material: mats[rng.gen_range(0..mats.len())].clone(),
                            transform: Transform::from_xyz(x as f32 + 5., 1.5, z as f32 + 5.),
                            ..Default::default()
                        })
                        .insert(Name::new(format!("Agent[{},{}]", x / 10, z / 10)))
                        .insert_bundle(PickableBundle::default())
                        .insert(Velocity::default())
                        .insert(
                            Thinker::build()
                                .picker(FirstToScore { threshold: 0.8 })
                                .when(Drunk, RandomMove::new())
                                .otherwise(Idle),
                        )
                        .id(),
                );
            }
        }
        units
    };

    // ground
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -500.,
                max_x: 500.,
                min_y: -5.,
                max_y: 0.,
                min_z: -500.,
                max_z: 500.,
            })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert(Name::new("Ground"))
        .push_children(&units);
}
