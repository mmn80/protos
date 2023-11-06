use bevy::{prelude::*, render::mesh::skinning::SkinnedMesh};
use bevy_xpbd_3d::prelude::*;

use crate::{
    ai::terrain::Terrain,
    camera::{MainCamera, ScreenPosition},
    ui::{
        basic_materials::BasicMaterials,
        selection::{Layer, Selectable, Selected},
        side_panel::{SidePanel, UiMode},
    },
};

pub struct FoxPlugin;

impl Plugin for FoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_fox).add_systems(
            Update,
            (add_fox, init_fox, start_move_fox, move_fox, shoot_balls),
        );
    }
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

#[derive(Component, Reflect)]
struct Fox {
    animator: Option<Entity>,
}

fn setup_fox(asset_server: Res<AssetServer>, mut cmd: Commands) {
    cmd.insert_resource(Animations(vec![
        asset_server.load("models/Fox.glb#Animation0"),
        asset_server.load("models/Fox.glb#Animation1"),
        asset_server.load("models/Fox.glb#Animation2"),
    ]));
}

fn add_fox(
    mouse: Res<Input<MouseButton>>,
    asset_server: Res<AssetServer>,
    spatial_query: SpatialQuery,
    panel: Res<SidePanel>,
    terrain: Res<Terrain>,
    q_camera: Query<&MainCamera>,
    mut cmd: Commands,
) {
    if panel.mode != UiMode::AddFox || panel.mouse_over || !mouse.just_pressed(MouseButton::Left) {
        return;
    };

    let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else {
        return;
    };
    let Some(ground) = terrain.ground else { return };
    let Some(hit) = spatial_query.cast_ray(
        ray.origin,
        ray.direction,
        1000.,
        false,
        SpatialQueryFilter::new().with_masks([Layer::Object]),
    ) else {
        return;
    };
    if hit.entity == ground {
        let pos = ray.origin + hit.time_of_impact * ray.direction;
        let dir_z = Vec3::new(ray.direction.x, 0., ray.direction.z).normalize();
        let dir_y = Vec3::Y;
        let rot = Quat::from_mat3(&Mat3::from_cols(
            dir_y.cross(dir_z).normalize(),
            dir_y,
            dir_z,
        ));
        let pos = pos + 0.5 * dir_y;
        let fox = cmd
            .spawn((
                Fox { animator: None },
                SpatialBundle {
                    transform: Transform::from_translation(pos).with_rotation(rot),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                ScreenPosition::default(),
                RigidBody::Kinematic,
                Collider::ball(0.5),
                CollisionLayers::new([Layer::Object], [Layer::Object]),
            ))
            .id();
        cmd.entity(fox)
            .insert((
                Name::new(format!("Fox ({fox:?})")),
                Selectable::new(fox, None),
            ))
            .with_children(|parent| {
                parent.spawn(SceneBundle {
                    transform: Transform::from_translation(-0.5 * Vec3::Y)
                        .with_scale(Vec3::splat(0.01)),
                    scene: asset_server.load("models/Fox.glb#Scene0"),
                    ..default()
                });
            });
    } else {
        info!("not ground: {:?}", hit.entity);
    }
}

fn init_fox(
    anims: Res<Animations>,
    mut q_player: Query<(Entity, &mut AnimationPlayer)>,
    mut q_fox: Query<(&Children, &mut Fox, &mut Visibility)>,
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
            let (mut fox_ent, mut selectable) = (None, None);
            for parent in q_parent.iter_ancestors(entity) {
                if let Ok((children, mut fox, mut visibility)) = q_fox.get_mut(parent) {
                    *visibility = Visibility::Inherited;
                    fox.animator = Some(entity);
                    for c in children.iter() {
                        if q_selectable.contains(*c) {
                            selectable = Some(*c);
                            fox_ent = Some(parent);
                            break;
                        }
                    }
                    break;
                }
            }
            if let (Some(fox), Some(selectable)) = (fox_ent, selectable) {
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

#[derive(Component)]
struct MoveFox {
    destination: Vec3,
    speed: f32,
}

fn start_move_fox(
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    spatial_query: SpatialQuery,
    panel: Res<SidePanel>,
    terrain: Res<Terrain>,
    anims: Res<Animations>,
    q_camera: Query<&MainCamera>,
    q_fox: Query<(Entity, &Fox), With<Selected>>,
    mut q_player: Query<&mut AnimationPlayer>,
    mut cmd: Commands,
) {
    if panel.mouse_over || !mouse.just_pressed(MouseButton::Right) {
        return;
    };
    let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else {
        return;
    };
    let Some(ground) = terrain.ground else { return };
    let Some(hit) = spatial_query.cast_ray(
        ray.origin,
        ray.direction,
        1000.,
        false,
        SpatialQueryFilter::new().with_masks([Layer::Object]),
    ) else {
        return;
    };
    if hit.entity == ground {
        let destination = ray.origin + hit.time_of_impact * ray.direction + 0.5 * Vec3::Y;
        for (fox_ent, fox) in &q_fox {
            if let Some(animator) = fox.animator {
                if let Ok(mut player) = q_player.get_mut(animator) {
                    let run = keyboard.pressed(KeyCode::ShiftLeft);
                    cmd.entity(fox_ent).insert(MoveFox {
                        destination,
                        speed: if run { 2. } else { 1. },
                    });
                    player
                        .play_with_transition(
                            anims.0[if run { 2 } else { 1 }].clone_weak(),
                            std::time::Duration::from_millis(250),
                        )
                        .repeat();
                }
            }
        }
    }
}

fn move_fox(
    time: Res<Time>,
    anims: Res<Animations>,
    mut q_fox: Query<(Entity, &mut Transform, &Fox, &MoveFox)>,
    mut q_player: Query<&mut AnimationPlayer>,
    mut cmd: Commands,
) {
    for (fox_ent, mut fox_tr, fox, move_fox) in &mut q_fox {
        if (move_fox.destination - fox_tr.translation).length() < 0.1 {
            cmd.entity(fox_ent).remove::<MoveFox>();
            if let Some(animator) = fox.animator {
                if let Ok(mut player) = q_player.get_mut(animator) {
                    player.play(anims.0[0].clone_weak()).repeat();
                }
            }
        } else {
            let dir = (move_fox.destination - fox_tr.translation).normalize();
            fox_tr.translation += time.delta_seconds() * move_fox.speed * dir;
            let up = fox_tr.up();
            fox_tr.look_to(-dir, up);
        }
    }
}

#[derive(Component)]
pub struct ShootyBall;

fn shoot_balls(
    panel: Res<SidePanel>,
    materials: Res<BasicMaterials>,
    mouse: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_camera: Query<&MainCamera>,
    q_balls: Query<(Entity, &GlobalTransform), With<ShootyBall>>,
    mut cmd: Commands,
) {
    for (ball, ball_tr) in &q_balls {
        if ball_tr.translation().y < -50. {
            cmd.entity(ball).despawn_recursive();
        }
    }

    if panel.mode != UiMode::ShootBalls
        || panel.mouse_over
        || !mouse.just_pressed(MouseButton::Left)
    {
        return;
    };

    let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else {
        return;
    };
    let ball = cmd
        .spawn((
            ShootyBall,
            PbrBundle {
                transform: Transform::from_translation(ray.origin),
                mesh: meshes.add(
                    Mesh::try_from(shape::Icosphere {
                        radius: 1.,
                        subdivisions: 20,
                    })
                    .unwrap(),
                ),
                material: materials.gold.clone(),
                ..default()
            },
            ScreenPosition::default(),
            RigidBody::Dynamic,
            LinearDamping(0.),
            AngularDamping(0.),
            LinearVelocity(30. * ray.direction),
            AngularVelocity(Vec3::ZERO),
            Collider::ball(1.0),
            CollisionLayers::new([Layer::Object], [Layer::Object]),
            ColliderDensity(0.8),
            Friction {
                dynamic_coefficient: 0.8,
                static_coefficient: 0.8,
                combine_rule: CoefficientCombine::Average,
            },
            Restitution {
                coefficient: 0.3,
                combine_rule: CoefficientCombine::Average,
            },
        ))
        .id();
    cmd.entity(ball).insert((
        Selectable::new(ball, Some(ball)),
        Name::new(format!("Ball ({ball:?})")),
    ));
}
