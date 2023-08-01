use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::ui::basic_materials::BasicMaterials;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Terrain>()
            .init_resource::<Terrain>()
            .add_systems(Startup, setup_terrain)
            .add_systems(Update, (draw_terrain_lines, display_rapier_events));
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Terrain {
    pub ground: Option<Entity>,
}

fn setup_terrain(
    mut terrain: ResMut<Terrain>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<BasicMaterials>,
    mut cmd: Commands,
) {
    let ground_size = Vec3::new(200.0, 1.0, 200.0);

    terrain.ground = Some({
        let id = cmd
            .spawn((
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
            .id();
        cmd.entity(id)
            .insert(Name::new(format!("Terrain ({id:?})")));
        id
    });
}

fn draw_terrain_lines(mut gizmos: Gizmos) {
    let col = Color::WHITE;
    let ground_size = Vec3::new(200.0, 1.0, 200.0);
    let h = 0.5;

    gizmos.line(
        Vec3::new(-ground_size.x / 2., h, 0.),
        Vec3::new(ground_size.x / 2., h, 0.),
        col,
    );
    gizmos.line(
        Vec3::new(0., h, -ground_size.z / 2.),
        Vec3::new(0., h, ground_size.z / 2.),
        col,
    );
    gizmos.line(Vec3::new(0., h, 1.), Vec3::new(1., h, 0.), col);

    let half_x = ground_size.x as i32 / 2 - 10;
    for x in (-half_x..half_x + 1).step_by(10) {
        gizmos.line(
            Vec3::new(x as f32, h, -0.5),
            Vec3::new(x as f32, h, 0.5),
            col,
        );
    }

    let half_z = ground_size.z as i32 / 2 - 10;
    for z in (-half_z..half_z + 1).step_by(10) {
        gizmos.line(
            Vec3::new(-0.5, h, z as f32),
            Vec3::new(0.5, h, z as f32),
            col,
        );
    }
}

fn display_rapier_events(
    mut collision_ev: EventReader<CollisionEvent>,
    mut contact_force_ev: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_ev.iter() {
        info!("Collision: {:?}", collision_event);
    }

    for contact_force_event in contact_force_ev.iter() {
        info!("Contact force: {:?}", contact_force_event);
    }
}
