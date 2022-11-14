use std::{f32::consts::PI, time::Instant};

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use big_brain::{choices::Choice, prelude::*, thinker::HasThinker};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, LogNormal};

use super::{
    fast_unit_index::Neighbours,
    ground::Ground,
    pathfind::{clear_path_components, Moving},
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
            .add_system_to_stage(BigBrainStage::Actions, sleep_action)
            .add_system_to_stage(BigBrainStage::Actions, random_move_action)
            .add_system_to_stage(BigBrainStage::Scorers, sleepy_scorer)
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
                            Sleeping::default(),
                            Thinker::build()
                                .picker(HighestScoreAbove { threshold: 0.8 })
                                .when(Drunk, RandomMove)
                                .when(Sleepy, Sleep)
                                .otherwise(Idle),
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

// picker

#[derive(Debug, Clone, Default)]
pub struct HighestScoreAbove {
    pub threshold: f32,
}

impl HighestScoreAbove {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Picker for HighestScoreAbove {
    fn pick<'a>(&self, choices: &'a [Choice], scores: &Query<&Score>) -> Option<&'a Choice> {
        let mut picked = None;
        let mut max_score = 0.0;
        for choice in choices {
            let score = choice.calculate(scores);
            if score >= self.threshold && score > max_score {
                picked = Some(choice);
                max_score = score;
            }
        }
        picked
    }
}

// random walk

#[derive(Clone, Component, Debug, Default)]
pub struct RandomMove;

const TARGET_SPD: f32 = 10.0;
const TARGET_SPD_D: f32 = 0.5;
const TARGET_TIME: f32 = 20.;
const TARGET_TIME_D: f32 = 5.;
const TARGET_MAX_DIST: f32 = 128.;

fn random_move_action(
    ground: Res<Ground>,
    mut action_q: Query<(&Actor, &mut ActionState), With<RandomMove>>,
    mut state_q: Query<(&Transform, Option<&Moving>, &mut Velocity)>,
    mut cmd: Commands,
) {
    let mut rng = thread_rng();
    for (Actor(actor), mut state) in &mut action_q {
        if let Ok((transform, move_target, mut velocity)) = state_q.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
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
                        transform.translation
                            + (target_speed * target_time).min(TARGET_MAX_DIST) * target_dir,
                        10.,
                    );
                    if ground.get_tile_vec3(target).is_some() {
                        cmd.entity(*actor).insert(Moving {
                            target,
                            speed: target_speed,
                            start_time: Instant::now(),
                        });
                        velocity.breaking = false;
                        *state = ActionState::Executing;
                    } else {
                        // warn!("invalid ground tile {target}");
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

pub fn drunk_scorer(
    ui: Res<SidePanelState>,
    selected_q: Query<With<Selected>>,
    mut drunk_q: Query<(&Actor, &mut Score), With<Drunk>>,
) {
    for (Actor(actor), mut score) in &mut drunk_q {
        let mut new_score = 0.;
        if ui.ai_active_all {
            new_score = 0.9;
        } else if selected_q.get(*actor).is_ok() {
            if ui.ai_active_selected {
                new_score = 0.9;
            }
        }
        score.set(new_score);
    }
}

// sleep

#[derive(Clone, Component, Debug)]
pub struct Sleeping {
    pub since: Instant,
    pub duration: f64, // seconds
}

impl Default for Sleeping {
    fn default() -> Self {
        Self {
            since: Instant::now(),
            duration: SLEEP_TIME,
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct Awake {
    pub since: Instant,
    pub duration: f64, // seconds
}

impl Awake {
    pub fn new() -> Self {
        Self {
            since: Instant::now(),
            duration: AWAKE_TIME + thread_rng().gen_range(-AWAKE_TIME_D..AWAKE_TIME_D),
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct Sleep;

const SLEEP_TIME: f64 = 1.;
const SLEEP_TIME_D: f64 = 0.1;
const AWAKE_TIME: f64 = 600.;
const AWAKE_TIME_D: f64 = 50.;

fn sleep_action(
    mut action_q: Query<(&Actor, &mut ActionState), With<Sleep>>,
    sleeping_q: Query<&Sleeping>,
    mut cmd: Commands,
) {
    let mut rng = thread_rng();
    for (Actor(actor), mut state) in &mut action_q {
        match *state {
            ActionState::Requested => {
                cmd.entity(*actor).remove::<Awake>().insert(Sleeping {
                    since: Instant::now(),
                    duration: SLEEP_TIME + rng.gen_range(-SLEEP_TIME_D..SLEEP_TIME_D),
                });
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                if let Ok(sleeping) = sleeping_q.get(*actor) {
                    let dt = Instant::now() - sleeping.since;
                    if dt.as_secs_f64() > sleeping.duration {
                        cmd.entity(*actor).remove::<Sleeping>().insert(Awake::new());
                        *state = ActionState::Success;
                    }
                } else {
                    cmd.entity(*actor).insert(Awake::new());
                    *state = ActionState::Failure;
                }
            }
            ActionState::Cancelled => {
                cmd.entity(*actor).remove::<Sleeping>().insert(Awake::new());
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct Sleepy;

pub fn sleepy_scorer(
    mut sleepy_q: Query<(&Actor, &mut Score), With<Sleepy>>,
    awake_q: Query<&Awake>,
    sleeping_q: Query<With<Sleeping>>,
) {
    for (Actor(actor), mut score) in &mut sleepy_q {
        let mut new_score = 0.;
        if let Ok(awake) = awake_q.get(*actor) {
            let dt = Instant::now() - awake.since;
            if dt.as_secs_f64() > awake.duration {
                new_score = 1.;
            }
        } else if sleeping_q.get(*actor).is_ok() {
            new_score = 1.;
        }
        score.set(new_score);
    }
}

// idle

#[derive(Clone, Component, Debug)]
pub struct Idle;

fn idle_action(mut action_query: Query<&mut ActionState, (With<Actor>, With<Idle>)>) {
    for mut state in &mut action_query {
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

// debug

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
        Option<&Sleep>,
    )>,
) {
    let mut info = String::new();
    for (unit_ent, neighbours) in &unit_query {
        info.push_str(format!("unit: {:?}, ", unit_ent).as_str());
        for (thinker_ent, actor, thinker) in &thinker_query {
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
        for (action_ent, actor, action_state, random_move, idle) in &action_query {
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
    info!("{info}");
}
