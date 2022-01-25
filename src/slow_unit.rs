use bevy::prelude::*;

use crate::sparse_grid::SparseGrid;

pub struct SlowUnitPlugin;

impl Plugin for SlowUnitPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Ground::new(1024))
            .add_startup_system(setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ground: ResMut<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sz = ground.tiles.width as f32 / 2.;
    ground.entity = Some(
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box {
                    min_x: -sz,
                    max_x: sz,
                    min_y: -5.,
                    max_y: 0.,
                    min_z: -sz,
                    max_z: sz,
                })),
                material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                ..Default::default()
            })
            .insert(Name::new("Ground"))
            .id(),
    );
}

#[derive(Debug, Clone, Default)]
pub struct GroundTile {
    pub color: Color,
    pub nav_cost: u8,
}

#[derive(Debug, Clone)]
pub struct Ground {
    pub entity: Option<Entity>,
    pub tiles: SparseGrid<GroundTile>,
}

impl Ground {
    pub fn new(width: u32) -> Self {
        Self {
            entity: None,
            tiles: SparseGrid::new(
                width,
                Some(GroundTile {
                    color: Color::rgb(0.3, 0.5, 0.3),
                    nav_cost: 32,
                }),
            ),
        }
    }
}
