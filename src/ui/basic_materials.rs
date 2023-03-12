use bevy::prelude::*;

pub struct BasicMaterialsPlugin;

impl Plugin for BasicMaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BasicMaterials>()
            .add_system(flip_materials);
    }
}

#[derive(Resource, Reflect)]
pub struct BasicMaterials {
    pub ui_default: Handle<StandardMaterial>,
    pub ui_red: Handle<StandardMaterial>,
    pub ui_green: Handle<StandardMaterial>,
    pub ui_blue: Handle<StandardMaterial>,
    pub ui_selected: Handle<StandardMaterial>,
    pub ui_transparent: Handle<StandardMaterial>,
    pub terrain: Handle<StandardMaterial>,
    pub salmon: Handle<StandardMaterial>,
    pub gold: Handle<StandardMaterial>,
}

impl FromWorld for BasicMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();

        BasicMaterials {
            ui_default: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.5, 0.5),
                emissive: Color::rgb(0.5, 0.5, 0.5),
                metallic: 0.0,
                perceptual_roughness: 0.5,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            ui_red: materials.add(StandardMaterial {
                base_color: Color::rgb(0.9, 0.5, 0.5),
                emissive: Color::rgb(0.9, 0.5, 0.5),
                metallic: 0.0,
                perceptual_roughness: 0.5,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            ui_green: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.9, 0.5),
                emissive: Color::rgb(0.5, 0.9, 0.5),
                metallic: 0.0,
                perceptual_roughness: 0.5,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            ui_blue: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.5, 0.9),
                emissive: Color::rgb(0.5, 0.5, 0.9),
                metallic: 0.0,
                perceptual_roughness: 0.5,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            ui_selected: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0),
                emissive: Color::rgb(1.0, 1.0, 1.0),
                metallic: 1.0,
                perceptual_roughness: 0.5,
                double_sided: true,
                cull_mode: None,
                ..default()
            }),
            ui_transparent: materials.add(StandardMaterial {
                base_color: Color::rgba(0.5, 0.9, 0.5, 0.5),
                emissive: Color::rgb(0.5, 0.9, 0.5),
                metallic: 1.0,
                perceptual_roughness: 0.8,
                reflectance: 0.8,
                alpha_mode: AlphaMode::Add,
                ..default()
            }),
            terrain: materials.add(StandardMaterial {
                base_color: Color::SILVER,
                metallic: 0.0,
                perceptual_roughness: 0.8,
                reflectance: 0.2,
                ..default()
            }),
            salmon: materials.add(StandardMaterial {
                base_color: Color::SALMON,
                metallic: 0.0,
                perceptual_roughness: 0.5,
                ..default()
            }),
            gold: materials.add(StandardMaterial {
                base_color: Color::GOLD,
                metallic: 1.0,
                perceptual_roughness: 0.5,
                ..default()
            }),
        }
    }
}

#[derive(Component)]
pub struct FlipMaterial {
    material: Option<Handle<StandardMaterial>>,
    original_material: Option<Handle<StandardMaterial>>,
}

impl FlipMaterial {
    pub fn new(material: &Handle<StandardMaterial>) -> Self {
        Self {
            material: Some(material.clone()),
            original_material: None,
        }
    }
}

#[derive(Component)]
pub struct RevertFlipMaterial;

fn flip_materials(
    mut q_material: Query<(
        Entity,
        &mut Handle<StandardMaterial>,
        &mut FlipMaterial,
        Option<&RevertFlipMaterial>,
    )>,
    q_bogus_reverts: Query<Entity, (With<RevertFlipMaterial>, Without<FlipMaterial>)>,
    mut cmd: Commands,
) {
    for (entity, mut material, mut flip, revert) in &mut q_material {
        if revert.is_some() {
            if let Some(original_material) = flip.original_material.take() {
                *material = original_material;

                cmd.entity(entity).remove::<FlipMaterial>();
                cmd.entity(entity).remove::<RevertFlipMaterial>();
            }
        } else if let Some(flip_material) = flip.material.take() {
            flip.original_material = Some(material.clone());
            *material = flip_material;
        }
    }
    for entity in &q_bogus_reverts {
        cmd.entity(entity).remove::<RevertFlipMaterial>();
    }
}
