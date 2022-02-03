use std::time::Instant;

use bevy::prelude::*;
use pathfinding::prelude::astar;

use super::{ground::Ground, sparse_grid::GridPos, velocity::Velocity};

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(compute_paths).add_system(move_to_target);
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
    pub path: Vec<Vec3>,
    pub current: usize,
}

impl MoveToPath {
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
    let mut todo: Vec<_> = query
        .iter()
        .map(|(entity, transform, move_to)| {
            (
                entity,
                transform.translation,
                move_to.target,
                move_to.start_time,
            )
        })
        .collect();
    todo.sort_unstable_by_key(|(_, _, _, t)| *t);

    let astar_begin = Instant::now();
    for (entity, start, end, _) in todo {
        let end_grid = end.into();
        let result = astar(
            &start.into(),
            |p| ground.nav_grid_successors(*p),
            |p| p.distance(end_grid) as u32,
            |p| *p == end_grid,
        );
        let path = if let Some((path, _)) = result {
            MoveToPath::smoothify_path(path, start, end)
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
    if paths > 0 && dt > 10000 {
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
