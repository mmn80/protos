use std::num::NonZeroU8;

use bevy::{prelude::*, render::render_resource::Extent3d};
use bevy_mod_raycast::{
    DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource, RaycastSystem,
};

use crate::{ai::sparse_grid::SparseGrid, ui::side_panel::SidePanelState};

use super::sparse_grid::GridPos;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Ground::new(1024, 1024))
            .add_plugin(DefaultRaycastingPlugin::<GroundRaycastSet>::default())
            .add_startup_system(setup.label("ground_setup"))
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(update_ground_raycast.before(RaycastSystem::BuildRays))
                    .with_system(ground_painter.after(RaycastSystem::UpdateRaycast)),
            )
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
    {
        let mut material = StandardMaterial::from(Color::rgb(1.0, 1.0, 1.0));
        material.base_color_texture = Some(images.add(Image::default()));
        ground.material = materials.add(material);
    }
    let width = ground.width() as i32;
    ground.add_dirty_rect(Rect {
        left: 0,
        right: width,
        top: width,
        bottom: 0,
    });
    let width = ground.width() as f32;
    ground.entity = Some(
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box {
                    min_x: 0.,
                    max_x: width,
                    min_y: -5.,
                    max_y: 0.,
                    min_z: 0.,
                    max_z: width,
                })),
                material: ground.material.clone(),
                transform: Transform::from_translation(Vec3::new(-width / 2., 0., -width / 2.)),
                ..Default::default()
            })
            .insert(Name::new("Ground"))
            .insert(RayCastMesh::<GroundRaycastSet>::default())
            //.insert(NavGrid::new())
            .id(),
    );
}

#[derive(Debug, Clone)]
pub struct GroundMaterial {
    pub color: Color,
    pub nav_cost: NonZeroU8,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GroundMaterialRef(u16);

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

#[derive(Debug, Clone)]
pub struct Ground {
    pub entity: Option<Entity>,
    palette: Vec<GroundMaterial>,
    tiles: SparseGrid<GroundMaterialRef>,
    nav_grid: SparseGrid<NonZeroU8>,
    material: Handle<StandardMaterial>,
    dirty_rects: Vec<Rect<i32>>,
}

impl Ground {
    pub const GRASS: GroundMaterialRef = GroundMaterialRef(0);
    pub const ROAD: GroundMaterialRef = GroundMaterialRef(1);

