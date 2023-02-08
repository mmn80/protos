use std::{f32::consts::PI, time::Instant};

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_mod_raycast::{RaycastMesh, RaycastSource};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, LogNormal};

use super::{
    fast_unit_index::Neighbours,
    ground::{Ground, GroundRaycastSet},
    pathfind::Moving,
    velocity::Velocity,
};
use crate::{
    camera::ScreenPosition,
    ui::selection::{Selectable, Selected},
};

pub struct FastUnitPlugin;

impl Plugin for FastUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.after("ground_setup"))
            .add_system(move_to_command)
            .add_system(show_unit_debug_info.with_run_criteria(f1_just_pressed));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    ground: Res<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let units = {
        let mesh = meshes.add(Mesh::from(shape::Capsule {
            depth: 2.,
            ..default()
        }));
        let mut rng = thread_rng();
        let mats = {
            let mut mats = vec![];
            for _ in 1..10 {
                let mut material = StandardMaterial::from(Color::rgb(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                ));
                material.perceptual_roughness = rng.gen_range(0.089..1.0);
                material.metallic = rng.gen_range(0.0..1.0);
                // material.reflectance = rng.gen_range(0.0..1.0);
                // material.emissive = Color::rgb(
                //     rng.gen_range(0.0..0.2),
                //     rng.gen_range(0.0..0.2),
                //     rng.gen_range(0.0..0.2),
                // );
                mats.push(materials.add(material));
            }
            mats
        };
        let mut units = vec![];
        let area_dist = LogNormal::new(PI * 0.8 * 0.8, 0.4).unwrap();
        let mut rng = thread_rng();
        let mut agent_id = 0;
        for x in (10..ground.width() - 10).step_by(50) {
            for z in (10..ground.width() - 10).step_by(50) {
                agent_id += 1;
                let scale = f32::sqrt(area_dist.sample(&mut rng) / PI);
                units.push(
                    commands
                        .spawn((
                            PbrBundle {
                                mesh: mesh.clone(),
                                material: mats[rng.gen_range(0..mats.len())].clone(),
                                transform: Transform::from_xyz(x as f32 + 0.5, 1.5, z as f32 + 0.5)
                                    .with_scale(Vec3::new(scale, 1., scale)),
                                ..default()
                            },
                            Name::new(format!("Agent_{}", agent_id)),
                            ScreenPosition::default(),
                            Selectable,
                            Velocity::default(),
                            Neighbours::default(),
                        ))
                        .id(),
                );
            }
        }
        units
    };
    if let Some(ground_ent) = ground.entity {
        commands.entity(ground_ent).push_children(&units);
    } else {
        warn!("NO GROUND!!");
    }
}

fn move_to_command(
    keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
    ground: Res<Ground>,
    source_query: Query<&RaycastSource<GroundRaycastSet>>,
    target_query: Query<&Transform, With<RaycastMesh<GroundRaycastSet>>>,
    mut selected_q: Query<(Entity, &mut Velocity), (With<Selected>, Without<Moving>)>,
    mut cmd: Commands,
) {
    if keyboard.pressed(KeyCode::LControl)
        && input_mouse.just_pressed(MouseButton::Right)
        && !selected_q.is_empty()
    {
        if let Ok(ground_transform) = target_query.get_single() {
            let mat = ground_transform.compute_matrix().inverse();
            for source in &source_query {
                let intersections = source.intersections();
                if !intersections.is_empty() {
                    if intersections.len() > 1 {
                        info!("more then 1 intersection!");
                    }
                    for (gnd_entity, intersection) in intersections {
                        if *gnd_entity == ground.entity.unwrap() {
                            let target = mat.project_point3(intersection.position());
                            let target = ground.clamp(target, 10.);
                            if ground.get_tile_vec3(target).is_some() {
                                info!("move to: {target:?}");
                                for (entity, mut velocity) in &mut selected_q {
                                    cmd.entity(entity).insert(Moving {
                                        target,
                                        speed: 10.,
                                        start_time: Instant::now(),
                                    });
                                    velocity.breaking = false;
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

// debug

fn f1_just_pressed(keyboard: Res<Input<KeyCode>>) -> ShouldRun {
    if keyboard.just_pressed(KeyCode::F1) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn show_unit_debug_info(unit_query: Query<(Entity, &Neighbours), With<Selected>>) {
    let mut info = String::new();
    for (unit_ent, neighbours) in &unit_query {
        info.push_str(format!("unit: {:?}, ", unit_ent).as_str());
        info.push_str(
            format!(
                "neighbours (<{}m): {:?}\n",
                neighbours.range, neighbours.neighbours
            )
            .as_str(),
        );
        break;
    }
    info!("{info}");
}
