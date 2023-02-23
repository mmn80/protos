use bevy::{pbr::NotShadowCaster, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{
    mesh::lines::{LineList, LineMaterial},
    ui::basic_materials::BasicMaterialsRes,
};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainRes::default())
            .add_startup_system(setup_terrain)
            .add_system(display_events);
    }
}

#[derive(Resource, Default)]
pub struct TerrainRes {
    pub ground: Option<Entity>,
}

fn setup_terrain(
    mut terrain: ResMut<TerrainRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
    materials: Res<BasicMaterialsRes>,
    mut cmd: Commands,
) {
    let ground_size = Vec3::new(200.0, 1.0, 200.0);

    terrain.ground = Some(
        cmd.spawn((
            PbrBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                mesh: meshes.add(Mesh::from(shape::Box::new(
                    ground_size.x,
                    ground_size.y,
                    ground_size.z,
                ))),
                material: materials.terrain.clone(),
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(ground_size.x / 2., ground_size.y / 2., ground_size.z / 2.),
        ))
        .id(),
    );

    let mut lines = vec![
        (
            Vec3::new(-ground_size.x / 2., 0., 0.),
            Vec3::new(ground_size.x / 2., 0., 0.),
        ),
        (
            Vec3::new(0., 0., -ground_size.z / 2.),
            Vec3::new(0., 0., ground_size.z / 2.),
        ),
        (Vec3::new(0., 0., 1.), Vec3::new(1., 0., 0.)),
    ];

    let half_x = ground_size.x as i32 / 2 - 10;
    for x in (-half_x..half_x + 1).step_by(10) {
        lines.push((Vec3::new(x as f32, 0., -0.5), Vec3::new(x as f32, 0., 0.5)));
    }

    let half_z = ground_size.z as i32 / 2 - 10;
    for z in (-half_z..half_z + 1).step_by(10) {
        lines.push((Vec3::new(-0.5, 0., z as f32), Vec3::new(0.5, 0., z as f32)));
    }

    cmd.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(LineList { lines })),
            transform: Transform::from_xyz(0., ground_size.y / 2. + 0.01, 0.),
            material: line_materials.add(LineMaterial {
                color: Color::WHITE,
            }),
            ..default()
        },
        NotShadowCaster,
    ));
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.iter() {
        println!("Received collision event: {:?}", collision_event);
    }

    for contact_force_event in contact_force_events.iter() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}
