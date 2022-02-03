use std::{f32::consts::PI, time::Instant};

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use big_brain::{prelude::*, thinker::HasThinker};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, LogNormal};

use super::{
    fast_unit_index::Neighbours,
    ground::Ground,
    pathfind::{clear_path_components, MoveTo},
    velocity::{Velocity, MAX_SPEED},
};
use crate::{
    camera::ScreenPosition,
    ui::{
        selection::{Selectable, Selected},
        side_panel::SidePanelState,
    },
};

pub struct FastUnitPlugin;

impl Plugin for FastUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.after("ground_setup"))
            .add_system_to_stage(BigBrainStage::Actions, idle_action)
            .add_system_to_stage(BigBrainStage::Actions, random_move_action)
            .add_system_to_stage(BigBrainStage::Scorers, drunk_scorer)
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
            ..Default::default()
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
        for x in (10..ground.width() - 10).step_by(10) {
            for z in (10..ground.width() - 10).step_by(10) {
                agent_id += 1;
                let scale = f32::sqrt(area_dist.sample(&mut rng) / PI);
                units.push(
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: mesh.clone(),
                            material: mats[rng.gen_range(0..mats.len())].clone(),
                            transform: Transform::from_xyz(x as f32 + 0.5, 1.5, z as f32 + 0.5)
                                .with_scale(Vec3::new(scale, 1., scale)),
                            ..Default::default()
                        })
                        .insert(Name::new(format!("Agent_{}", agent_id)))
                        .insert(ScreenPosition::default())
                        .insert(Selectable)
                        .insert(Velocity::default())
                        .insert(Neighbours::default())
                        .insert(
                            Thinker::build()
                                .picker(FirstToScore { threshold: 0.8 })
                                .when(Drunk, RandomMove)
                                .otherwise(Idle),
                        )
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

#[derive(Clone, Component, Debug, Default)]
pub struct RandomMove;

const TARGET_SPD: f32 = 10.0;
const TARGET_SPD_D: f32 = 0.5;
const TARGET_TIME: f32 = 10.;
const TARGET_TIME_D: f32 = 1.;

fn random_move_action(
    ground: Res<Ground>,
    mut action_query: Query<(&Actor, &mut ActionState), With<RandomMove>>,
    mut state_query: Query<(&Transform, Option<&MoveTo>, &mut Velocity)>,
    mut cmd: Commands,
) {
    for (Actor(actor), mut state) in action_query.iter_mut() {
        if let Ok((transform, move_target, mut velocity)) = state_query.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    let mut rng = thread_rng();
                    let speed = (TARGET_SPD / transform.scale.x).max(0.2);
                    let (min_s, max_s) = (
                        (speed - TARGET_SPD_D).max(0.1),
                        (speed + TARGET_SPD_D).min(MAX_SPEED),
                    );
                    let target_speed = rng.gen_range(min_s..max_s);
                    let target_time =
                        rng.gen_range(TARGET_TIME - TARGET_TIME_D..TARGET_TIME + TARGET_TIME_D);
                    let target_dir = Quat::from_rotation_y(rng.gen_range(0.0..2.0 * PI))
                        .mul_vec3(Vec3::X)
                        .normalize();
                    let target = ground.clamp(
                        transform.translation + target_speed * target_time * target_dir,
                        10.,
                    );
                    if ground.get_tile_vec3(target).is_some() {
                        cmd.entity(*actor).insert(MoveTo {
                            target,
                            speed: target_speed,
                            start_time: Instant::now(),
                        });
                        velocity.breaking = false;
                        *state = ActionState::Executing;
                    } else {
                        // warn!("invalid ground tile {}", target);
                    }
                }
                ActionState::Executing => {
                    if move_target.is_none() {
                        *state = ActionState::Success;
                    }
                }
                ActionState::Cancelled => {
                    clear_path_components(&mut cmd, *actor);
                    velocity.breaking = true;
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        } else {
            *state = ActionState::Failure;
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct Drunk;

const NEW_PATHS_PER_FRAME: u32 = 1;

pub fn drunk_scorer(
    ui: Res<SidePanelState>,
    selected: Query<With<Selected>>,
    mut query: Query<(&Actor, &mut Score), With<Drunk>>,
    moving_query: Query<With<MoveTo>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        let mut new_score = 0.;
        if ui.random_walk_all {
            if score.get() > 0.9 {
                new_score = 1.;
            } else {
                let moving = moving_query.iter().count();
                let mut rng = thread_rng();
                if rng.gen_bool(
                    (NEW_PATHS_PER_FRAME as f64 / (10000. - moving as f64))
                        .min(0.999)
                        .max(0.001),
                ) {
                    new_score = 1.;
                };
            }
        } else if selected.get(*actor).is_ok() {
            if ui.random_walk_selected {
                new_score = 1.;
            }
        }
        score.set(new_score);
    }
}

#[derive(Clone, Component, Debug)]
pub struct Idle;

fn idle_action(mut action_query: Query<&mut ActionState, (With<Actor>, With<Idle>)>) {
    for mut state in action_query.iter_mut() {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

fn f1_just_pressed(keyboard: Res<Input<KeyCode>>) -> ShouldRun {
    if keyboard.just_pressed(KeyCode::F1) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn show_unit_debug_info(
    unit_query: Query<(Entity, &Neighbours), (With<Selected>, With<HasThinker>)>,
    thinker_query: Query<(Entity, &Actor, &Thinker)>,
    action_query: Query<(
        Entity,
        &Actor,
        &ActionState,
        Option<&RandomMove>,
        Option<&Idle>,
    )>,
) {
    let mut info = String::new();
    for (unit_ent, neighbours) in unit_query.iter() {
        info.push_str(format!("unit: {:?}, ", unit_ent).as_str());
        for (thinker_ent, actor, thinker) in thinker_query.iter() {
            if actor.0 == unit_ent {
                info.push_str(format!("thinker: {:?}\n", thinker_ent).as_str());
                info.push_str(format!("{:?}\n", thinker).as_str());
                break;
            }
        }
        info.push_str(
            format!(
                "neighbours (<{}m): {:?}\n",
                neighbours.range, neighbours.neighbours
            )
            .as_str(),
        );
        for (action_ent, actor, action_state, random_move, idle) in action_query.iter() {
            if actor.0 == unit_ent {
                info.push_str(format!("action: {:?} ({:?})", action_ent, action_state).as_str());
                if let Some(random_move) = random_move {
                    info.push_str(format!(" {:?}\n", random_move).as_str());
                } else if let Some(idle) = idle {
                    info.push_str(format!(" {:?}\n", idle).as_str());
                } else {
                    info.push_str(" mistery action\n");
                }
            }
        }
        break;
    }
    info!("{}", info);
}
