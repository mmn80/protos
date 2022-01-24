use std::f32::consts::PI;

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_picking::Selection;
use big_brain::{prelude::*, thinker::HasThinker};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, LogNormal};

use crate::{fast_unit_index::Neighbours, ui::UiState};

pub const MAP_SIZE: f32 = 1000.;

pub struct FastUnitPlugin;

impl Plugin for FastUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(BigBrainStage::Actions, idle_action)
            .add_system_to_stage(BigBrainStage::Actions, random_move_action)
            .add_system_to_stage(BigBrainStage::Scorers, drunk_scorer)
            .add_system(move_to_target)
            .add_system(avoid_collisions)
            .add_system(apply_velocity)
            .add_system(show_unit_debug_info.with_run_criteria(f1_just_pressed))
            .register_inspectable::<Velocity>()
            .register_inspectable::<MoveTarget>();
    }
}

#[derive(Clone, Component, Debug, Default, Inspectable)]
pub struct Velocity {
    pub velocity: Vec3,
    pub breaking: bool,
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &mut Velocity)>) {
    let dt = time.delta_seconds();
    for (mut transform, mut velocity) in query.iter_mut() {
        transform.translation += velocity.velocity * dt;
        if velocity.breaking {
            if velocity.velocity.length() < 0.5 {
                velocity.velocity = Vec3::ZERO;
                velocity.breaking = false;
            } else {
                let dir = velocity.velocity.normalize();
                velocity.velocity -= dir * dt;
            }
        }
    }
}

const COLLISION_DIST: f32 = 5.;
const COLLISION_FORCE: f32 = 5.;

fn avoid_collisions(
    time: Res<Time>,
    mut query: Query<(&Transform, &Neighbours, &mut Velocity)>,
    neigh_query: Query<&Transform>,
) {
    let dt = time.delta_seconds();
    for (transform, neighbours, mut velocity) in query.iter_mut() {
        let speed = velocity.velocity.length();
        if speed > 0.5 {
            if let Some(nearest) = neighbours.neighbours.first() {
                if let Ok(neigh_tran) = neigh_query.get(nearest.entity) {
                    let src_size = transform.scale.x;
                    let dest_size = neigh_tran.scale.x;
                    let dist = f32::max(0., nearest.distance - src_size - dest_size);
                    let force = 1. - dist / COLLISION_DIST;
                    if force > 0. {
                        let acceleration =
                            COLLISION_FORCE * f32::min(speed, 10.) * force.powi(5) / src_size;
                        let direction = (transform.translation - neigh_tran.translation)
                            .normalize()
                            .cross(Vec3::Y);
                        let old_speed = velocity.velocity.length();
                        velocity.velocity += dt * acceleration * direction;
                        let new_speed = velocity.velocity.length();
                        if new_speed > old_speed {
                            velocity.velocity *= old_speed / new_speed;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Component, Debug, Default, Inspectable)]
pub struct MoveTarget {
    pub target: Vec3,
    pub speed: f32,
}

const TURN_ACC: f32 = 10.;

fn move_to_target(
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &mut Velocity, &MoveTarget)>,
    mut cmd: Commands,
) {
    for (entity, transform, mut velocity, MoveTarget { target, speed }) in query.iter_mut() {
        if (transform.translation - *target).length() > 0.5 {
            let dt = time.delta_seconds();
            let target_velocity = (*target - transform.translation).normalize() * *speed;
            let acceleration = TURN_ACC * (target_velocity - velocity.velocity).normalize() * dt;
            velocity.velocity += acceleration;
            velocity.breaking = false;
        } else {
            cmd.entity(entity).remove::<MoveTarget>();
            velocity.breaking = true;
        }
    }
}

pub fn get_random_radius(mean_radius: f32, stddev: f32) -> f32 {
    let area_dist = LogNormal::new(PI * mean_radius * mean_radius, stddev).unwrap();
    let area = area_dist.sample(&mut thread_rng());
    f32::sqrt(area / PI)
}

#[derive(Clone, Component, Debug, Default)]
pub struct RandomMove;

const TARGET_DST: f32 = 3.;
const TARGET_SPD: f32 = 10.0;
const TARGET_SPD_D: f32 = 0.5;

fn random_move_action(
    mut action_query: Query<(&Actor, &mut ActionState), With<RandomMove>>,
    mut state_query: Query<(&Transform, Option<&MoveTarget>, &mut Velocity)>,
    mut cmd: Commands,
) {
    for (Actor(actor), mut state) in action_query.iter_mut() {
        if let Ok((transform, move_target, mut velocity)) = state_query.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    let mut rng = thread_rng();
                    let v = f32::max(0.2, TARGET_SPD / transform.scale.x);
                    let dst = v * TARGET_DST;
                    let sz = MAP_SIZE / 2. - 10.;
                    let target = (transform.translation
                        + Vec3::new(rng.gen_range(-dst..dst), 0., rng.gen_range(-dst..dst)))
                    .clamp(Vec3::new(-sz, 0., -sz), Vec3::new(sz, 10., sz));
                    let (min_s, max_s) = (f32::max(0.1, v - TARGET_SPD_D), v + TARGET_SPD_D);
                    let speed = rng.gen_range(min_s..max_s);
                    cmd.entity(*actor).insert(MoveTarget { target, speed });
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if move_target.is_none() {
                        *state = ActionState::Success;
                    }
                }
                ActionState::Cancelled => {
                    cmd.entity(*actor).remove::<MoveTarget>();
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
    ui: Res<UiState>,
    selected: Query<&Selection>,
    mut query: Query<(&Actor, &mut Score), With<Drunk>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        let mut new_score = 0.;
        if let Ok(sel) = selected.get(*actor) {
            if ui.random_walk_all || (ui.random_walk_selected && sel.selected()) {
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
    unit_query: Query<(Entity, &Selection, &Neighbours), With<HasThinker>>,
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
    for (unit_ent, selection, neighbours) in unit_query.iter() {
        if selection.selected() {
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
                    info.push_str(
                        format!("action: {:?} ({:?})", action_ent, action_state).as_str(),
                    );
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
    }
    info!("{}", info);
}
