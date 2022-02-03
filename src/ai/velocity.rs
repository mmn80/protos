use bevy::prelude::*;

use super::{fast_unit_index::Neighbours, ground::Ground, sparse_grid::GridPos};

pub struct VelocityPlugin;

impl Plugin for VelocityPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(avoid_collisions).add_system(apply_velocity);
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
pub const MAX_SPEED: f32 = 30.;

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
