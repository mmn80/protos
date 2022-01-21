use bevy::prelude::*;
use bevy_mod_picking::Selection;
use big_brain::prelude::*;
use rand::{thread_rng, Rng};

use crate::ui::UiState;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BigBrainPlugin)
            .add_system_to_stage(BigBrainStage::Actions, random_move_action)
            .add_system_to_stage(BigBrainStage::Scorers, drunk_scorer);
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct Velocity {
    pub velocity: Vec3,
}

#[derive(Clone, Component, Debug, Default)]
pub struct MoveTarget {
    pub target: Option<Vec3>,
}

#[derive(Clone, Component, Debug)]
pub struct RandomMove {
    pub target: Vec3,
    pub speed: f32,
}

impl RandomMove {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let target = Vec3::new(rng.gen_range(-10.0..10.0), 0., rng.gen_range(-10.0..10.0));
        let speed = rng.gen_range(0.0..1.0);
        RandomMove { target, speed }
    }
}

fn random_move_action(
    time: Res<Time>,
    mut action_query: Query<(&Actor, &mut ActionState, &RandomMove)>,
    mut state_query: Query<(&mut Transform, &mut Velocity, &mut MoveTarget)>,
) {
    for (Actor(actor), mut state, random_move) in action_query.iter_mut() {
        match *state {
            ActionState::Requested => {
                if let Ok((transform, _, mut move_target)) = state_query.get_mut(*actor) {
                    move_target.target = Some(transform.translation + random_move.target);
                    *state = ActionState::Executing;
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if let Ok((mut transform, mut velocity, move_target)) = state_query.get_mut(*actor)
                {
                    let move_target = move_target.target.unwrap();
                    let dt = time.delta_seconds();
                    let target_velocity =
                        (move_target - transform.translation).normalize() * random_move.speed;
                    let acceleration = 10. * (target_velocity - velocity.velocity).normalize() * dt;
                    velocity.velocity += acceleration;
                    transform.translation += velocity.velocity * dt;
                    if (transform.translation - random_move.target).length() < 0.5 {
                        *state = ActionState::Success;
                    }
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {
                if let Ok((_, _, mut move_target)) = state_query.get_mut(*actor) {
                    move_target.target = None;
                }
            }
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
    if ui.random_walk_all || ui.random_walk_selected {
        for (Actor(actor), mut score) in query.iter_mut() {
            if let Ok(sel) = selected.get(*actor) {
                if ui.random_walk_all || sel.selected() {
                    score.set(1.);
                }
            }
        }
    }
}
