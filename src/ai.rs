use bevy::{ecs::schedule::ShouldRun, prelude::*};
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
            .add_system(show_ai_debug_info.with_run_criteria(f1_just_pressed))
            .register_inspectable::<Velocity>()
            .register_inspectable::<MoveTarget>();
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

#[derive(Clone, Component, Debug, Default)]
pub struct RandomMove;

fn random_move_action(
    mut action_query: Query<(&Actor, &mut ActionState), With<RandomMove>>,
    state_query: Query<(&Transform, Option<&MoveTarget>)>,
    mut cmd: Commands,
) {
    for (Actor(actor), mut state) in action_query.iter_mut() {
        if let Ok((transform, move_target)) = state_query.get(*actor) {
            match *state {
                ActionState::Requested => {
                    let mut rng = thread_rng();
                    let target =
                        Vec3::new(rng.gen_range(-10.0..10.0), 0., rng.gen_range(-10.0..10.0));
                    let speed = rng.gen_range(0.0..5.0);
                    cmd.entity(*actor).insert(MoveTarget {
                        target: transform.translation + target,
                        speed: speed,
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

fn f1_just_pressed(keyboard: Res<Input<KeyCode>>) -> ShouldRun {
    if keyboard.just_pressed(KeyCode::F1) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn show_ai_debug_info(
    unit_query: Query<(Entity, &Selection), With<HasThinker>>,
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
    for (unit_ent, selection) in unit_query.iter() {
        if selection.selected() {
            info.push_str(format!("unit: {:?}, ", unit_ent).as_str());
            for (thinker_ent, actor, thinker) in thinker_query.iter() {
                if actor.0 == unit_ent {
                    info.push_str(format!("thinker: {:?}\n", thinker_ent).as_str());
                    info.push_str(format!("{:?}\n", thinker).as_str());
                    break;
                }
            }
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
