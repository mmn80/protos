use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::mesh::cone::Cone;

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
    pub bar: Option<Handle<Mesh>>,
    pub cone: Option<Handle<Mesh>>,
}

impl Default for MoveGizmoRes {
    fn default() -> Self {
        Self {
            x_mat: None,
            y_mat: None,
            z_mat: None,
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
        base_color: Color::rgba(0.9, 0.5, 0.5, 0.2),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        ..default()
    }));
    res.y_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.9, 0.5, 0.2),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        ..default()
    }));
    res.z_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.5, 0.9, 0.2),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        ..default()
    }));
    res.bar = Some(meshes.add(Mesh::from(shape::Box::new(BAR_W, BAR_H, BAR_W))));
    res.cone = Some(meshes.add(Mesh::from(Cone::new(CONE_W / 2., CONE_H, 10))));
}

#[derive(Component)]
pub struct MoveGizmo;

fn update_move_gizmos(
    ui: Res<SidePanelState>,
    res: Res<MoveGizmoRes>,
    ctx: Res<RapierContext>,
    q_selected: Query<(Entity, &GlobalTransform, &Children), With<Selected>>,
    q_gizmo: Query<Entity, With<MoveGizmo>>,
    q_parent: Query<&Parent>,
    mut cmd: Commands,
) {
    if ui.selected_show_move_gizmo {
        for (sel, trans, children) in q_selected.iter() {
            if !children.iter().any(|c| q_gizmo.contains(*c)) {
                let pos = trans.translation();
                for (y_axis, x_axis, m) in [
                    (trans.right(), trans.down(), res.x_mat.clone().unwrap()),
                    (trans.up(), trans.right(), res.y_mat.clone().unwrap()),
                    (trans.back(), trans.up(), res.z_mat.clone().unwrap()),
                ] {
                    add_gizmo(&res, &ctx, sel, trans, pos, y_axis, x_axis, m, &mut cmd);
                }
            }
        }
        for ent in q_gizmo.iter() {
            for parent in q_parent.iter_ancestors(ent) {
                if !q_selected.contains(parent) {
                    cmd.entity(ent).despawn_recursive();
                }
                break;
            }
        }
    } else {
        for ent in q_gizmo.iter() {
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
    commands: &mut Commands,
) {
    if let Some((_ent, attach_point_toi)) =
        rapier_ctx.cast_ray(pos, dir_y, 50., false, QueryFilter::new())
    {
        let inverse = sel_trans.affine().inverse();
        let attach_point = inverse.transform_point3(pos + attach_point_toi * dir_y);
        let dir_x = inverse.transform_vector3(dir_x);
        let dir_y = inverse.transform_vector3(dir_y);
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
                    parent.spawn(PbrBundle {
                        transform: Transform::from_xyz(0., BAR_H / 2., 0.),
                        mesh: res.bar.clone().unwrap(),
                        material: material.clone(),
                        ..default()
                    });
                    parent
                        .spawn(PbrBundle {
                            transform: Transform::from_xyz(0., BAR_H + CONE_H / 2., 0.),
                            mesh: res.cone.clone().unwrap(),
                            material,
                            ..default()
                        })
                        .insert((Collider::cone(CONE_H / 2., CONE_W / 2.), Sensor));
                });
        });
    }
}
