use bevy::{
    ecs::system::EntityCommand,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::view::RenderLayers,
    transform::TransformSystem,
    window::PrimaryWindow,
};
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use crate::{
    camera::{MainCamera, UI_CAMERA_LAYER},
    mesh::cone::Cone,
};

use super::basic_materials::BasicMaterials;

pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransformGizmoMeshes>()
            .add_system(clean_orphan_gizmos.in_base_set(CoreSet::PreUpdate))
            .add_systems((
                update_gizmo_state,
                sync_parent_to_gizmo.after(update_gizmo_state),
            ))
            .add_system(
                sync_gizmo_to_parent
                    .in_base_set(CoreSet::PostUpdate)
                    .after(TransformSystem::TransformPropagate),
            );
    }
}

pub struct AddTransformGizmo;

impl EntityCommand for AddTransformGizmo {
    fn write(self, id: Entity, world: &mut World) {
        let no_gizmo = {
            let mut q_gizmos = world.query::<(Entity, &TransformGizmo)>();
            q_gizmos.iter(world).all(|(_, gizmo)| gizmo.entity != id)
        };
        if no_gizmo {
            let (ball, bar, cone, square, cylinder) = {
                let meshes = world.resource::<TransformGizmoMeshes>();
                (
                    meshes.ball.clone(),
                    meshes.bar.clone(),
                    meshes.cone.clone(),
                    meshes.square.clone(),
                    meshes.cylinder.clone(),
                )
            };
            let (ui_default, ui_red, ui_green, ui_blue) = {
                let materials = world.resource::<BasicMaterials>();
                (
                    materials.ui_default.clone(),
                    materials.ui_red.clone(),
                    materials.ui_green.clone(),
                    materials.ui_blue.clone(),
                )
            };
            world
                .spawn((
                    SpatialBundle::default(),
                    TransformGizmo {
                        entity: id,
                        active_state: None,
                    },
                    Name::new(format!("Gizmo (@{id:?})")),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            transform: Transform::IDENTITY,
                            mesh: ball.clone(),
                            material: ui_default.clone(),
                            ..default()
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                        RenderLayers::layer(UI_CAMERA_LAYER),
                        Collider::ball(BALL_R),
                        Sensor,
                        TransformGizmoPart {
                            material: ui_default.clone(),
                            highlighted: false,
                            constraint: GizmoConstraint::Plane {
                                plane: GizmoPlane::Camera,
                                is_rotation: false,
                            },
                        },
                    ));
                    for (axis, material) in [
                        (GizmoAxis::X, ui_red.clone()),
                        (GizmoAxis::Y, ui_green.clone()),
                        (GizmoAxis::Z, ui_blue.clone()),
                    ] {
                        add_axis_gizmo(parent, bar.clone(), cone.clone(), material, axis);
                    }
                    for (plane, material) in [
                        (GizmoPlane::YZ, ui_red.clone()),
                        (GizmoPlane::ZX, ui_green.clone()),
                        (GizmoPlane::XY, ui_blue.clone()),
                    ] {
                        add_plane_gizmo(parent, square.clone(), material.clone(), plane);
                        add_rotation_gizmo(parent, cylinder.clone(), material, plane);
                    }
                });

            world.entity_mut(id).insert(HasTransformGizmo);
        }
    }
}

fn add_axis_gizmo(
    parent: &mut WorldChildBuilder,
    bar: Handle<Mesh>,
    cone: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    axis: GizmoAxis,
) {
    let (dir_y, dir_x) = axis.to_yx_axes();
    let rot = Quat::from_mat3(&Mat3::from_cols(
        dir_x,
        dir_y,
        dir_x.cross(dir_y).normalize(),
    ));

    let d = 2. * BALL_R;
    parent.spawn((
        PbrBundle {
            transform: Transform::from_translation((d + BAR_H / 2.) * dir_y).with_rotation(rot),
            mesh: bar,
            material: material.clone(),
            ..default()
        },
        NotShadowCaster,
        NotShadowReceiver,
        RenderLayers::layer(UI_CAMERA_LAYER),
        Collider::cylinder(BAR_H / 2., BAR_W),
        Sensor,
        TransformGizmoPart {
            material: material.clone(),
            highlighted: false,
            constraint: GizmoConstraint::Axis(axis),
        },
    ));

    parent.spawn((
        PbrBundle {
            transform: Transform::from_translation((d + BAR_H + CONE_H / 2.) * dir_y)
                .with_rotation(rot),
            mesh: cone,
            material: material.clone(),
            ..default()
        },
        NotShadowCaster,
        NotShadowReceiver,
        RenderLayers::layer(UI_CAMERA_LAYER),
        Collider::cone(CONE_H / 2., CONE_R),
        Sensor,
        TransformGizmoPart {
            material,
            highlighted: false,
            constraint: GizmoConstraint::Axis(axis),
        },
    ));
}

