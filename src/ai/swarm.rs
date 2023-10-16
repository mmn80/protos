use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

use crate::ui::basic_materials::BasicMaterials;

pub struct SwarmPlugin;

impl Plugin for SwarmPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InitSwarmEvent>()
            .add_systems(Update, init_swarm);
    }
}

#[derive(Event)]
pub struct InitSwarmEvent;

#[derive(Component)]
pub struct SwarmNPC;

const NPCW: i32 = 100;

fn init_swarm(
    materials: Res<BasicMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ev_init_swarm: EventReader<InitSwarmEvent>,
    q_swarm_npcs: Query<Entity, With<SwarmNPC>>,
    mut cmd: Commands,
) {
    if !ev_init_swarm.is_empty() {
        if q_swarm_npcs.is_empty() {
            let parent_ent = cmd
                .spawn((SpatialBundle::default(), Name::new("Swarm")))
                .id();
            let cube = meshes.add(Mesh::try_from(shape::Cube::new(0.1)).unwrap());
            for x in -NPCW..NPCW {
                for z in -NPCW..NPCW {
                    let cube_ent = cmd
                        .spawn((
                            PbrBundle {
                                transform: Transform::from_translation(Vec3::new(
                                    x as f32, 0.6, z as f32,
                                )),
                                mesh: cube.clone(),
                                material: materials.gold.clone(),
                                ..default()
                            },
                            NotShadowCaster,
                            NotShadowReceiver,
                            SwarmNPC,
                        ))
                        .id();
                    cmd.entity(parent_ent).add_child(cube_ent);
                }
            }
        } else {
            for ent in &q_swarm_npcs {
                cmd.entity(ent).despawn_recursive();
            }
        }
    }
    ev_init_swarm.clear();
}