    pub fn new(width: u32, height: u32) -> Self {
        let grass = GroundMaterial {
            color: Color::rgb(0.3, 0.5, 0.3),
            nav_cost: NonZeroU8::new(32).unwrap(),
        };
        let grass_nav_cost = grass.nav_cost;
        let road = GroundMaterial {
            color: Color::rgb(0.8, 0.7, 0.5),
            nav_cost: NonZeroU8::new(1).unwrap(),
        };
        Self {
            entity: None,
            palette: vec![grass, road],
            tiles: SparseGrid::new(width, height, Some(Self::GRASS)),
            nav_grid: SparseGrid::new(width, height, Some(grass_nav_cost)),
            material: Default::default(),
            dirty_rects: Vec::new(),
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.tiles.width()
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.tiles.height()
    }

    pub fn contains(&self, pos: Vec3) -> bool {
        pos.x >= 0. && pos.x < self.width() as f32 && pos.z >= 0. && pos.z < self.height() as f32
    }

    pub fn clamp(&self, pos: Vec3, buffer: f32) -> Vec3 {
        pos.clamp(
            Vec3::ZERO,
            Vec3::new(
                self.width() as f32 - buffer,
                10.,
                self.height() as f32 - buffer,
            ),
        )
    }

    pub fn register_ground_material(&mut self, tile: GroundMaterial) -> GroundMaterialRef {
        let id = self.palette.len();
        assert!(id < u16::MAX as usize);
        self.palette.push(tile);
        GroundMaterialRef(id as u16)
    }

    pub fn nav_grid(&self) -> &SparseGrid<NonZeroU8> {
        &self.nav_grid
    }

    pub fn get_tile_ref(&self, pos: GridPos) -> Option<GroundMaterialRef> {
        self.tiles.get(pos).map(|id| *id)
    }

    pub fn get_tile_vec3(&self, pos: Vec3) -> Option<&GroundMaterial> {
        if !self.contains(pos) {
            None
        } else {
            self.tiles
                .get(pos.into())
                .map(|id| &self.palette[id.0 as usize])
        }
    }

    pub fn get_tile(&self, pos: GridPos) -> Option<&GroundMaterial> {
        self.tiles.get(pos).map(|id| &self.palette[id.0 as usize])
    }

    pub fn set_tile(&mut self, pos: GridPos, tile: GroundMaterialRef, add_dirty_pos: bool) {
        self.tiles.insert(pos, tile);
        self.nav_grid
            .insert(pos, self.palette[tile.0 as usize].nav_cost);
        if add_dirty_pos {
            self.add_dirty_pos(pos.x, pos.y);
        }
    }

    pub fn clear_tile(&mut self, pos: GridPos, add_dirty_pos: bool) {
        self.tiles.remove(pos);
        self.nav_grid.remove(pos);
        if add_dirty_pos {
            self.add_dirty_pos(pos.x, pos.y);
        }
    }

    pub fn add_dirty_rect(&mut self, rect: Rect<i32>) {
        self.dirty_rects.push(rect);
    }

    pub fn add_dirty_pos(&mut self, x: i32, y: i32) {
        self.dirty_rects.push(Rect {
            left: x,
            right: x + 1,
            top: y + 1,
            bottom: y,
        });
    }
}

fn update_ground_texture(
    mut ground: ResMut<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !ground.dirty_rects.is_empty() {
        if let Some(material) = materials.get_mut(ground.material.clone()) {
            if let Some(image_handle) = &material.base_color_texture {
                if let Some(image) = images.get_mut(image_handle) {
                    let start = std::time::Instant::now();
                    image.resize(Extent3d {
                        width: ground.width(),
                        height: ground.height(),
                        depth_or_array_layers: 1,
                    });
                    for rect in ground.dirty_rects.iter() {
                        for y in rect.bottom..rect.top {
                            for x in rect.left..rect.right {
                                let pos = GridPos { x, y };
                                let pixel = ground
                                    .tiles
                                    .get(pos)
                                    .map_or(Color::BLACK, |t| ground.palette[t.0 as usize].color)
                                    .as_rgba_f32()
                                    .map(|c| (c * 255.) as u8);
                                let idx = 4 * (y * ground.width() as i32 + x) as usize;
                                image
                                    .data
                                    .get_mut(idx..idx + 4)
                                    .map(|slice| slice.copy_from_slice(&pixel));
                            }
                        }
                    }
                    ground.dirty_rects.clear();
                    let dt = (std::time::Instant::now() - start).as_micros();
                    if dt > 1000 {
                        info!("ground texture update time: {dt}μs");
                    }
                }
            }
        }
    }
}

pub struct GroundRaycastSet;

fn update_ground_raycast(
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
    ui: Res<SidePanelState>,
    mut ground: ResMut<Ground>,
    keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
    source_query: Query<&RayCastSource<GroundRaycastSet>>,
    target_query: Query<&Transform, With<RayCastMesh<GroundRaycastSet>>>,
) {
    if keyboard.pressed(KeyCode::LAlt) && input_mouse.just_pressed(MouseButton::Left) {
        if let Ok(ground_transform) = target_query.get_single() {
            let mat = ground_transform.compute_matrix().inverse();
            for source in source_query.iter() {
                if let Some(intersections) = source.intersect_list() {
                    if intersections.len() > 1 {
                        info!("more then 1 intersection!");
                    }
                    for (entity, intersection) in intersections {
                        if *entity == ground.entity.unwrap() {
                            let center: GridPos =
                                mat.project_point3(intersection.position()).into();
                            // info!("ground paint center: {center:?}");
                            let mat = ui.ground_material.to_material_ref();
                            for y in 0..ui.ground_brush_size {
                                for x in 0..ui.ground_brush_size {
                                    let pos = GridPos {
                                        x: center.x + x as i32,
                                        y: center.y + y as i32,
                                    };
                                    if let Some(mat_ref) = mat {
                                        ground.set_tile(pos, mat_ref, true);
                                    } else {
                                        ground.clear_tile(pos, true);
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

// new API (WIP)

// #[derive(Debug, Clone)]
// pub struct TilePortalId(u8);

// #[derive(Debug, Clone)]
// pub struct GridPortalId(usize);

// #[derive(Debug, Clone)]
// pub enum NavPoint {
//     Grid {
//         grid: Entity,
//         portal: GridPortalId,
//     },
//     Tile {
//         grid: Entity,
//         tile: GridPos,
//         portal: TilePortalId,
//     },
//     Point {
//         grid: Entity,
//         point: GridPos,
//     },
// }

// #[derive(Debug, Clone, Component)]
// pub struct NavPath {
//     path: Vec<NavPoint>,
//     current: usize,
// }

// #[derive(Debug, Clone)]
// struct GridPortal {
//     dest_grid: Entity,
//     dest_portal: GridPortalId,
//     position: Vec2,
//     portal: Vec2,
//     cost: u8,
// }

// #[derive(Debug, Clone)]
// enum TileNavPoint {
//     Grid(GridPortalId),
//     Tile(TilePortalId),
//     Point(GridPos),
// }

// #[derive(Debug, Clone)]
// struct FlowFieldConfig {
//     pub agent_radius: f32,
//     pub flow_dest: TileNavPoint,
// }

// type TilePortalLink = (TileNavPoint, u8);

// #[derive(Debug, Clone)]
// enum TilePortal {
//     Grid {
//         portal: GridPortalId,
//         links: Vec<TilePortalLink>,
//     },
//     Tile {
//         side: u8,
//         start: u8,
//         end: u8,
//         links: Vec<TilePortalLink>,
//     },
//     Point {
//         point: GridPos,
//         links: Vec<TilePortalLink>,
//     },
// }

// #[derive(Debug, Clone)]
// pub struct NavGridTile {
//     position: GridPos,
//     cost: SparseGrid<NonZeroU8>,
//     height: SparseGrid<NonZeroI32>,
//     dirty_since: Option<Instant>,
//     flow: HashMap<FlowFieldConfig, SparseGrid<NonZeroU8>>,
//     portals: Vec<TilePortal>,
// }

// #[derive(Debug, Clone, Component)]
// pub struct NavGrid {
//     tiles: SparseGrid<NavGridTile>,
//     material: Handle<StandardMaterial>,
//     portals: Vec<GridPortal>,
// }

// impl NavGrid {
//     pub fn new() -> Self {
//         Self {
//             tiles: SparseGrid::new(0, 0, None),
//             material: Default::default(),
//             portals: Default::default(),
//         }
//     }
// }