fn add_plane_gizmo(
    parent: &mut WorldChildBuilder,
    square: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    plane: GizmoPlane,
) {
    let Some((dir_y, dir_x)) = plane.to_yx_axes() else { return };
    let dir_z = dir_x.cross(dir_y).normalize();
    let rot = Quat::from_mat3(&Mat3::from_cols(dir_x, dir_y, dir_z));

    let d = 2. * BALL_R + SQUARE_H / 2.;
    parent.spawn((
        PbrBundle {
            transform: Transform::from_translation(d * dir_x + d * dir_z).with_rotation(rot),
            mesh: square,
            material: material.clone(),
            ..default()
        },
        NotShadowCaster,
        NotShadowReceiver,
        RenderLayers::layer(UI_CAMERA_LAYER),
        Collider::cuboid(SQUARE_H / 2., 0.05, SQUARE_H / 2.),
        Sensor,
        TransformGizmoPart {
            material: material.clone(),
            highlighted: false,
            constraint: GizmoConstraint::Plane {
                plane,
                is_rotation: false,
            },
        },
    ));
}

fn add_rotation_gizmo(
    parent: &mut WorldChildBuilder,
    cylinder: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    plane: GizmoPlane,
) {
    let Some((dir_y, dir_x)) = plane.to_yx_axes() else { return };
    let dir_z = dir_x.cross(dir_y).normalize();
    let rot = Quat::from_mat3(&Mat3::from_cols(dir_x, dir_y, dir_z));

    let d = 2. * BALL_R + SQUARE_H + CYLINDER_R + 1.;
    parent.spawn((
        PbrBundle {
            transform: Transform::from_translation(d * dir_x + d * dir_z).with_rotation(rot),
            mesh: cylinder,
            material: material.clone(),
            ..default()
        },
        NotShadowCaster,
        NotShadowReceiver,
        RenderLayers::layer(UI_CAMERA_LAYER),
        Collider::cylinder(0.01, CYLINDER_R),
        Sensor,
        TransformGizmoPart {
            material: material.clone(),
            highlighted: false,
            constraint: GizmoConstraint::Plane {
                plane,
                is_rotation: true,
            },
        },
    ));
}

pub struct RemoveTransformGizmo;

impl EntityCommand for RemoveTransformGizmo {
    fn write(self, id: Entity, world: &mut World) {
        let gizmo = {
            let mut q_gizmos = world.query::<(Entity, &TransformGizmo)>();
            q_gizmos.iter(world).find(|(_, gizmo)| gizmo.entity == id)
        };
        if let Some((gizmo_ent, gizmo)) = gizmo {
            world.entity_mut(gizmo.entity).remove::<HasTransformGizmo>();
            world.entity_mut(gizmo_ent).despawn_recursive();
        }
    }
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

