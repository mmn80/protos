use std::f32::consts::PI;

use bevy::{ecs::schedule::ShouldRun, prelude::*, render::primitives::Aabb};
use bevy_mod_raycast::{RayCastMesh, RayCastSource};
use big_brain::prelude::*;
use rand::{thread_rng, Rng};

use super::{
    fast_unit::{Drunk, Idle, RandomMove, Velocity},
    ground::{Ground, GroundMaterialRef, GroundRaycastSet},
    sparse_grid::{GridPos, SparseGrid},
};
use crate::{
    camera::ScreenPosition,
    ui::{selection::Selectable, side_panel::SidePanelState},
};

pub struct SlowUnitPlugin;

impl Plugin for SlowUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.after("ground_setup"))
            .add_system_to_stage(CoreStage::PreUpdate, update_nav_grid)
            .add_system(spawn_building.with_run_criteria(building_spawning));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    ground: Res<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = thread_rng();
    for _ in 1..100 {
        let is_static = rng.gen_bool(0.8);
        let size = if is_static {
            Vec3::new(
                rng.gen_range(5.0..50.0),
                rng.gen_range(2.0..15.0),
                rng.gen_range(5.0..50.0),
            )
        } else {
            Vec3::new(
                rng.gen_range(5.0..20.0),
                rng.gen_range(2.0..15.0),
                rng.gen_range(5.0..20.0),
            )
        };
        spawn(
            &mut commands,
            &mut meshes,
            &ground,
            &mut materials,
            size,
            Vec2::new(rng.gen_range(124.0..900.0), rng.gen_range(124.0..900.0)),
            rng.gen_range(0.0..2. * PI),
            is_static,
        );
    }
}

fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    ground: &Ground,
    materials: &mut Assets<StandardMaterial>,
    size: Vec3,
    position: Vec2,
    rotation: f32,
    is_static: bool,
) {
    let mut rng = thread_rng();
    let material = materials.add(
        Color::rgb(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        )
        .into(),
    );
    let tower_size = size.x.min(size.z) / 10.;
    let tower_id = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(if is_static {
                Mesh::from(shape::Box {
                    min_x: -tower_size,
                    max_x: tower_size,
                    min_y: -tower_size,
                    max_y: tower_size,
                    min_z: -tower_size,
                    max_z: tower_size,
                })
            } else {
                Mesh::from(shape::Icosphere {
                    radius: tower_size,
                    subdivisions: 4,
                })
            }),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.,
                size.y + tower_size,
                tower_size + -size.z / 2.,
            )),
            ..Default::default()
        })
        .id();
    let bld_id = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -size.x / 2.,
                max_x: size.x / 2.,
                min_y: 0.,
                max_y: size.y,
                min_z: -size.z / 2.,
                max_z: size.z / 2.,
            })),
            material,
            transform: Transform::from_rotation(Quat::from_rotation_y(rotation))
                .with_translation(Vec3::new(position.x, 0., position.y)),
            ..Default::default()
        })
        .insert(Name::new("Building"))
        .insert(NavGridCarve::default())
        .insert(ScreenPosition::default())
        .insert(Selectable)
        .add_child(tower_id)
        .id();
    if !is_static {
        commands
            .entity(bld_id)
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
            );
    }
    if let Some(ground_ent) = ground.entity {
        commands.entity(ground_ent).add_child(bld_id);
    } else {
        warn!("NO GROUND!!");
    }
}

fn building_spawning(
    ui: Res<SidePanelState>,
    input_mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
) -> ShouldRun {
    if ui.spawn_building
        && input_mouse.just_pressed(MouseButton::Left)
        && keyboard.pressed(KeyCode::LControl)
    {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn spawn_building(
    keyboard: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    ground: Res<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    source_query: Query<&RayCastSource<GroundRaycastSet>>,
    target_query: Query<&Transform, With<RayCastMesh<GroundRaycastSet>>>,
) {
    if let Ok(ground_transform) = target_query.get_single() {
        let mat = ground_transform.compute_matrix().inverse();
        let mut rng = thread_rng();

        for source in source_query.iter() {
            if let Some(intersections) = source.intersect_list() {
                if intersections.len() > 1 {
                    info!("more then 1 intersection!");
                }
                for (entity, intersection) in intersections {
                    if *entity == ground.entity.unwrap() {
                        let center = mat.project_point3(intersection.position());
                        // info!("spawn building at: {:?}", center);
                        spawn(
                            &mut commands,
                            &mut meshes,
                            &ground,
                            &mut materials,
                            Vec3::new(
                                rng.gen_range(5.0..20.0),
                                rng.gen_range(2.0..15.0),
                                rng.gen_range(5.0..20.0),
                            ),
                            Vec2::new(center.x, center.z),
                            rng.gen_range(0.0..2. * PI),
                            keyboard.pressed(KeyCode::LShift),
                        );
                        break;
                    }
                }
            }
        }
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
