use bevy::prelude::*;

pub struct BasicMaterialsPlugin;

impl Plugin for BasicMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BasicMaterialsRes>();
    }
}

#[derive(Resource)]
pub struct BasicMaterialsRes {
    pub ui_red: Handle<StandardMaterial>,
    pub ui_green: Handle<StandardMaterial>,
    pub ui_blue: Handle<StandardMaterial>,
    pub ui_selected: Handle<StandardMaterial>,
    pub ui_transparent: Handle<StandardMaterial>,
    pub terrain: Handle<StandardMaterial>,
    pub salmon: Handle<StandardMaterial>,
    pub gold: Handle<StandardMaterial>,
}

impl FromWorld for BasicMaterialsRes {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        BasicMaterialsRes {
            ui_red: materials.add(StandardMaterial {
                base_color: Color::rgb(0.9, 0.5, 0.5),
                emissive: Color::rgb(0.9, 0.5, 0.5),
                metallic: 0.5,
                perceptual_roughness: 0.8,
                reflectance: 0.5,
                ..default()
            }),
            ui_green: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.9, 0.5),
                emissive: Color::rgb(0.5, 0.9, 0.5),
                metallic: 0.5,
                perceptual_roughness: 0.8,
                reflectance: 0.5,
                ..default()
            }),
            ui_blue: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.5, 0.9),
                emissive: Color::rgb(0.5, 0.5, 0.9),
                metallic: 0.5,
                perceptual_roughness: 0.8,
                reflectance: 0.5,
                ..default()
            }),
            ui_selected: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0),
                emissive: Color::rgb(1.0, 1.0, 1.0),
                metallic: 0.8,
                perceptual_roughness: 0.5,
                reflectance: 0.5,
                ..default()
            }),
            ui_transparent: materials.add(StandardMaterial {
                base_color: Color::rgba(0.5, 0.9, 0.5, 0.4),
                emissive: Color::rgb(0.5, 0.9, 0.5),
                metallic: 0.9,
                perceptual_roughness: 0.8,
                reflectance: 0.8,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            terrain: materials.add(StandardMaterial {
                base_color: Color::SILVER,
                metallic: 0.2,
                perceptual_roughness: 0.8,
                reflectance: 0.2,
                ..default()
            }),
            salmon: materials.add(StandardMaterial {
                base_color: Color::SALMON,
                metallic: 0.2,
                perceptual_roughness: 0.8,
                reflectance: 0.5,
                ..default()
            }),
            gold: materials.add(StandardMaterial {
                base_color: Color::GOLD,
                metallic: 0.8,
                perceptual_roughness: 0.4,
                reflectance: 0.5,
                ..default()
            }),
        }
    }
}
