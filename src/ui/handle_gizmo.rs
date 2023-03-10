use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use crate::{camera::MainCamera, mesh::cone::Cone};

use super::{basic_materials::BasicMaterials, side_panel::SidePanel};

pub struct HandleGizmoPlugin;

impl Plugin for HandleGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HandleGizmo>()
            .add_event::<AddHandleGizmo>()
            .add_event::<RemoveHandleGizmo>()
            .add_event::<HandleGizmoDragged>()
            .add_systems((add_handles, remove_handles, update_handles));
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Reflect)]
pub enum HandleGizmoAxis {
    X,
    Y,
    Z,
}

impl HandleGizmoAxis {
    pub fn to_yx_axes(&self, trans: &GlobalTransform) -> (Vec3, Vec3) {
        match self {
            HandleGizmoAxis::X => (trans.right(), trans.down()),
            HandleGizmoAxis::Y => (trans.up(), trans.right()),
            HandleGizmoAxis::Z => (trans.back(), trans.up()),
        }
    }
}

pub struct AddHandleGizmo {
    pub entity: Entity,
    pub axis: HandleGizmoAxis,
    pub material: Handle<StandardMaterial>,
}

pub struct RemoveHandleGizmo {
    pub entity: Entity,
    pub axis: HandleGizmoAxis,
}

pub struct HandleGizmoDragged {
    pub entity: Entity,
    pub axis: HandleGizmoAxis,
    pub direction: Vec3,
    pub drag_delta: f32,
}

#[derive(Component, Reflect)]
pub struct HandleGizmo {
    pub axis: HandleGizmoAxis,
    pub material: Handle<StandardMaterial>,
}

#[derive(Component, Reflect)]
pub struct HandleGizmoElement;

#[derive(Default)]
struct HandleGizmoMeshes {
    pub base: Option<Handle<Mesh>>,
    pub bar: Option<Handle<Mesh>>,
    pub cone: Option<Handle<Mesh>>,
    pub ball: Option<Handle<Mesh>>,
}

const BASE_W: f32 = 0.8;
const BASE_H: f32 = 0.1;
const BAR_H: f32 = 2.0;
const BAR_W: f32 = 0.1;
const CONE_W: f32 = 0.8;
const CONE_H: f32 = 1.0;
const BALL_R: f32 = 0.5;

