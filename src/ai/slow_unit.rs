use std::f32::consts::PI;

use bevy::{prelude::*, render::primitives::Aabb};
use big_brain::prelude::*;

use super::{
    fast_unit::{Drunk, Idle, RandomMove, Velocity},
    ground::{Ground, GroundMaterialRef},
    sparse_grid::{GridPos, SparseGrid},
};
use crate::{camera::ScreenPosition, ui::multi_select::Selected};

pub struct SlowUnitPlugin;

impl Plugin for SlowUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.after("ground_setup"))
            .add_system_to_stage(CoreStage::PreUpdate, update_nav_grid);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    ground: Res<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bld_id = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -5.,
                max_x: 5.,
                min_y: 0.,
                max_y: 8.,
                min_z: -5.,
                max_z: 5.,
            })),
            material: materials.add(Color::rgb(1., 0.3, 0.6).into()),
            transform: Transform::from_rotation(Quat::from_rotation_y(2.))
                .with_translation(Vec3::new(510., 0., 500.)),
            ..Default::default()
        })
        .insert(Name::new("Building"))
        .insert(NavGridCarve::default())
        .insert(ScreenPosition::default())
        .insert(Selected::default())
        .insert(Velocity {
            velocity: Vec3::ZERO,
            breaking: false,
            ignore_collisions: true,
        })
        .insert(
            Thinker::build()
                .picker(FirstToScore { threshold: 0.8 })
                .when(Drunk, RandomMove)
                .otherwise(Idle),
        )
        .id();
    if let Some(ground_ent) = ground.entity {
        commands.entity(ground_ent).add_child(bld_id);
    } else {
        warn!("NO GROUND!!");
    }
}

#[derive(Clone, Component, Debug)]
pub struct NavGridCarve {
    last_pos: Vec3,
    last_rot: Quat,
    ground_pos: Option<GridPos>,
    ground: SparseGrid<GroundMaterialRef>,
}

impl Default for NavGridCarve {
    fn default() -> Self {
        Self {
            last_pos: Vec3::ZERO,
            last_rot: Quat::IDENTITY,
            ground_pos: None,
            ground: SparseGrid::new(1, 1, None),
        }
    }
}

fn update_nav_grid(
    mut ground: ResMut<Ground>,
    mut query: Query<(&Transform, &Aabb, &mut NavGridCarve)>,
) {
    for (transform, aabb, mut carve) in query.iter_mut() {
        if (transform.translation - carve.last_pos).length() < 0.5
            && transform.rotation.angle_between(carve.last_rot) < PI / 18.
        {
            continue;
        }
        carve.last_pos = transform.translation;
        carve.last_rot = transform.rotation;

        let (ext_x, ext_z) = (aabb.half_extents.x, aabb.half_extents.z);

        let bounds = {
            let bot_l = transform.mul_vec3(aabb.center + Vec3::new(-ext_x, 0., -ext_z));
            let bot_r = transform.mul_vec3(aabb.center + Vec3::new(-ext_x, 0., ext_z));
            let top_l = transform.mul_vec3(aabb.center + Vec3::new(ext_x, 0., -ext_z));
            let top_r = transform.mul_vec3(aabb.center + Vec3::new(ext_x, 0., ext_z));
            Rect {
                left: bot_l.x.min(bot_r.x).min(top_l.x).min(top_r.x).floor() as u32,
                right: bot_l.x.max(bot_r.x).max(top_l.x).max(top_r.x).ceil() as u32,
                top: bot_l.z.max(bot_r.z).max(top_l.z).max(top_r.z).ceil() as u32,
                bottom: bot_l.z.min(bot_r.z).min(top_l.z).min(top_r.z).floor() as u32,
            }
        };

        let mut dirty_rect = bounds;

        if let Some(pos) = carve.ground_pos {
            for y in 0..carve.ground.height() {
                for x in 0..carve.ground.width() {
                    let local_pos = GridPos { x, y };
                    if let Some(tile) = carve.ground.get(local_pos) {
                        ground.set_tile(pos + local_pos, *tile, false);
                    }
                }
            }
            dirty_rect.left = dirty_rect.left.min(pos.x);
            dirty_rect.right = dirty_rect.right.max(pos.x + carve.ground.width());
            dirty_rect.bottom = dirty_rect.bottom.min(pos.y);
            dirty_rect.top = dirty_rect.top.max(pos.y + carve.ground.height());
        }

        carve.ground_pos = Some(GridPos {
            x: bounds.left,
            y: bounds.bottom,
        });
        carve
            .ground
            .reset(bounds.right - bounds.left, bounds.top - bounds.bottom, None);

        let mat = transform.compute_matrix().inverse();
        for y in bounds.bottom..bounds.top {
            for x in bounds.left..bounds.right {
                let sample = GridPos { x, y };
                let inside = {
                    let local = mat.transform_point3(Vec3::new(x as f32 + 0.5, 0., y as f32 + 0.5));
                    local.x >= -ext_x && local.x <= ext_x && local.z >= -ext_z && local.z <= ext_z
                };
                if inside {
                    let tile = ground.get_tile_ref(sample);
                    if let Some(tile) = tile {
                        ground.clear_tile(sample, false);
                        carve.ground.insert(
                            GridPos {
                                x: x - bounds.left,
                                y: y - bounds.bottom,
                            },
                            tile,
                        );
                    }
                }
            }
        }

        ground.add_dirty_rect(dirty_rect);
    }
}
