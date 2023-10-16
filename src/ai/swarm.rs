use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use rand::prelude::*;

use crate::ui::basic_materials::BasicMaterials;

pub struct SwarmPlugin;

impl Plugin for SwarmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SwarmStats>()
            .add_event::<InitSwarmEvent>()
            .add_systems(Update, (init_swarm, move_swarm));
    }
}

#[derive(Event)]
pub struct InitSwarmEvent;

#[derive(Component, Default)]
pub struct SwarmNPC {
    pub speed: f32,
}

#[derive(Resource, Default)]
pub struct SwarmStats {
    pub hits: u32,
    pub last_elapsed_sec: f32,
}

const SPAWN_MAX: f32 = 100.;
const NPC_NUM: u32 = 10000;
const HEIGHT: f32 = 0.6;
const FORCE: f32 = 10.;

fn init_swarm(
    materials: Res<BasicMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ev_init_swarm: EventReader<InitSwarmEvent>,
    q_swarm_npcs: Query<Entity, With<SwarmNPC>>,
    mut cmd: Commands,
) {
    if !ev_init_swarm.is_empty() {
        if q_swarm_npcs.is_empty() {
            let mut rng = rand::thread_rng();
            let parent_ent = cmd
                .spawn((SpatialBundle::default(), Name::new("Swarm")))
                .id();
            let cube = meshes.add(Mesh::try_from(shape::Cube::new(0.1)).unwrap());
            for npc in 0..NPC_NUM {
                let x = rng.gen_range(-SPAWN_MAX..SPAWN_MAX);
                let z = rng.gen_range(-SPAWN_MAX..SPAWN_MAX);
                let cube_ent = cmd
                    .spawn((
                        PbrBundle {
                            transform: Transform::from_translation(Vec3::new(x, HEIGHT, z)),
                            mesh: cube.clone(),
                            material: materials.salmon.clone(),
                            ..default()
                        },
                        Name::new(format!("NPC {npc}")),
                        NotShadowCaster,
                        NotShadowReceiver,
                        SwarmNPC::default(),
                    ))
                    .id();
                cmd.entity(parent_ent).add_child(cube_ent);
            }
        } else {
            for ent in &q_swarm_npcs {
                cmd.entity(ent).despawn_recursive();
            }
        }
    }
    ev_init_swarm.clear();
}

fn move_swarm(
    time: Res<Time>,
    mut swarm_stats: ResMut<SwarmStats>,
    mut q_swarm_npcs: Query<(Entity, &mut Transform, &mut SwarmNPC)>,
    mut cmd: Commands,
) {
    for (ent, mut tr, mut npc) in &mut q_swarm_npcs {
        let dist = tr.translation.length();
        let force = FORCE / dist.powi(2);
        npc.speed += force * time.delta_seconds();
        if npc.speed > dist - 1. || dist < 1. {
            cmd.entity(ent).despawn_recursive();
            swarm_stats.hits += 1;
        } else {
            let mut dir = -tr.translation.normalize();
            dir.y = 0.;
            tr.translation += npc.speed * dir;
        }
    }
    if swarm_stats.hits > 0 && time.elapsed_seconds() - swarm_stats.last_elapsed_sec >= 1. {
        info!("Hits: {}", swarm_stats.hits);
        swarm_stats.hits = 0;
        swarm_stats.last_elapsed_sec = time.elapsed_seconds();
    }
}
