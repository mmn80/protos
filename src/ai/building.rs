use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuildingsRes::default())
            .add_startup_system(setup_buildings);
    }
}

#[derive(Resource)]
struct BuildingsRes {
    pub materials: Option<BuildingsMaterials>,
}

struct BuildingsMaterials {
    pub ui_mat: Handle<StandardMaterial>,
    pub floor_mat: Handle<StandardMaterial>,
    pub wall_mat: Handle<StandardMaterial>,
    pub stairs_mat: Handle<StandardMaterial>,
}

impl Default for BuildingsRes {
    fn default() -> Self {
        Self { materials: None }
    }
}

/// The root of a building structure. Contains some floors.
///
/// Pathfinding in a building is hierarchical. First a generic path is found in the graph of rooms & doors,
/// then, optionally, explicit paths in each room's nav mesh.
#[derive(Component)]
struct Building;

/// Top level element of buildings. Contains purely physical elements such as floor tiles & walls,
/// and navigation elements, such as rooms & doors.
///
/// Can be resized, leading to cascaded resizing of anchored floor tiles & walls, & optionally floor(s) above.
/// Can be extended by extruding from a selected section, leading to generating new external walls & optionally floor(s) above.
#[derive(Component)]
struct Floor;

/// Navigable element of floors (anchored). Has collider(s).
#[derive(Component)]
struct FloorTile;

/// Blocking element of floors (anchored). Containes wall tiles.
///
/// May also be anchored to the floor above & become tilted.
#[derive(Component)]
struct Wall;

/// Element of walls (anchored). Has collider(s).
#[derive(Component)]
struct WallTile;

/// Navigation element. Has nav mesh(es). Origin at center of main entrance. Contains furniture.
#[derive(Component)]
struct Room;

/// Navigation element connecting 2 rooms (nav meshes), or 1 room & outside. Origin at center of door.
///
/// Contains door furniture. Anchored to wall tiles.
/// Adding a door splits a wall tile into 3: one above, one to the left & one to the right.
#[derive(Component)]
struct Door {
    /// If this is None, then it's a door leading to outside the building.
    pub outside: Option<Entity>,
    /// A room entity.
    pub inside: Entity,
    /// Used by pathfinder to know if the agent fits.
    pub width: f32,
    /// Used by pathfinder to know if the agent fits.
    pub height: f32,
    /// Detected based on raycasts (an object might also block the path, not just the door being closed).
    pub opened: bool,
}

/// Special type of floor laied onto the ground.
#[derive(Component)]
struct Foundation;

/// Special type of floor. May have no rooms or doors.
#[derive(Component)]
struct Roof;

/// A special type of room. Entrance to a floor.
/// Contains a door leading to another floor's room, or outside.
#[derive(Component)]
struct Stairs;

fn setup_buildings(mut res: ResMut<BuildingsRes>, mut materials: ResMut<Assets<StandardMaterial>>) {
    res.materials = Some(BuildingsMaterials {
        ui_mat: materials.add(StandardMaterial {
            base_color: Color::rgba(0.5, 0.9, 0.5, 0.4),
            emissive: Color::rgb(0.5, 0.9, 0.5),
            metallic: 0.9,
            perceptual_roughness: 0.8,
            reflectance: 0.8,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
        floor_mat: materials.add(StandardMaterial {
            base_color: Color::SILVER,
            metallic: 0.2,
            perceptual_roughness: 0.8,
            reflectance: 0.4,
            ..default()
        }),
        wall_mat: materials.add(StandardMaterial {
            base_color: Color::OLIVE,
            metallic: 0.2,
            perceptual_roughness: 0.8,
            reflectance: 0.2,
            ..default()
        }),
        stairs_mat: materials.add(StandardMaterial {
            base_color: Color::SILVER,
            metallic: 0.2,
            perceptual_roughness: 0.8,
            reflectance: 0.4,
            ..default()
        }),
    });
}
