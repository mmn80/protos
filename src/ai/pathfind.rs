use std::{sync::Arc, time::Instant};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use pathfinding::prelude::astar;

use super::{ground::Ground, sparse_grid::GridPos, velocity::Velocity};

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_pathfinding_tasks)
            .add_system(handle_pathfinding_tasks)
            .add_system(move_to_target);
    }
}

#[derive(Clone, Component, Debug)]
pub struct Moving {
    pub target: Vec3,
    pub speed: f32,
    pub start_time: Instant,
}

impl Default for Moving {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            speed: 0.,
            start_time: Instant::now(),
        }
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct MovingPath {
    pub path: Vec<Vec3>,
    pub current: usize,
}

impl MovingPath {
    pub fn smoothify_path(path: Vec<GridPos>, start: Vec3, end: Vec3) -> Vec<Vec3> {
        if path.is_empty() {
            return Vec::new();
        }
        let mut res = vec![start];
        let no_dir = GridPos::new(0, 0);
        let mut dir = no_dir;
        let mut curr = path[0];
        for p in path.iter().skip(1) {
            let new_dir = *p - curr;
            assert!(new_dir.x.abs() <= 1 && new_dir.y.abs() <= 1, "invalid path");
            if new_dir != dir && dir != no_dir {
                let mut pos = Vec3::new(p.x as f32, start.y, p.y as f32);
                if new_dir.x == 1 {
                    pos.z += 0.5;
                } else if new_dir.x == -1 {
                    pos.x += 1.0;
                    pos.z += 0.5;
                } else if new_dir.y == 1 {
                    pos.x += 0.5;
                } else if new_dir.y == -1 {
                    pos.x += 0.5;
                    pos.z += 1.0;
                }
                res.push(pos);
            }
            dir = new_dir;
            curr = *p;
        }
        res.push(end);
        res
    }
}

pub fn clear_path_components(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<Moving>()
        .remove::<MovingPath>();
}

struct PathfindingTaskResult {
    path: Vec<Vec3>,
}

fn spawn_pathfinding_tasks(
    thread_pool: Res<AsyncComputeTaskPool>,
    ground: Res<Ground>,
    query: Query<
        (Entity, &Name, &Transform, &Moving),
        (Without<MovingPath>, Without<Task<PathfindingTaskResult>>),
    >,
    mut cmd: Commands,
) {
    if query.is_empty() {
        return;
    }
    let shared_grid = Arc::new(ground.nav_grid().clone());
    for (entity, name, transform, move_to) in query.iter() {
        let name = name.to_string();
        let from = transform.translation;
        let to = move_to.target;
        let grid = shared_grid.clone();
        let task = thread_pool.spawn(async move {
            let from_grid = from.into();
            let to_grid = to.into();
            let begin_time = Instant::now();
            let result = astar(
                &from_grid,
                |p| grid.successors(*p),
                |p| p.distance(to_grid) as u32,
                |p| *p == to_grid,
            );
            let path = if let Some((path, _)) = result {
                MovingPath::smoothify_path(path, from, to)
            } else {
                warn!(
                    "failed to find a path for {name} from {from_grid} to {to_grid} in {}ms",
                    (Instant::now() - begin_time).as_millis()
                );
                vec![]
            };
            let duration = Instant::now() - begin_time;
            let dt = duration.as_millis();
            if dt > 100 && !path.is_empty() {
                info!(
                    "path for {name} from {from_grid} to {to_grid} computed in {dtms}ms",
                    dtms = dt / 1000
                );
            }
            PathfindingTaskResult { path }
        });
        cmd.entity(entity).insert(task);
    }
}

fn handle_pathfinding_tasks(
    mut tasks: Query<(Entity, &mut Task<PathfindingTaskResult>, Option<&Moving>)>,
    mut cmd: Commands,
) {
    for (entity, mut task, moving) in tasks.iter_mut() {
        if let Some(result) = future::block_on(future::poll_once(&mut *task)) {
            if moving.is_some() {
                cmd.entity(entity)
                    .insert(MovingPath {
                        path: result.path,
                        current: 0,
                    })
                    .remove::<Task<PathfindingTaskResult>>();
            } else {
                cmd.entity(entity).remove::<Task<PathfindingTaskResult>>();
            }
        }
    }
}

const TURN_ACC: f32 = 10.;

fn move_to_target(
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &mut Velocity, &Moving, &mut MovingPath)>,
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
                let curr = transform.translation;
                for idx in start_idx..end_idx {
                    if (path.path[idx] - curr).length() < 1. {
                        path.current = (idx + 1).min(p_max);
                        break;
                    }
                }
                path.path[path.current]
            };
            if (transform.translation - target).length() > 0.2 {
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
