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
            .add_system_to_stage(BigBrainStage::Scorers, drunk_scorer)
            .add_system(move_to_target);
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct Velocity {
    pub velocity: Vec3,
}

#[derive(Clone, Component, Debug, Default)]
pub struct MoveTarget {
    pub target_pos: Option<Vec3>,
    pub target_speed: f32,
}

fn move_to_target(
    time: Res<Time>,
    mut state: Query<(&mut Transform, &mut Velocity, &mut MoveTarget)>,
) {
    for (mut transform, mut velocity, mut move_target) in state.iter_mut() {
        if let MoveTarget {
            target_pos: Some(target_pos),
            target_speed,
        } = move_target.clone()
        {
            if (transform.translation - target_pos).length() < 0.5 {
                let dt = time.delta_seconds();
                let target_velocity =
                    (target_pos - transform.translation).normalize() * target_speed;
                let acceleration = 10. * (target_velocity - velocity.velocity).normalize() * dt;
                velocity.velocity += acceleration;
                transform.translation += velocity.velocity * dt;
            } else {
                move_target.target_pos = None;
            }
        }
    }
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
    mut action_query: Query<(&Actor, &mut ActionState, &RandomMove)>,
    mut state_query: Query<(&Transform, &mut MoveTarget)>,
) {
    for (Actor(actor), mut state, random_move) in action_query.iter_mut() {
        if let Ok((transform, mut move_target)) = state_query.get_mut(*actor) {
            match *state {
                ActionState::Requested => {
                    move_target.target_pos = Some(transform.translation + random_move.target);
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if move_target.target_pos.is_none() {
                        *state = ActionState::Success;
                    }
                }
                ActionState::Cancelled => {
                    move_target.target_pos = None;
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
