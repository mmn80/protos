use bevy::prelude::*;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Buildings>()
            .init_resource::<Buildings>();
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Buildings {}

impl Default for Buildings {
    fn default() -> Self {
        Self {}
    }
}

/// The root of a building structure. Contains some floors.
///
/// Pathfinding in a building is hierarchical. First a generic path is found in the graph of rooms & doors,
/// then, optionally, explicit paths in each room's nav mesh.
#[derive(Component)]
pub struct Building;

/// Top level element of buildings. Contains purely physical elements such as floor tiles & walls,
/// and navigation elements, such as rooms & doors.
///
/// Can be resized, leading to cascaded resizing of anchored floor tiles & walls, & optionally floor(s) above.
/// Can be extended by extruding from a selected section, leading to generating new external walls & optionally floor(s) above.
#[derive(Component)]
pub struct Floor;

/// Navigable element of floors (anchored). Has collider(s).
#[derive(Component)]
pub struct FloorTile;

/// Blocking element of floors (anchored). Containes wall tiles.
///
/// May also be anchored to the floor above & become tilted.
#[derive(Component)]
pub struct Wall;

/// Element of walls (anchored). Has collider(s).
#[derive(Component)]
pub struct WallTile;

/// Navigation element. Has nav mesh(es). Origin at center of main entrance. Contains furniture.
#[derive(Component)]
pub struct Room;

/// Navigation element connecting 2 rooms (nav meshes), or 1 room & outside. Origin at center of door.
///
/// Contains door furniture. Anchored to wall tiles.
/// Adding a door splits a wall tile into 3: one above, one to the left & one to the right.
#[derive(Component)]
pub struct Door {
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
pub struct Foundation;

/// Special type of floor. May have no rooms or doors.
#[derive(Component)]
pub struct Roof;

/// A special type of room. Entrance to a floor (for people).
///
/// Contains a door leading to another floor's room, or outside.
#[derive(Component)]
pub struct Stairs;

/// A special type of room. Entrance to a floor (for vehicles).
///
/// Contains a door leading to another floor's room, or outside.
#[derive(Component)]
pub struct Ramp;
