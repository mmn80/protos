use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::ui::lines::{LineList, LineMaterial};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_terrain);
    }
}

fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    let ground_size = Vec3::new(100.0, 1.0, 100.0);
    let ground_mat = StandardMaterial {
        base_color: Color::SILVER,
        metallic: 0.2,
        perceptual_roughness: 0.8,
        reflectance: 0.2,
        ..default()
    };

    commands
        .spawn(PbrBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            mesh: meshes.add(Mesh::from(shape::Box::new(
                ground_size.x,
                ground_size.y,
                ground_size.z,
            ))),
            material: materials.add(ground_mat),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(
            ground_size.x / 2.,
            ground_size.y / 2.,
            ground_size.z / 2.,
        ));

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(
                Vec3::new(-ground_size.x / 2., 0., 0.),
                Vec3::new(ground_size.x / 2., 0., 0.),
            )],
        })),
        transform: Transform::from_xyz(0., ground_size.y / 2. + 0.1, 0.),
        material: line_materials.add(LineMaterial {
            color: Color::GREEN,
        }),
        ..default()
    });

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(
                Vec3::new(0., 0., -ground_size.z / 2.),
                Vec3::new(0., 0., ground_size.z / 2.),
            )],
        })),
        transform: Transform::from_xyz(0., ground_size.y / 2. + 0.1, 0.),
        material: line_materials.add(LineMaterial {
            color: Color::WHITE,
        }),
        ..default()
    });
}