fn add_handles(
    mut local: Local<HandleGizmoMeshes>,
    mut ev_add: EventReader<AddHandleGizmo>,
    mut meshes: ResMut<Assets<Mesh>>,
    rapier: Res<RapierContext>,
    q_global_trans: Query<&GlobalTransform>,
    mut cmd: Commands,
) {
    if local.base.is_none() {
        local.base = Some(
            meshes.add(
                Mesh::try_from(shape::Cylinder {
                    radius: BASE_W / 2.,
                    height: BASE_H,
                    resolution: 20,
                    segments: 1,
                })
                .unwrap(),
            ),
        );
    }
    if local.bar.is_none() {
        local.bar = Some(
            meshes.add(
                Mesh::try_from(shape::Cylinder {
                    radius: BAR_W / 2.,
                    height: BAR_H,
                    resolution: 10,
                    segments: 1,
                })
                .unwrap(),
            ),
        );
    }
    if local.cone.is_none() {
        local.cone = Some(meshes.add(Mesh::from(Cone::new(CONE_W / 2., CONE_H, 10))));
    }
    if local.ball.is_none() {
        local.ball = Some(
            meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: BALL_R,
                    subdivisions: 20,
                })
                .unwrap(),
            ),
        );
    }

    for AddHandleGizmo {
        entity,
        axis,
        material,
    } in ev_add.iter()
    {
        let Ok(trans) = q_global_trans.get(*entity) else { continue };
        let (dir_y, dir_x) = axis.to_yx_axes(trans);
        let pos = trans.translation();
        let Some((_ent, attach_point_toi)) =
            rapier.cast_ray(pos, dir_y, 50., false, QueryFilter::new()) else { continue };
        let inverse = trans.affine().inverse();
        let attach_point = inverse.transform_point3(pos + attach_point_toi * dir_y);
        let dir_x = inverse.transform_vector3(dir_x).normalize();
        let dir_y = inverse.transform_vector3(dir_y).normalize();
        cmd.entity(*entity).with_children(|parent| {
            let rotation = Quat::from_mat3(&Mat3::from_cols(
                dir_x,
                dir_y,
                dir_x.cross(dir_y).normalize(),
            ));
            parent
                .spawn((
                    SpatialBundle::from(
                        Transform::from_translation(attach_point).with_rotation(rotation),
                    ),
                    HandleGizmo {
                        axis: *axis,
                        material: material.clone(),
                    },
                ))
                .with_children(|parent| {
                    let mut dist = 0.;
                    parent.spawn((
                        PbrBundle {
                            transform: Transform::from_xyz(0., BASE_H / 2. + dist, 0.),
                            mesh: local.base.clone().unwrap(),
                            material: material.clone(),
                            ..default()
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                        HandleGizmoElement,
                        Collider::cylinder(BASE_H / 2., BASE_W / 2.),
                        Sensor,
                    ));
                    dist += BASE_H;
                    parent.spawn((
                        PbrBundle {
                            transform: Transform::from_xyz(0., dist + BAR_H / 2., 0.),
                            mesh: local.bar.clone().unwrap(),
                            material: material.clone(),
                            ..default()
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                        HandleGizmoElement,
                        Collider::cuboid(BAR_W / 2., BAR_H / 2., BAR_W / 2.),
                        Sensor,
                    ));
                    dist += BAR_H;
                    parent.spawn((
                        PbrBundle {
                            transform: Transform::from_xyz(0., dist + BALL_R, 0.),
                            mesh: local.ball.clone().unwrap(),
                            material: material.clone(),
                            ..default()
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                        HandleGizmoElement,
                        Collider::ball(BALL_R),
                        Sensor,
                    ));
                });
        });
    }
}

fn remove_handles(
    mut ev_remove: EventReader<RemoveHandleGizmo>,
    q_gizmo: Query<&HandleGizmo>,
    q_children: Query<&Children>,
    mut cmd: Commands,
) {
    for RemoveHandleGizmo { entity, axis } in ev_remove.iter() {
        for child in q_children.iter_descendants(*entity) {
            if let Ok(gizmo) = q_gizmo.get(child) {
                if gizmo.axis == *axis {
                    cmd.entity(child).despawn_recursive();
                }
            }
        }
    }
}

#[derive(Default)]
struct HandleGizmoState {
    pub active_gizmo: Option<Entity>,
    pub drag_last_y: Option<f32>,
}

fn update_handles(
    mut local: Local<HandleGizmoState>,
    mut ev_drag: EventWriter<HandleGizmoDragged>,
    mouse: Res<Input<MouseButton>>,
    rapier: Res<RapierContext>,
    panel: Res<SidePanel>,
    materials: Res<BasicMaterials>,
    q_parent: Query<&Parent>,
    q_camera: Query<&MainCamera>,
    q_gizmo: Query<(&HandleGizmo, &GlobalTransform)>,
    mut q_material: Query<(Entity, &mut Handle<StandardMaterial>), With<HandleGizmoElement>>,
) {
    let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) else { return };
    if local.active_gizmo.is_none() {
        if let Some((hit_ent, _)) = rapier.cast_ray(
            ray.origin,
            ray.direction,
            1000.,
            false,
            QueryFilter::new().exclude_solids(),
        ) {
            if let Some(gizmo) = q_parent.iter_ancestors(hit_ent).next() {
                if q_gizmo.contains(gizmo) {
                    local.active_gizmo = Some(gizmo);
                }
            }
        }
    }

    if mouse.pressed(MouseButton::Left) && !panel.mouse_over {
        if let Some(active_gizmo) = local.active_gizmo {
            if let (Some(target), Ok((gizmo, gizmo_gtr))) = (
                q_parent.iter_ancestors(active_gizmo).next(),
                q_gizmo.get(active_gizmo),
            ) {
                let ray_p = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());
                let center = gizmo_gtr.transform_point(Vec3::ZERO);
                if let (Some(toi0), Some(toi1)) = (
                    ray_toi_with_halfspace(&center.into(), &gizmo_gtr.right().into(), &ray_p),
                    ray_toi_with_halfspace(&center.into(), &gizmo_gtr.back().into(), &ray_p),
                ) {
                    let y0 = gizmo_gtr.up().dot(ray.origin + toi0 * ray.direction);
                    let y1 = gizmo_gtr.up().dot(ray.origin + toi1 * ray.direction);
                    let drag_y = (y0 + y1) / 2.;
                    if let Some(drag_last_y) = local.drag_last_y {
                        ev_drag.send(HandleGizmoDragged {
                            entity: target,
                            axis: gizmo.axis,
                            direction: gizmo_gtr.up(),
                            drag_delta: drag_y - drag_last_y,
                        });
                        local.drag_last_y = Some(drag_y);
                    } else {
                        local.drag_last_y = Some(drag_y);
                    }
                }
            }
        }
    }

    for (element, mut mat_handle) in q_material.iter_mut() {
        let Some(parent) = q_parent.iter_ancestors(element).next() else { continue };
        if Some(parent) == local.active_gizmo {
            if *mat_handle != materials.ui_selected {
                *mat_handle = materials.ui_selected.clone();
            }
        } else if let Ok((gizmo, _)) = q_gizmo.get(parent) {
            if *mat_handle != gizmo.material {
                *mat_handle = gizmo.material.clone();
            }
        }
    }

    if !mouse.pressed(MouseButton::Left) || panel.mouse_over {
        local.active_gizmo = None;
        local.drag_last_y = None;
    }
}
