use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

use crate::{camera::MainCamera, mesh::cone::Cone};

use super::{selection::Selected, side_panel::SidePanelState};

pub struct MoveGizmoPlugin;

impl Plugin for MoveGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MoveGizmoRes::default())
            .add_startup_system(setup_move_gizmos)
            .add_system(update_move_gizmos);
    }
}

#[derive(Resource)]
struct MoveGizmoRes {
    pub x_mat: Option<Handle<StandardMaterial>>,
    pub y_mat: Option<Handle<StandardMaterial>>,
    pub z_mat: Option<Handle<StandardMaterial>>,
    pub selected_mat: Option<Handle<StandardMaterial>>,
    pub bar: Option<Handle<Mesh>>,
    pub cone: Option<Handle<Mesh>>,
}

impl Default for MoveGizmoRes {
    fn default() -> Self {
        Self {
            x_mat: None,
            y_mat: None,
            z_mat: None,
            selected_mat: None,
            bar: None,
            cone: None,
        }
    }
}

const BAR_H: f32 = 2.0;
const BAR_W: f32 = 0.1;
const CONE_W: f32 = 0.5;
const CONE_H: f32 = 1.0;

fn setup_move_gizmos(
    mut res: ResMut<MoveGizmoRes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    res.x_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.9, 0.5, 0.5, 0.9),
        emissive: Color::rgb(0.9, 0.5, 0.5),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }));
    res.y_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.9, 0.5, 0.9),
        emissive: Color::rgb(0.5, 0.9, 0.5),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }));
    res.z_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.5, 0.9, 0.9),
        emissive: Color::rgb(0.5, 0.5, 0.9),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        alpha_mode: AlphaMode::Blend,
        ..default()
    }));
    res.selected_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        emissive: Color::rgb(1.0, 1.0, 1.0),
        metallic: 0.8,
        perceptual_roughness: 0.5,
        reflectance: 0.5,
        ..default()
    }));
    res.bar = Some(meshes.add(Mesh::from(shape::Box::new(BAR_W, BAR_H, BAR_W))));
    res.cone = Some(meshes.add(Mesh::from(Cone::new(CONE_W / 2., CONE_H, 10))));
}

#[derive(Component)]
pub struct MoveGizmo;

#[derive(Component)]
pub enum MoveGizmoHandle {
    X,
    Y,
    Z,
}

fn update_move_gizmos(
    ui: Res<SidePanelState>,
    res: Res<MoveGizmoRes>,
    rapier: Res<RapierContext>,
    q_selected: Query<(Entity, &GlobalTransform, &Children), With<Selected>>,
    q_gizmo: Query<(Entity, &GlobalTransform), With<MoveGizmo>>,
    q_parent: Query<&Parent>,
    q_camera: Query<&MainCamera>,
    mut q_material: Query<(Entity, &mut Handle<StandardMaterial>, &MoveGizmoHandle)>,
    mut cmd: Commands,
) {
    if ui.selected_show_move_gizmo {
        for (sel, trans, children) in q_selected.iter() {
            if !children.iter().any(|c| q_gizmo.contains(*c)) {
                let pos = trans.translation();
                for (y_axis, x_axis, m, g) in [
                    (
                        trans.right(),
                        trans.down(),
                        res.x_mat.clone().unwrap(),
                        MoveGizmoHandle::X,
                    ),
                    (
                        trans.up(),
                        trans.right(),
                        res.y_mat.clone().unwrap(),
                        MoveGizmoHandle::Y,
                    ),
                    (
                        trans.back(),
                        trans.up(),
                        res.z_mat.clone().unwrap(),
                        MoveGizmoHandle::Z,
                    ),
                ] {
                    add_gizmo(
                        &res, &rapier, sel, trans, pos, y_axis, x_axis, m, g, &mut cmd,
                    );
                }
            }
        }

        for (ent, _) in q_gizmo.iter() {
            if let Some(parent) = q_parent.iter_ancestors(ent).next() {
                if !q_selected.contains(parent) {
                    cmd.entity(ent).despawn_recursive();
                }
            }
        }

        let mut hover_ent = None;
        if let Ok(Some(ray)) = q_camera.get_single().map(|c| c.mouse_ray.clone()) {
            if let Some((hit_ent, _)) = rapier.cast_ray_and_get_normal(
                ray.origin,
                ray.direction,
                1000.,
                false,
                QueryFilter::new().exclude_solids(),
            ) {
                if let Some(parent) = q_parent.iter_ancestors(hit_ent).next() {
                    if let Ok((_, _gizmo_tr)) = q_gizmo.get(parent) {
                        hover_ent = Some(hit_ent);
                    }
                }
            }
        }

        if let (Some(x_m), Some(y_m), Some(z_m), Some(sel_m)) = (
            res.x_mat.clone(),
            res.y_mat.clone(),
            res.z_mat.clone(),
            res.selected_mat.clone(),
        ) {
            for (ent, mut mat_handle, gizmo) in q_material.iter_mut() {
                if Some(ent) == hover_ent {
                    *mat_handle = sel_m.clone();
                } else {
                    *mat_handle = match gizmo {
                        MoveGizmoHandle::X => x_m.clone(),
                        MoveGizmoHandle::Y => y_m.clone(),
                        MoveGizmoHandle::Z => z_m.clone(),
                    }
                }
            }
        }
    } else {
        for (ent, _) in q_gizmo.iter() {
            cmd.entity(ent).despawn_recursive();
        }
    }
}

fn add_gizmo(
    res: &MoveGizmoRes,
    rapier_ctx: &Res<RapierContext>,
    sel: Entity,
    sel_trans: &GlobalTransform,
    pos: Vec3,
    dir_y: Vec3,
    dir_x: Vec3,
    material: Handle<StandardMaterial>,
    gizmo_handle: MoveGizmoHandle,
    commands: &mut Commands,
) {
    if let Some((_ent, attach_point_toi)) =
        rapier_ctx.cast_ray(pos, dir_y, 50., false, QueryFilter::new())
    {
        let inverse = sel_trans.affine().inverse();
        let attach_point = inverse.transform_point3(pos + attach_point_toi * dir_y);
        let dir_x = inverse.transform_vector3(dir_x).normalize();
        let dir_y = inverse.transform_vector3(dir_y).normalize();
        commands.entity(sel).with_children(|parent| {
            let rotation = Quat::from_mat3(&Mat3::from_cols(
                dir_x,
                dir_y,
                dir_x.cross(dir_y).normalize(),
            ));
            parent
                .spawn(SpatialBundle::from(
                    Transform::from_translation(attach_point).with_rotation(rotation),
                ))
                .insert(MoveGizmo)
                .with_children(|parent| {
                    parent.spawn((
                        PbrBundle {
                            transform: Transform::from_xyz(0., BAR_H / 2., 0.),
                            mesh: res.bar.clone().unwrap(),
                            material: material.clone(),
                            ..default()
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                    parent
                        .spawn((
                            PbrBundle {
                                transform: Transform::from_xyz(0., BAR_H + CONE_H / 2., 0.),
                                mesh: res.cone.clone().unwrap(),
                                material,
                                ..default()
                            },
                            gizmo_handle,
                            NotShadowCaster,
                            NotShadowReceiver,
                        ))
                        .insert((Collider::cone(CONE_H / 2., CONE_W / 2.), Sensor));
                });
        });
    }
}
