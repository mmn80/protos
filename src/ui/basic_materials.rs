use bevy::prelude::*;

pub struct BasicMaterialsPlugin;

impl Plugin for BasicMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BasicMaterialsRes::default())
            .add_startup_system(setup_basic_materials.label(SetupBasicMaterialsSystem));
    }
}

#[derive(SystemLabel)]
pub struct SetupBasicMaterialsSystem;

#[derive(Resource, Default)]
pub struct BasicMaterialsRes {
    pub ui_red: Option<Handle<StandardMaterial>>,
    pub ui_green: Option<Handle<StandardMaterial>>,
    pub ui_blue: Option<Handle<StandardMaterial>>,
    pub ui_selected: Option<Handle<StandardMaterial>>,
    pub ui_transparent: Option<Handle<StandardMaterial>>,
    pub terrain: Option<Handle<StandardMaterial>>,
    pub salmon: Option<Handle<StandardMaterial>>,
    pub gold: Option<Handle<StandardMaterial>>,
}

fn setup_basic_materials(
    mut res: ResMut<BasicMaterialsRes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    res.ui_red = Some(materials.add(StandardMaterial {
        base_color: Color::rgb(0.9, 0.5, 0.5),
        emissive: Color::rgb(0.9, 0.5, 0.5),
        metallic: 0.5,
        perceptual_roughness: 0.8,
        reflectance: 0.5,
        ..default()
    }));
    res.ui_green = Some(materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.9, 0.5),
        emissive: Color::rgb(0.5, 0.9, 0.5),
        metallic: 0.5,
        perceptual_roughness: 0.8,
        reflectance: 0.5,
        ..default()
    }));
    res.ui_blue = Some(materials.add(StandardMaterial {
        base_color: Color::rgb(0.5, 0.5, 0.9),
        emissive: Color::rgb(0.5, 0.5, 0.9),
        metallic: 0.5,
        perceptual_roughness: 0.8,
        reflectance: 0.5,
        ..default()
    }));
    res.ui_selected = Some(materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        emissive: Color::rgb(1.0, 1.0, 1.0),
        metallic: 0.8,
        perceptual_roughness: 0.5,
        reflectance: 0.5,
        ..default()
    }));
    res.ui_transparent = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.9, 0.5, 0.4),
        emissive: Color::rgb(0.5, 0.9, 0.5),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }));
    res.terrain = Some(materials.add(StandardMaterial {
        base_color: Color::SILVER,
        metallic: 0.2,
        perceptual_roughness: 0.8,
        reflectance: 0.2,
        ..default()
    }));
    res.salmon = Some(materials.add(StandardMaterial {
        base_color: Color::SALMON,
        metallic: 0.2,
        perceptual_roughness: 0.8,
        reflectance: 0.5,
        ..default()
    }));
    res.gold = Some(materials.add(StandardMaterial {
        base_color: Color::GOLD,
        metallic: 0.8,
        perceptual_roughness: 0.4,
        reflectance: 0.5,
        ..default()
    }));
}
