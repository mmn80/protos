use bevy::{prelude::*, render::primitives::Aabb};

use super::{
    ground::{Ground, GroundMaterialRef},
    sparse_grid::{GridPos, SparseGrid},
};

pub struct SlowUnitPlugin;

impl Plugin for SlowUnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_to_stage(CoreStage::PreUpdate, update_nav_grid);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
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
            transform: Transform::from_rotation(Quat::from_rotation_y(2.)),
            ..Default::default()
        })
        .insert(Name::new("Building"))
        .insert(NavGridCarve::default());
}

#[derive(Clone, Component, Debug)]
pub struct NavGridCarve {
    ground_pos: Option<GridPos>,
    ground: SparseGrid<GroundMaterialRef>,
}

impl Default for NavGridCarve {
    fn default() -> Self {
        Self {
            ground_pos: None,
            ground: SparseGrid::new(1, 1, None),
        }
    }
}

fn update_nav_grid(
    mut ground: ResMut<Ground>,
    mut query: Query<
        (&Transform, &Aabb, &mut NavGridCarve),
        Or<(
            Added<Transform>,
            Added<Aabb>,
            Changed<Transform>,
            Changed<Aabb>,
        )>,
    >,
) {
    for (transform, aabb, mut carve) in query.iter_mut() {
        let (ext_x, ext_z) = (aabb.half_extents.x, aabb.half_extents.z);

        let bot_l = transform.mul_vec3(aabb.center + Vec3::new(-ext_x, 0., -ext_z));
        let bot_r = transform.mul_vec3(aabb.center + Vec3::new(-ext_x, 0., ext_z));
        let top_l = transform.mul_vec3(aabb.center + Vec3::new(ext_x, 0., ext_z));
        let top_r = transform.mul_vec3(aabb.center + Vec3::new(ext_x, 0., -ext_z));

        let x_min = bot_l.x.min(bot_r.x).min(top_l.x).min(top_r.x).floor();
        let x_max = bot_l.x.max(bot_r.x).max(top_l.x).max(top_r.x).ceil();
        let z_min = bot_l.z.min(bot_r.z).min(top_l.z).min(top_r.z).floor();
        let z_max = bot_l.z.max(bot_r.z).max(top_l.z).max(top_r.z).ceil();

        let mut dirty_rect = Rect {
            left: x_min,
            right: x_max,
            top: z_max,
            bottom: z_min,
        };

        if let Some(pos) = carve.ground_pos {
            for y in 0..carve.ground.height() {
                for x in 0..carve.ground.width() {
                    let ground_pos = Vec3::new((x + pos.x) as f32, 0., (y + pos.y) as f32);
                    if let Some(tile) = carve.ground.get(GridPos { x, y }) {
                        ground.set_tile(ground_pos, *tile, false);
                    }
                }
            }
            let x_min_old = pos.x as f32;
            if x_min_old < x_min {
                dirty_rect.left = x_min_old;
            }
            let x_max_old = (pos.x as f32) + (carve.ground.width() as f32);
            if x_max_old > x_max {
                dirty_rect.right = x_max_old;
            }
            let z_min_old = pos.y as f32;
            if z_min_old < z_min {
                dirty_rect.bottom = z_min_old;
            }
            let z_max_old = (pos.y as f32) + (carve.ground.height() as f32);
            if z_max_old > z_max {
                dirty_rect.top = z_max_old;
            }
        }

        carve.ground_pos = Some(GridPos {
            x: x_min as u32,
            y: z_min as u32,
        });
        carve
            .ground
            .reset((x_max - x_min) as u32, (z_max - z_min) as u32, None);

        let mat = transform.compute_matrix().inverse();
        let mut x = 0.5 + x_min;
        let mut z = 0.5 + z_min;
        while z < z_max {
            let sample = Vec3::new(x, 0., z);
            let local = mat.transform_vector3(sample);
            let inside =
                local.x >= -ext_x && local.x <= ext_x && local.z >= -ext_z && local.z <= ext_z;
            if inside {
                let tile = ground.get_tile_ref(sample).unwrap();
                ground.clear_tile(sample, false);
                carve.ground.insert(
                    GridPos {
                        x: (x - x_min).floor() as u32,
                        y: (z - z_min).floor() as u32,
                    },
                    tile,
                );
            }
            x += 1.;
            if x > x_max {
                z += 1.;
                x = 0.5 + x_min;
            }
        }

        ground.add_dirty_rect_f32(dirty_rect);
    }
}
