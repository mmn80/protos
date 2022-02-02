use std::f32::consts::PI;
use std::time::Instant;

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use big_brain::{prelude::*, thinker::HasThinker};
use pathfinding::prelude::astar;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, LogNormal};

use crate::ai::{fast_unit_index::Neighbours, ground::Ground};
use crate::camera::ScreenPosition;
use crate::ui::selection::{Selectable, Selected};
use crate::ui::side_panel::SidePanelState;

use super::sparse_grid::GridPos;

pub struct FastUnitPlugin;

impl Plugin for FastUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.after("ground_setup"))
            .add_system_to_stage(BigBrainStage::Actions, idle_action)
            .add_system_to_stage(BigBrainStage::Actions, random_move_action)
            .add_system_to_stage(BigBrainStage::Scorers, drunk_scorer)
            .add_system(compute_paths)
            .add_system(move_to_target)
            .add_system(avoid_collisions)
            .add_system(apply_velocity)
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
        for x in (10..ground.width() - 10).step_by(10) {
            for z in (10..ground.width() - 10).step_by(10) {
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
                        .insert(Name::new(format!("Agent[{},{}]", x / 10, z / 10)))
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

#[derive(Clone, Component, Debug)]
pub struct Velocity {
    pub velocity: Vec3,
    pub breaking: bool,
    pub ignore_collisions: bool,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            breaking: true,
            ignore_collisions: false,
        }
    }
}

fn apply_velocity(
    time: Res<Time>,
    ground: Res<Ground>,
    mut query: Query<(&mut Transform, &mut Velocity)>,
) {
    let dt = time.delta_seconds();
    for (mut transform, mut velocity) in query.iter_mut() {
        let pos = transform.translation + velocity.velocity * dt;
        if velocity.ignore_collisions || ground.get_tile_vec3(pos).is_some() {
            transform.translation = pos;
        } else {
            velocity.velocity = Vec3::ZERO;
        }
        if velocity.breaking {
            if velocity.velocity.length() < 0.5 {
                velocity.velocity = Vec3::ZERO;
            } else {
                let dir = velocity.velocity.normalize();
                velocity.velocity -= 20. * dir * dt;
            }
        }
        if velocity.velocity.length_squared() > 0.1 {
            let look_at = transform.translation + velocity.velocity.normalize();
            transform.look_at(look_at, Vec3::Y);
        }
        if !velocity.ignore_collisions {
            let pos = transform.translation;
            let pos = Vec3::new(pos.x.floor() + 0.5, pos.y, pos.z.floor() + 0.5);
            if ground.contains(pos) && ground.get_tile(pos.into()).is_none() {
                for i in 1..200 {
                    let cell = pos + (i as f32) * Vec3::X;
                    if !ground.contains(cell) {
                        break;
                    }
                    let c = cell.into();
                    if ground.get_tile(c).is_some() {
                        let neigh = vec![
                            GridPos::new(c.x + 1, c.y),
                            GridPos::new(c.x + 1, c.y - 1),
                            GridPos::new(c.x, c.y - 1),
                            GridPos::new(c.x - 1, c.y - 1),
                            GridPos::new(c.x - 1, c.y),
                            GridPos::new(c.x - 1, c.y + 1),
                            GridPos::new(c.x, c.y + 1),
                            GridPos::new(c.x + 1, c.y + 1),
                        ];
                        if neigh.iter().all(|p| ground.get_tile(*p).is_some()) {
                            transform.translation = cell;
                            velocity.velocity = Vec3::ZERO;
                            velocity.breaking = true;
                            break;
                        }
                    }
                }
            }
        }
    }
}

const COLLISION_DIST: f32 = 5.;
const COLLISION_FORCE: f32 = 5.;
const COLLISION_BLOCK_FORCE: f32 = 50.;
const MAX_SPEED: f32 = 30.;

fn avoid_collisions(
    time: Res<Time>,
    ground: Res<Ground>,
    mut query: Query<(&Transform, &Neighbours, &mut Velocity)>,
    neigh_query: Query<&Transform>,
) {
    let dt = time.delta_seconds();
    for (transform, neighbours, mut velocity) in query.iter_mut() {
        if velocity.ignore_collisions || ground.get_tile_vec3(transform.translation).is_none() {
            continue;
        }
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

        let mut free = true;
        let pos = transform.translation;
        let pos = Vec3::new(pos.x.floor() + 0.5, pos.y, pos.z.floor() + 0.5);
        for cell in [
            pos + Vec3::X,
            pos + Vec3::Z,
            pos - Vec3::X,
            pos - Vec3::Z,
            pos + Vec3::X + Vec3::Z,
            pos - Vec3::X - Vec3::Z,
            pos + Vec3::X - Vec3::Z,
            pos - Vec3::X + Vec3::Z,
        ] {
            if ground.get_tile_vec3(cell).is_none() {
                free = false;
                let src_size = transform.scale.x;
                let dir = transform.translation - cell;
                let dist = dir.length();
                let direction = dir.normalize();
                if dist > 0. {
                    let acceleration = COLLISION_BLOCK_FORCE * dist.powi(4) / src_size;
                    velocity.velocity += dt * acceleration * direction;
                }
            }
        }
        let speed = velocity.velocity.length();
        if free && speed > MAX_SPEED {
            velocity.velocity *= MAX_SPEED / speed;
        }
    }
}

