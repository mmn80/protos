use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use bevy_rapier3d::prelude::*;

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
) {
    let ground_size = Vec3::new(100.0, 1.0, 100.0);
    let nsubdivs = 100;

    let mut heights = Vec::new();
    for i in 0..nsubdivs + 1 {
        for j in 0..nsubdivs + 1 {
            if i == 0 || i == nsubdivs || j == 0 || j == nsubdivs {
                heights.push(0.0);
            } else {
                let x = i as f32 * ground_size.x / (nsubdivs as f32);
                let z = j as f32 * ground_size.z / (nsubdivs as f32);
                heights.push(x.sin() + z.cos());
            }
        }
    }
    let height_collider = Collider::heightfield(heights, nsubdivs + 1, nsubdivs + 1, ground_size);
    let height_field = height_collider.as_heightfield().unwrap();
    let height_mesh = {
        let tri_mesh = height_field.raw.to_trimesh();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            tri_mesh
                .0
                .iter()
                .map(|p| [p.x, p.y, p.z])
                .collect::<Vec<[f32; 3]>>(),
        );
        //mesh.set_indices(Some(Indices::U32(tri_mesh.1.concat().into())));
        mesh.compute_flat_normals();
        mesh
    };

    commands
        .spawn(PbrBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            mesh: meshes.add(height_mesh),
            material: materials.add(Color::SILVER.into()),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(height_collider);
}
