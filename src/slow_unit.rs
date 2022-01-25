use bevy::{prelude::*, render::render_resource::Extent3d};
use bevy_mod_raycast::{
    DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource, RaycastSystem,
};

use crate::{sparse_grid::SparseGrid, ui::UiState};

pub struct SlowUnitPlugin;

impl Plugin for SlowUnitPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Ground::new(1024))
            .add_plugin(DefaultRaycastingPlugin::<GroundRaycastSet>::default())
            .add_startup_system(setup)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_raycast_with_cursor.before(RaycastSystem::BuildRays),
            )
            .add_system(ground_painter.label("ground_painter_system"))
            .add_system(update_ground_texture.after("ground_painter_system"));
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
        let mut material = StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0));
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
            .insert(RayCastMesh::<GroundRaycastSet>::default())
            .id(),
    );
}

#[derive(Debug, Clone)]
pub struct GroundMaterial {
    pub color: Color,
    pub nav_cost: u8,
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

#[derive(PartialEq)]
pub enum GroundMaterials {
    None,
    Grass,
    Road,
}

impl Default for GroundMaterials {
    fn default() -> Self {
        GroundMaterials::None
    }
}

impl GroundMaterials {
    pub fn to_material_ref(&self) -> Option<GroundMaterialRef> {
        match &self {
            GroundMaterials::None => None,
            GroundMaterials::Grass => Some(Ground::GRASS),
            GroundMaterials::Road => Some(Ground::ROAD),
        }
    }
}

impl Ground {
    pub const GRASS: GroundMaterialRef = GroundMaterialRef(0);
    pub const ROAD: GroundMaterialRef = GroundMaterialRef(1);

    pub fn new(width: u32) -> Self {
        let grass = GroundMaterial {
            color: Color::rgb(0.3, 0.5, 0.3),
            nav_cost: 32,
        };
        let grass_nav_cost = grass.nav_cost;
        let road = GroundMaterial {
            color: Color::rgb(0.8, 0.7, 0.5),
            nav_cost: 1,
        };
        Self {
            entity: None,
            palette: vec![grass, road],
            tiles: SparseGrid::new(width, Some(Self::GRASS)),
            nav_grid: SparseGrid::new(width, Some(grass_nav_cost)),
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
        let pos = self.tiles.grid_pos(pos);
        self.tiles.insert(pos.clone(), tile.clone());
        self.nav_grid
            .insert(pos, self.palette[tile.0 as usize].nav_cost);
        self.dirty = true;
    }

    pub fn clear_tile(&mut self, pos: Vec3) {
        let pos = self.tiles.grid_pos(pos);
        self.tiles.remove(pos.clone());
        self.nav_grid.remove(pos);
        self.dirty = true;
    }
}

fn update_ground_texture(
    mut ground: ResMut<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if ground.dirty {
        if let Some(material) = materials.get_mut(ground.material.clone()) {
            if let Some(image_handle) = &material.base_color_texture {
                if let Some(image) = images.get_mut(image_handle) {
                    let start = std::time::Instant::now();
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
                    let dt = (std::time::Instant::now() - start).as_micros();
                    info!("ground texture update time: {}μs", dt);
                }
            }
        }
    }
}

pub struct GroundRaycastSet;

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut query: Query<&mut RayCastSource<GroundRaycastSet>>,
) {
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };
    for mut pick_source in &mut query.iter_mut() {
        pick_source.cast_method = RayCastMethod::Screenspace(cursor_position);
    }
}

fn ground_painter(
    ui: Res<UiState>,
    mut ground: ResMut<Ground>,
    keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
    query: Query<&RayCastSource<GroundRaycastSet>>,
) {
    if keyboard.pressed(KeyCode::LControl) && input_mouse.just_pressed(MouseButton::Left) {
        for source in query.iter() {
            if let Some(intersections) = source.intersect_list() {
                for (entity, intersection) in intersections {
                    if *entity == ground.entity.unwrap() {
                        let pos = intersection.position();
                        info!("ground paint position: {}", pos);
                        let mat = ui.ground_material.to_material_ref();
                        if let Some(mat_ref) = mat {
                            ground.set_tile(pos, mat_ref);
                        } else {
                            ground.clear_tile(pos);
                        }
                        break;
                    }
                }
            }
        }
    }
}
