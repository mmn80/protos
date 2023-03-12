use bevy::{prelude::*, render::mesh::skinning::SkinnedMesh};
use bevy_rapier3d::prelude::*;

use crate::{
    ai::terrain::Terrain,
    camera::{MainCamera, ScreenPosition},
    ui::{
        selection::Selectable,
        side_panel::{SidePanel, UiMode},
    },
};

pub struct FoxPlugin;

impl Plugin for FoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_bot)
            .add_systems((add_fox, play_animation));
    }
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

#[derive(Component, Reflect)]
struct Fox;

fn setup_bot(asset_server: Res<AssetServer>, mut cmd: Commands) {
    cmd.insert_resource(Animations(vec![
        asset_server.load("models/Fox.glb#Animation0"),
        asset_server.load("models/Fox.glb#Animation1"),
        asset_server.load("models/Fox.glb#Animation2"),
    ]));
}

fn add_fox(
    mouse: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
    rapier: Res<RapierContext>,
    panel: Res<SidePanel>,
    terrain: Res<Terrain>,
    q_camera: Query<&MainCamera>,
    mut cmd: Commands,
) {
    if panel.mode != UiMode::AddBot || panel.mouse_over || !mouse.just_pressed(MouseButton::Left) {
        return;
    };

    let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else { return };
    let Some(ground) = terrain.ground else { return };
    if let Some((hit_ent, toi)) = rapier.cast_ray(
        ray.origin,
        ray.direction,
        1000.,
        false,
        QueryFilter::new().exclude_sensors(),
    ) {
        if hit_ent == ground {
            let pos = ray.origin + toi * ray.direction;
            let dir_z = Vec3::new(ray.direction.x, 0., ray.direction.z).normalize();
            let dir_y = Vec3::Y;
            let rot = Quat::from_mat3(&Mat3::from_cols(
                dir_y.cross(dir_z).normalize(),
                dir_y,
                dir_z,
            ));
            let fox = cmd
                .spawn(SceneBundle {
                    transform: Transform::from_translation(pos)
                        .with_rotation(rot)
                        .with_scale(Vec3::splat(0.01)),
                    scene: asset_server.load("models/Fox.glb#Scene0"),
                    visibility: Visibility::Hidden,
                    ..default()
                })
                .id();
            cmd.entity(fox)
                .insert((
                    Fox,
                    Name::new(format!("Fox ({fox:?})")),
                    ScreenPosition::default(),
                    RigidBody::KinematicPositionBased,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        SpatialBundle::from_transform(Transform::from_translation(0.5 * Vec3::Y)),
                        Collider::ball(50.),
                        Selectable::new(fox, None),
                    ));
                });
        } else {
            info!("not ground: {:?}", hit_ent);
        }
    }
}

fn play_animation(
    anims: Res<Animations>,
    mut q_player: Query<(Entity, &mut AnimationPlayer)>,
    mut q_visibility: Query<(&Children, &mut Visibility), With<Fox>>,
    q_parent: Query<&Parent>,
    mut q_selectable: Query<&mut Selectable>,
    q_mesh: Query<Entity, With<SkinnedMesh>>,
    mut started: Local<Vec<Entity>>,
) {
    if anims.0.is_empty() {
        return;
    }
    for (entity, mut player) in &mut q_player {
        if !started.contains(&entity) {
            player.play(anims.0[0].clone_weak()).repeat();
            started.push(entity);
            let (mut fox, mut selectable) = (None, None);
            for parent in q_parent.iter_ancestors(entity) {
                if let Ok((children, mut visibility)) = q_visibility.get_mut(parent) {
                    *visibility = Visibility::Inherited;
                    for c in children.iter() {
                        if q_selectable.contains(*c) {
                            selectable = Some(*c);
                            fox = Some(parent);
                            break;
                        }
                    }
                    break;
                }
            }
            if let (Some(fox), Some(selectable)) = (fox, selectable) {
                if let Ok(mut selectable) = q_selectable.get_mut(selectable) {
                    for mesh in &q_mesh {
                        for parent in q_parent.iter_ancestors(entity) {
                            if parent == fox {
                                selectable.mesh = Some(mesh);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
