use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    transform::TransformSystem,
};
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use crate::{camera::MainCamera, mesh::cone::Cone};

use super::basic_materials::BasicMaterials;

pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransformGizmoMeshes>()
            .add_event::<AddTransformGizmo>()
            .add_event::<RemoveTransformGizmo>()
            .add_systems(
                (process_gizmo_events, clean_orphan_gizmos).in_base_set(CoreSet::PreUpdate),
            )
            .add_systems((update_gizmo_state,))
            .add_system(
                sync_gizmo_to_parent
                    .in_base_set(CoreSet::PostUpdate)
                    .after(TransformSystem::TransformPropagate),
            );
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum GizmoAxis {
    X,
    Y,
    Z,
}

impl GizmoAxis {
    pub fn ray_cast_planes(&self, gtr: &GlobalTransform) -> (Vec3, Vec3) {
        match self {
            GizmoAxis::X => (gtr.up(), gtr.forward()),
            GizmoAxis::Y => (gtr.right(), gtr.forward()),
            GizmoAxis::Z => (gtr.right(), gtr.up()),
        }
    }

    pub fn axis(&self, gtr: &GlobalTransform) -> Vec3 {
        match self {
            GizmoAxis::X => gtr.right(),
            GizmoAxis::Y => gtr.up(),
            GizmoAxis::Z => gtr.back(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GizmoPlane {
    XY,
    YZ,
    ZX,
    Camera,
}

impl GizmoPlane {
    pub fn ray_cast_plane(&self, gtr: &GlobalTransform, ray: &Ray) -> Vec3 {
        match self {
            GizmoPlane::XY => gtr.back(),
            GizmoPlane::YZ => gtr.right(),
            GizmoPlane::ZX => gtr.up(),
            GizmoPlane::Camera => ray.direction,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GizmoConstraint {
    Axis(GizmoAxis),
    Plane(GizmoPlane),
}

impl GizmoConstraint {
    fn ray_cast(&self, ray: &Ray, gtr: &GlobalTransform, rapier: &RapierContext) -> Option<Vec3> {
        let origin = ray.origin;
        let ray_p = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());
        match self {
            GizmoConstraint::Axis(axis) => {
                let (plane1, plane2) = axis.ray_cast_planes(gtr);
                let toi0 = ray_toi_with_halfspace(&origin.into(), &plane1.into(), &ray_p)?;
                let toi1 = ray_toi_with_halfspace(&origin.into(), &plane2.into(), &ray_p)?;
                let dir = axis.axis(gtr);
                let y0 = dir.dot(origin + toi0 * ray.direction);
                let y1 = dir.dot(origin + toi1 * ray.direction);
                let drag_y = (y0 + y1) / 2.;
                Some(drag_y * dir)
            }
            GizmoConstraint::Plane(plane) => {
                let ray_plane = plane.ray_cast_plane(gtr, ray);
                let toi = ray_toi_with_halfspace(&origin.into(), &ray_plane.into(), &ray_p)?;
                Some(toi * ray_plane)
            }
        }
    }
}

struct GizmoActiveState {
    start_drag: Vec3,
    constraint: GizmoConstraint,
}

#[derive(Component)]
struct TransformGizmo {
    entity: Entity,
    active: Option<GizmoActiveState>,
}

#[derive(Component)]
struct TransformGizmoPart {
    material: Handle<StandardMaterial>,
    highlighted: bool,
    constraint: GizmoConstraint,
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
                    active: None,
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
                        constraint: GizmoConstraint::Plane(GizmoPlane::Camera),
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

fn clean_orphan_gizmos(q_gizmo: Query<(Entity, &TransformGizmo)>, mut cmd: Commands) {
    for (gizmo_ent, gizmo) in &q_gizmo {
        if cmd.get_entity(gizmo.entity).is_none() {
            cmd.entity(gizmo_ent).despawn_recursive();
        }
    }
}

fn update_gizmo_state(
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    materials: Res<BasicMaterials>,
    q_camera: Query<&MainCamera>,
    mut q_gizmo: Query<(&mut TransformGizmo, &GlobalTransform)>,
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
        let Ok((mut gizmo, gizmo_gtr)) = q_gizmo.get_mut(parent.get()) else { continue };
        if gizmo.active.is_none() {
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
                }
            } else {
                if gizmo_part.highlighted {
                    gizmo_part.highlighted = false;
                    *material = gizmo_part.material.clone();
                }
            }
            if gizmo_part.highlighted && mouse.just_pressed(MouseButton::Left) {
                if let Some(start_drag) = gizmo_part
                    .constraint
                    .ray_cast(&mouse_ray, gizmo_gtr, &rapier)
                {
                    gizmo.active = Some(GizmoActiveState {
                        start_drag,
                        constraint: gizmo_part.constraint,
                    });
                }
            }
        } else if !mouse.pressed(MouseButton::Left) {
            gizmo.active = None;
            gizmo_part.highlighted = false;
            *material = gizmo_part.material.clone();
        }
    }
}