    pub fn to_yx_axes(&self) -> (Vec3, Vec3) {
        match self {
            GizmoAxis::X => (Vec3::X, Vec3::NEG_Y),
            GizmoAxis::Y => (Vec3::Y, Vec3::X),
            GizmoAxis::Z => (Vec3::Z, Vec3::Y),
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
    pub fn plane_normal(&self, gtr: &GlobalTransform, ray: &Ray) -> Vec3 {
        match self {
            GizmoPlane::XY => gtr.back(),
            GizmoPlane::YZ => gtr.right(),
            GizmoPlane::ZX => gtr.up(),
            GizmoPlane::Camera => ray.direction,
        }
    }

    pub fn to_yx_axes(&self) -> Option<(Vec3, Vec3)> {
        match self {
            GizmoPlane::XY => Some((Vec3::Z, Vec3::Y)),
            GizmoPlane::YZ => Some((Vec3::X, Vec3::Z)),
            GizmoPlane::ZX => Some((Vec3::Y, Vec3::X)),
            GizmoPlane::Camera => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GizmoConstraint {
    Axis(GizmoAxis),
    Plane {
        plane: GizmoPlane,
        is_rotation: bool,
    },
}

impl GizmoConstraint {
    fn ray_cast(&self, origin: Vec3, ray: &Ray, gtr: &GlobalTransform) -> Option<Vec3> {
        let ray_p = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());
        match self {
            GizmoConstraint::Axis(axis) => {
                let (plane0, plane1) = axis.ray_cast_planes(gtr);
                let toi0 = ray_toi_with_halfspace(&origin.into(), &plane0.into(), &ray_p)?;
                let toi1 = ray_toi_with_halfspace(&origin.into(), &plane1.into(), &ray_p)?;
                let dir = axis.axis(gtr);
                let d0 = dir.dot(ray.origin + toi0 * ray.direction);
                let d1 = dir.dot(ray.origin + toi1 * ray.direction);
                let d = (d0 + d1) / 2.;
                Some(d * dir)
            }
            GizmoConstraint::Plane {
                plane,
                is_rotation: _,
            } => {
                let plane_normal = plane.plane_normal(gtr, ray);
                let toi = ray_toi_with_halfspace(&origin.into(), &plane_normal.into(), &ray_p)?;
                Some(ray.origin + toi * ray.direction)
            }
        }
    }

    fn is_rotation(&self) -> bool {
        if let GizmoConstraint::Plane {
            plane: _,
            is_rotation,
        } = self
        {
            *is_rotation
        } else {
            false
        }
    }
}

struct GizmoActiveState {
    origin: Vec3,
    delta: Vec3,
    constraint: GizmoConstraint,
}

#[derive(Component)]
struct TransformGizmo {
    entity: Entity,
    active_state: Option<GizmoActiveState>,
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
    pub square: Handle<Mesh>,
    pub cylinder: Handle<Mesh>,
}

const BAR_H: f32 = 4.0;
const BAR_W: f32 = 0.1;
const CONE_R: f32 = 0.4;
const CONE_H: f32 = 1.0;
const BALL_R: f32 = 0.5;
const SQUARE_H: f32 = 1.0;
const CYLINDER_R: f32 = 0.5;

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
            cone: meshes.add(Mesh::from(Cone::new(CONE_R, CONE_H, 10))),
            ball: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: BALL_R,
                    subdivisions: 20,
                })
                .unwrap(),
            ),
            square: meshes.add(Mesh::try_from(shape::Plane::from_size(SQUARE_H)).unwrap()),
            cylinder: meshes.add(
                Mesh::try_from(shape::Cylinder {
                    radius: CYLINDER_R,
                    height: 0.01,
                    ..Default::default()
                })
                .unwrap(),
            ),
        }
    }
}

fn update_gizmo_state(
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    materials: Res<BasicMaterials>,
    q_camera: Query<&MainCamera>,
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
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
        if gizmo.active_state.is_none() {
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
                let origin = gizmo_gtr.translation();
                if let Some(start_drag) = gizmo_part
                    .constraint
                    .ray_cast(origin, &mouse_ray, gizmo_gtr)
                {
                    gizmo.active_state = Some(GizmoActiveState {
                        origin,
                        delta: origin - start_drag,
                        constraint: gizmo_part.constraint,
                    });
                    if let Ok(mut window) = q_window.get_single_mut() {
                        window.cursor.icon = CursorIcon::Move;
                    }
                }
            }
        } else if !mouse.pressed(MouseButton::Left) {
            gizmo.active_state = None;
            gizmo_part.highlighted = false;
            *material = gizmo_part.material.clone();
            if let Ok(mut window) = q_window.get_single_mut() {
                window.cursor.icon = CursorIcon::Default;
            }
        }
    }
}

fn sync_parent_to_gizmo(
    q_camera: Query<&MainCamera>,
    mut q_parent: Query<(&mut Transform, Option<&Parent>), With<HasTransformGizmo>>,
    q_parent_parent: Query<&GlobalTransform, Without<TransformGizmo>>,
    q_gizmo: Query<(&TransformGizmo, &GlobalTransform)>,
) {
    let Ok(Some(mouse_ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else { return };
    for (gizmo, gizmo_gtr) in &q_gizmo {
        let Some(ref state) = gizmo.active_state else { continue };
        let Some(current) = state.constraint.ray_cast(state.origin, &mouse_ray, gizmo_gtr) else { continue };
        let current = current + state.delta;

        let Ok((mut parent_tr, parent_ent)) = q_parent.get_mut(gizmo.entity) else { continue };
        if let Some(parent_ent) = parent_ent {
            let Ok(parent_gtr) = q_parent_parent.get(parent_ent.get()) else { continue };
            if state.constraint.is_rotation() {
            } else {
                parent_tr.translation = parent_gtr.affine().inverse().transform_point3(current);
            }
        } else if state.constraint.is_rotation() {
        } else {
            parent_tr.translation = current;
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
                let (_, rot, pos) = parent_gtr.to_scale_rotation_translation();
                let tr = Transform::from_translation(pos).with_rotation(rot);
                *gizmo_gtr = GlobalTransform::from(tr);
                *gizmo_tr = tr;
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
