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

// contains floors
#[derive(Component)]
struct Building;

// contains rooms & doors
#[derive(Component)]
struct Floor;

// special type of floor
#[derive(Component)]
struct Foundation;

// special type of floor
#[derive(Component)]
struct Roof;

#[derive(Component)]
struct Door {
    pub outside: Option<Entity>,
    pub inside: Entity,
    pub center: Vec3,
}

// contains floor tiles & walls; has nav mesh(es)
#[derive(Component)]
struct Room;

// a special type of room
#[derive(Component)]
struct Stairs;

// navigable element of rooms; has collider(s)
#[derive(Component)]
struct FloorTile;

// blocking element of rooms; has collider(s)
#[derive(Component)]
struct Wall;

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
