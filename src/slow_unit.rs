use bevy::{prelude::*, render::render_resource::Extent3d};

use crate::sparse_grid::SparseGrid;

pub struct SlowUnitPlugin;

impl Plugin for SlowUnitPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Ground::new(1024))
            .add_startup_system(setup)
            .add_system(update_ground_texture);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ground: ResMut<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let sz = ground.nav_grid.width as f32 / 2.;
    {
        let mut material = StandardMaterial::from(GroundMaterial::default().color);
        material.base_color_texture = Some(images.add(Image::default()));
        ground.material = materials.add(material);
    }
    ground.dirty = true;
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
                material: ground.material.clone(),
                ..Default::default()
            })
            .insert(Name::new("Ground"))
            .id(),
    );
    ground.dirty = true;
}

#[derive(Debug, Clone)]
pub struct GroundMaterial {
    pub color: Color,
    pub nav_cost: u8,
}

impl Default for GroundMaterial {
    fn default() -> Self {
        Self {
            color: Color::rgb(0.3, 0.5, 0.3),
            nav_cost: 32,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GroundMaterialRef(u16);

#[derive(Debug, Clone)]
pub struct Ground {
    pub entity: Option<Entity>,
    palette: Vec<GroundMaterial>,
    tiles: SparseGrid<GroundMaterialRef>,
    nav_grid: SparseGrid<u8>,
    material: Handle<StandardMaterial>,
    dirty: bool,
}

impl Ground {
    pub const DEFAULT_TILE: GroundMaterialRef = GroundMaterialRef(0);

    pub fn new(width: u32) -> Self {
        let default_tile = GroundMaterial::default();
        let nav_cost = default_tile.nav_cost;
        Self {
            entity: None,
            palette: vec![default_tile],
            tiles: SparseGrid::new(width, Some(Self::DEFAULT_TILE)),
            nav_grid: SparseGrid::new(width, Some(nav_cost)),
            material: Default::default(),
            dirty: true,
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.tiles.width
    }

    pub fn register_ground_material(&mut self, tile: GroundMaterial) -> GroundMaterialRef {
        let id = self.palette.len();
        assert!(id < u16::MAX as usize);
        self.palette.push(tile);
        GroundMaterialRef(id as u16)
    }

    pub fn nav_grid(&self) -> &SparseGrid<u8> {
        &self.nav_grid
    }

    pub fn get_tile_ref(&self, pos: Vec3) -> Option<GroundMaterialRef> {
        self.tiles
            .get(self.tiles.grid_pos(pos))
            .map(|id| id.clone())
    }

    pub fn get_tile(&self, pos: Vec3) -> Option<&GroundMaterial> {
        self.tiles
            .get(self.tiles.grid_pos(pos))
            .map(|id| &self.palette[id.0 as usize])
    }

    pub fn set_tile(&mut self, pos: Vec3, tile: GroundMaterialRef) {
        *self.tiles.get_mut(self.tiles.grid_pos(pos)).unwrap() = tile;
        self.dirty = true;
    }
}

fn update_ground_texture(
    mut ground: ResMut<Ground>,
    materials: Res<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if ground.dirty {
        if let Some(material) = materials.get(ground.material.clone()) {
            if let Some(image_handle) = &material.base_color_texture {
                if let Some(image) = images.get_mut(image_handle) {
                    image.resize(Extent3d {
                        width: ground.width(),
                        height: ground.width(),
                        depth_or_array_layers: 1,
                    });
                    for (pos, x, y) in ground.tiles.iter_pos() {
                        let pixel: [u8; 4] = ground
                            .tiles
                            .get(pos)
                            .map_or(Color::BLACK, |t| ground.palette[t.0 as usize].color)
                            .as_rgba_f32()
                            .map(|c| (c * 255.) as u8);
                        let idx = 4 * (y * ground.width() + x) as usize;
                        image
                            .data
                            .get_mut(idx..idx + 4)
                            .map(|slice| slice.copy_from_slice(&pixel));
                    }
                    ground.dirty = false;
                }
            }
        }
    }
}
