use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use bevy_mod_picking::Selection;
use big_brain::{prelude::*, thinker::HasThinker};
use rand::{thread_rng, Rng};

use crate::ui::UiState;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BigBrainPlugin)
            .add_system_to_stage(BigBrainStage::Actions, idle_action)
            .add_system_to_stage(BigBrainStage::Actions, random_move_action)
            .add_system_to_stage(BigBrainStage::Scorers, drunk_scorer)
            .add_system(move_to_target)
            .add_system(update_inspected_unit)
            .register_inspectable::<Velocity>()
            .register_inspectable::<MoveTarget>()
            .register_inspectable::<RandomMove>();
    }
}

#[derive(Clone, Component, Debug, Default, Inspectable)]
pub struct Velocity {
    pub velocity: Vec3,
}

#[derive(Clone, Component, Debug, Default, Inspectable)]
pub struct MoveTarget {
    pub target: Vec3,
    pub speed: f32,
}

fn move_to_target(
    time: Res<Time>,
    mut state: Query<(Entity, &mut Transform, &mut Velocity, &MoveTarget)>,
    mut cmd: Commands,
) {
    for (entity, mut transform, mut velocity, MoveTarget { target, speed }) in state.iter_mut() {
        if (transform.translation - *target).length() > 0.5 {
            let dt = time.delta_seconds();
            let target_velocity = (*target - transform.translation).normalize() * *speed;
            let acceleration = 10. * (target_velocity - velocity.velocity).normalize() * dt;
            velocity.velocity += acceleration;
            transform.translation += velocity.velocity * dt;
        } else {
            cmd.entity(entity).remove::<MoveTarget>();
        }
    }
}

#[derive(Clone, Component, Debug, Inspectable)]
pub struct RandomMove {
    pub target: Vec3,
    pub speed: f32,
}

impl RandomMove {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let target = Vec3::new(rng.gen_range(-10.0..10.0), 0., rng.gen_range(-10.0..10.0));
        let speed = rng.gen_range(0.0..5.0);
        RandomMove { target, speed }
    }
}

fn random_move_action(
    mut action_query: Query<(&Actor, &mut ActionState, &RandomMove)>,
    state_query: Query<(&Transform, Option<&MoveTarget>)>,
    mut cmd: Commands,
) {
    for (Actor(actor), mut state, RandomMove { target, speed }) in action_query.iter_mut() {
        if let Ok((transform, move_target)) = state_query.get(*actor) {
            match *state {
                ActionState::Requested => {
                    cmd.entity(*actor).insert(MoveTarget {
                        target: transform.translation + *target,
                        speed: *speed,
                    });
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if move_target.is_none() {
                        *state = ActionState::Success;
                    }
                }
                ActionState::Cancelled => {
                    cmd.entity(*actor).remove::<MoveTarget>();
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

fn update_inspected_unit(
    mut ui: ResMut<UiState>,
    unit_query: Query<(Entity, &Selection), With<HasThinker>>,
    thinker_query: Query<(Entity, &Actor), With<Thinker>>,
    action_query: Query<(Entity, &Actor), With<ActionState>>,
) {
    let mut info = String::new();
    for (unit_ent, selection) in unit_query.iter() {
        if selection.selected() {
            info.push_str(format!("unit: {:?}\n", unit_ent).as_str());
            for (thinker_ent, actor) in thinker_query.iter() {
                if actor.0 == unit_ent {
                    info.push_str(format!("thinker: {:?}\n", thinker_ent).as_str());
                    break;
                }
            }
            for (action_ent, actor) in action_query.iter() {
                if actor.0 == unit_ent {
                    info.push_str(format!("action: {:?}\n", action_ent).as_str());
                }
            }
            break;
        }
    }
    ui.ai_debug_info = info;
}