#[derive(Clone, Component, Debug)]
pub struct MoveTo {
    pub target: Vec3,
    pub speed: f32,
    pub start_time: Instant,
}

impl Default for MoveTo {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            speed: 0.,
            start_time: Instant::now(),
        }
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct MoveToPath {
    pub path: Vec<GridPos>,
    pub current: usize,
}

fn clear_path_components(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<MoveTo>()
        .remove::<MoveToPath>();
}

fn compute_paths(
    ground: Res<Ground>,
    query: Query<(Entity, &Transform, &MoveTo), Without<MoveToPath>>,
    mut cmd: Commands,
) {
    let begin = Instant::now();
    let mut paths = 0;
    let mut failed = 0;
    let mut todo = vec![];
    for (
        entity,
        transform,
        MoveTo {
            target,
            speed: _,
            start_time,
        },
    ) in query.iter()
    {
        let start = transform.translation.into();
        let end = (*target).into();
        todo.push((entity, start, end, start_time));
    }
    todo.sort_unstable_by_key(|(_, _, _, t)| *t);
    todo.reverse();

    let astar_begin = Instant::now();
    for (entity, start, end, _) in todo {
        let result = astar(
            &start,
            |p| ground.nav_grid_successors(*p),
            |p| p.distance(end),
            |p| *p == end,
        );
        let path = if let Some((path, _)) = result {
            path
        } else {
            failed += 1;
            vec![]
        };
        cmd.entity(entity).insert(MoveToPath { path, current: 0 });

        paths += 1;
        let dt = (Instant::now() - begin).as_micros();
        if dt > 1000 {
            break;
        }
    }
    let dt = (Instant::now() - begin).as_micros();
    if paths > 0 && dt > 5000 {
        let dt_astar = (Instant::now() - astar_begin).as_micros();
        info!(
            "{} paths ({} failed) computed in {}μs (setup: {}μs)",
            paths,
            failed,
            dt,
            dt - dt_astar
        );
    }
}

const TURN_ACC: f32 = 10.;

fn move_to_target(
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &mut Velocity, &MoveTo, &mut MoveToPath)>,
    mut cmd: Commands,
) {
    for (entity, transform, mut velocity, move_to, mut path) in query.iter_mut() {
        if path.path.is_empty() {
            let target = move_to.target;
            if (transform.translation - target).length() > 0.5 {
                let dt = time.delta_seconds();
                let target_velocity = (target - transform.translation).normalize() * move_to.speed;
                let acceleration =
                    TURN_ACC * (target_velocity - velocity.velocity).normalize() * dt;
                velocity.velocity += acceleration;
            } else {
                clear_path_components(&mut cmd, entity);
                velocity.breaking = true;
            }
        } else {
            let p_max = path.path.len() - 1;
            let target = {
                let start_idx = path.current;
                let end_idx = (start_idx + 8).min(p_max);
                let curr = transform.translation.into();
                for idx in start_idx..end_idx {
                    if path.path[idx] == curr {
                        path.current = (idx + 1).min(p_max);
                        break;
                    }
                }
                let target = path.path[path.current];
                Vec3::new(
                    target.x as f32 + 0.5,
                    transform.translation.y,
                    target.y as f32 + 0.5,
                )
            };
            if (transform.translation - target).length() > 0.4 {
                let dt = time.delta_seconds();
                let target_velocity = (target - transform.translation).normalize() * move_to.speed;
                let acceleration =
                    TURN_ACC * (target_velocity - velocity.velocity).normalize() * dt;
                velocity.velocity += acceleration;
            }
            if path.current >= p_max {
                clear_path_components(&mut cmd, entity);
                velocity.breaking = true;
            }
        }
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct RandomMove;

const TARGET_DST: f32 = 10.;
const TARGET_SPD: f32 = 10.0;
const TARGET_SPD_D: f32 = 0.5;

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
                    let v = f32::max(0.2, TARGET_SPD / transform.scale.x);
                    let dst = v * TARGET_DST;
                    let width = ground.width() as f32;
                    let target = (transform.translation
                        + Vec3::new(rng.gen_range(-dst..dst), 0., rng.gen_range(-dst..dst)))
                    .clamp(
                        Vec3::new(10., 0., 10.),
                        Vec3::new(width - 10., 10., width - 10.),
                    );
                    let (min_s, max_s) = (f32::max(0.1, v - TARGET_SPD_D), v + TARGET_SPD_D);
                    let speed = rng.gen_range(min_s..max_s);
                    if ground.get_tile_vec3(target).is_some() {
                        cmd.entity(*actor).insert(MoveTo {
                            target,
                            speed,
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

pub fn drunk_scorer(
    ui: Res<SidePanelState>,
    selected: Query<With<Selected>>,
    mut query: Query<(&Actor, &mut Score), With<Drunk>>,
) {
    for (Actor(actor), mut score) in query.iter_mut() {
        let mut new_score = 0.;
        if ui.random_walk_all {
            new_score = 1.;
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
