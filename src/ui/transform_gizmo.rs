use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    transform::TransformSystem,
};
use bevy_rapier3d::prelude::*;

use crate::{camera::MainCamera, mesh::cone::Cone};

use super::basic_materials::BasicMaterials;

pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransformGizmoMeshes>()
            .add_event::<AddTransformGizmo>()
            .add_event::<RemoveTransformGizmo>()
            .add_system(
                sync_gizmo_to_parent
                    .in_base_set(CoreSet::PostUpdate)
                    .after(TransformSystem::TransformPropagate),
            )
            .add_systems((
                process_gizmo_events,
                update_gizmo_state,
                clean_orphan_gizmos,
            ));
    }
}

pub struct AddTransformGizmo {
    pub entity: Entity,
}

pub struct RemoveTransformGizmo {
    pub entity: Entity,
}

#[derive(Component)]
pub struct HasTransformGizmo;

#[derive(Component)]
struct TransformGizmo {
    entity: Entity,
    active: bool,
}

#[derive(Component)]
struct TransformGizmoPart {
    material: Handle<StandardMaterial>,
    highlighted: bool,
}

#[derive(Resource, Reflect)]
struct TransformGizmoMeshes {
    pub bar: Handle<Mesh>,
    pub cone: Handle<Mesh>,
    pub ball: Handle<Mesh>,
}

const BAR_H: f32 = 2.0;
const BAR_W: f32 = 0.1;
const CONE_W: f32 = 0.8;
const CONE_H: f32 = 1.0;
const BALL_R: f32 = 0.5;

impl FromWorld for TransformGizmoMeshes {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        TransformGizmoMeshes {
            bar: meshes.add(
                Mesh::try_from(shape::Cylinder {
                    radius: BAR_W / 2.,
                    height: BAR_H,
                    resolution: 10,
                    segments: 1,
                })
                .unwrap(),
            ),
            cone: meshes.add(Mesh::from(Cone::new(CONE_W / 2., CONE_H, 10))),
            ball: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: BALL_R,
                    subdivisions: 20,
                })
                .unwrap(),
            ),
        }
    }
}

fn process_gizmo_events(
    meshes: Res<TransformGizmoMeshes>,
    materials: Res<BasicMaterials>,
    mut ev_add: EventReader<AddTransformGizmo>,
    mut ev_del: EventReader<RemoveTransformGizmo>,
    q_gizmos: Query<(Entity, &TransformGizmo)>,
    mut cmd: Commands,
) {
    for AddTransformGizmo { entity } in ev_add.iter() {
        if q_gizmos.iter().all(|(_, gizmo)| gizmo.entity != *entity) {
            cmd.spawn((
                SpatialBundle::default(),
                TransformGizmo {
                    entity: *entity,
                    active: false,
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    PbrBundle {
                        transform: Transform::IDENTITY,
                        mesh: meshes.ball.clone(),
                        material: materials.salmon.clone(),
                        ..default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                    Collider::ball(BALL_R),
                    Sensor,
                    TransformGizmoPart {
                        material: materials.salmon.clone(),
                        highlighted: false,
                    },
                ));
            });

            cmd.entity(*entity).insert(HasTransformGizmo);
        }
    }

    for RemoveTransformGizmo { entity } in ev_del.iter() {
        if let Some((gizmo_ent, gizmo)) = q_gizmos.iter().find(|(_, gizmo)| gizmo.entity == *entity)
        {
            cmd.entity(gizmo.entity).remove::<HasTransformGizmo>();
            cmd.entity(gizmo_ent).despawn_recursive();
        }
    }
}

fn sync_gizmo_to_parent(
    mut q_gizmos: Query<(&TransformGizmo, &mut Transform, &mut GlobalTransform), Without<Parent>>,
    mut q_gizmo_parts: Query<
        (&Transform, &mut GlobalTransform, &Parent),
        (With<TransformGizmoPart>, Without<TransformGizmo>),
    >,
    q_attach: Query<
        &GlobalTransform,
        (
            With<HasTransformGizmo>,
            Without<TransformGizmo>,
            Without<TransformGizmoPart>,
        ),
    >,
) {
    for (gizmo, mut gizmo_tr, mut gizmo_gtr) in &mut q_gizmos {
        if let Ok(parent_gtr) = q_attach.get(gizmo.entity) {
            if *gizmo_gtr != *parent_gtr {
                *gizmo_gtr = *parent_gtr;
                *gizmo_tr = gizmo_gtr.compute_transform();
            }
        }
    }
    for (part_tr, mut part_gtr, parent) in &mut q_gizmo_parts {
        if let Ok((_, _, gizmo_gtr)) = q_gizmos.get(parent.get()) {
            *part_gtr = gizmo_gtr.mul_transform(*part_tr);
        }
    }
}

fn update_gizmo_state(
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    materials: Res<BasicMaterials>,
    q_camera: Query<&MainCamera>,
    mut q_gizmo: Query<&mut TransformGizmo>,
    mut q_gizmo_part: Query<(
        Entity,
        &Parent,
        &mut Handle<StandardMaterial>,
        &mut TransformGizmoPart,
    )>,
) {
    let Ok(Some(mouse_ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else { return };
    let mut mouse_ray_ent = None;
    let mut mouse_ray_casted = false;
    for (gizmo_part_ent, parent, mut material, mut gizmo_part) in &mut q_gizmo_part {
        let Ok(mut gizmo) = q_gizmo.get_mut(parent.get()) else { continue };
        if !gizmo.active {
            if mouse_ray_casted == false && mouse_ray_ent.is_none() {
                if let Some((hit_ent, _)) = rapier.cast_ray(
                    mouse_ray.origin,
                    mouse_ray.direction,
                    1000.,
                    false,
                    QueryFilter::new().exclude_solids(),
                ) {
                    mouse_ray_ent = Some(hit_ent);
                }
                mouse_ray_casted = true;
            }
            if Some(gizmo_part_ent) == mouse_ray_ent {
                if !gizmo_part.highlighted {
                    gizmo_part.highlighted = true;
                    *material = materials.ui_selected.clone();

                    if mouse.just_pressed(MouseButton::Left) {
                        gizmo.active = true;
                    }
                }
            } else {
                if gizmo_part.highlighted {
                    gizmo_part.highlighted = false;
                    *material = gizmo_part.material.clone();
                }
            }
        } else if !mouse.pressed(MouseButton::Left) {
            gizmo.active = false;
            gizmo_part.highlighted = false;
            *material = gizmo_part.material.clone();
        }
    }
}

fn clean_orphan_gizmos(q_gizmo: Query<(Entity, &TransformGizmo)>, mut cmd: Commands) {
    for (gizmo_ent, gizmo) in &q_gizmo {
        if cmd.get_entity(gizmo.entity).is_none() {
            cmd.entity(gizmo_ent).despawn_recursive();
        }
    }
}
