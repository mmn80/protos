use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

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
    pub gizmo_mat: Option<Handle<StandardMaterial>>,
    pub gizmo_bar: Option<Handle<Mesh>>,
    pub gizmo_handle: Option<Handle<Mesh>>,
}

impl Default for MoveGizmoRes {
    fn default() -> Self {
        Self {
            gizmo_mat: None,
            gizmo_bar: None,
            gizmo_handle: None,
        }
    }
}

fn setup_move_gizmos(
    mut res: ResMut<MoveGizmoRes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    res.gizmo_mat = Some(materials.add(StandardMaterial {
        base_color: Color::rgba(0.5, 0.9, 0.5, 0.2),
        metallic: 0.9,
        perceptual_roughness: 0.8,
        reflectance: 0.8,
        ..default()
    }));
    res.gizmo_bar = Some(meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, GIZMO_DIST))));
    res.gizmo_handle = Some(meshes.add(Mesh::from(shape::Box::new(
        GIZMO_SIZE, GIZMO_SIZE, GIZMO_SIZE,
    ))));
}

#[derive(Component)]
pub struct MoveGizmo {
    pub attach_point: Vec3,
    pub move_direction: Vec3,
}

fn update_move_gizmos(
    ui: Res<SidePanelState>,
    res: Res<MoveGizmoRes>,
    ctx: Res<RapierContext>,
    q_selected: Query<(Entity, &GlobalTransform, &Children), With<Selected>>,
    q_gizmo: Query<&MoveGizmo>,
    mut cmd: Commands,
) {
    if ui.selected_show_move_gizmo {
        for (sel, trans, children) in q_selected.iter() {
            if !children.iter().any(|c| q_gizmo.contains(*c)) {
                let pos = trans.translation();
                add_gizmo(
                    &res,
                    &ctx,
                    sel,
                    trans,
                    pos,
                    trans.up(),
                    trans.right(),
                    &mut cmd,
                );
                add_gizmo(
                    &res,
                    &ctx,
                    sel,
                    trans,
                    pos,
                    trans.right(),
                    trans.back(),
                    &mut cmd,
                );
                add_gizmo(
                    &res,
                    &ctx,
                    sel,
                    trans,
                    pos,
                    trans.back(),
                    trans.up(),
                    &mut cmd,
                );
            }
        }
    }
}

const GIZMO_DIST: f32 = 2.0;
const GIZMO_SIZE: f32 = 0.5;

fn add_gizmo(
    res: &MoveGizmoRes,
    rapier_ctx: &Res<RapierContext>,
    sel: Entity,
    sel_trans: &GlobalTransform,
    pos: Vec3,
    dir: Vec3,
    dir_y: Vec3,
    commands: &mut Commands,
) {
    if let Some((_ent, attach_point_toi)) =
        rapier_ctx.cast_ray(pos, dir, 50., false, QueryFilter::new())
    {
        let material = res.gizmo_mat.clone().unwrap();
        let attach_point = sel_trans
            .affine()
            .inverse()
            .transform_point3(pos + attach_point_toi * dir);
        println!("Gizmo attach point {attach_point}");
        commands.entity(sel).with_children(|parent| {
            parent
                .spawn(SpatialBundle::from(
                    Transform::from_translation(attach_point).looking_at(attach_point + dir, dir_y),
                ))
                .insert(MoveGizmo {
                    attach_point,
                    move_direction: dir,
                })
                .with_children(|parent| {
                    parent.spawn(PbrBundle {
                        transform: Transform::from_xyz(0., 0., -GIZMO_DIST / 2.),
                        mesh: res.gizmo_bar.clone().unwrap(),
                        material: material.clone(),
                        ..default()
                    });
                    parent
                        .spawn(PbrBundle {
                            transform: Transform::from_xyz(0., 0., -GIZMO_DIST - GIZMO_SIZE / 2.),
                            mesh: res.gizmo_handle.clone().unwrap(),
                            material,
                            ..default()
                        })
                        .insert((
                            Collider::cuboid(GIZMO_SIZE / 2., GIZMO_SIZE / 2., GIZMO_SIZE / 2.),
                            Sensor,
                        ));
                });
        });
    }
}
