use bevy::{prelude::*, transform::TransformSystem};
use std::f32::consts::PI;

use crate::{mesh::cylinder::Cylinder, ui::basic_materials::BasicMaterialsRes};

pub struct JointsPlugin;

impl Plugin for JointsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicHinge>()
            .register_type::<KinematicHingeCommand>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                process_joints.before(TransformSystem::TransformPropagate),
            )
            .add_system(update_joint_mesh);
    }
}

#[derive(Component, Reflect)]
pub struct KinematicHinge {
    pub axis: Vec3,
    pub anchor: Vec3,
    pub length: f32,
    pub start_dir_up: Vec3,
    pub speed: f32,
    pub show_mesh: bool,
}

impl KinematicHinge {
    pub fn get_angle(&self, transform: &Transform) -> f32 {
        let sign = {
            let dir = self.start_dir_up.cross(transform.up());
            if dir.length() > 0.01 && dir.dot(transform.right()) < 0.01 {
                -1.
            } else {
                1.
            }
        };
        sign * self.start_dir_up.angle_between(transform.up())
    }
}

#[derive(Component, Reflect)]
pub struct KinematicHingeCommand {
    pub target_angle: f32,
    pub current_angle: f32,
}

fn process_joints(
    mut q_hinge: Query<(
        Entity,
        &mut Transform,
        &KinematicHinge,
        &mut KinematicHingeCommand,
    )>,
    mut cmd: Commands,
) {
    for (entity, mut platform_tr, hinge, mut hinge_cmd) in &mut q_hinge {
        let anchor = platform_tr.transform_point(hinge.anchor);
        hinge_cmd.current_angle = hinge.get_angle(&platform_tr);
        hinge_cmd.target_angle = hinge_cmd.target_angle.clamp(-PI + 0.01, PI);
        let diff = hinge_cmd.target_angle - hinge_cmd.current_angle;
        if diff.abs() > 0.001 {
            let rotation =
                Quat::from_axis_angle(hinge.axis, hinge.speed.min(diff.abs()) * diff.signum());
            platform_tr.rotate_around(anchor, rotation);
        } else {
            cmd.entity(entity).remove::<KinematicHingeCommand>();
        }
    }
}

#[derive(Component, Reflect)]
pub struct KinematicHingeMesh;

fn update_joint_mesh(
    materials: Res<BasicMaterialsRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_hinge: Query<(
        Entity,
        &Transform,
        &Handle<StandardMaterial>,
        &KinematicHinge,
        Option<&KinematicHingeCommand>,
        &Children,
    )>,
    mut q_mesh: Query<
        &mut Handle<StandardMaterial>,
        (With<KinematicHingeMesh>, Without<KinematicHinge>),
    >,
    mut cmd: Commands,
) {
    for (entity, tr, material, hinge, hinge_cmd, children) in &q_hinge {
        if hinge.show_mesh {
            let material = if hinge_cmd.is_some() {
                &materials.ui_green
            } else {
                material
            };
            let mesh_ent = children.iter().find(|c| q_mesh.contains(**c));
            if let Some(mesh_ent) = mesh_ent {
                if let Ok(mut mesh_material) = q_mesh.get_mut(*mesh_ent) {
                    if mesh_material.as_ref() != material {
                        *mesh_material = material.clone();
                    }
                }
            } else {
                cmd.entity(entity).with_children(|children| {
                    let dir_y = tr.compute_affine().inverse().transform_vector3(hinge.axis);
                    let dir_x = dir_y.any_orthonormal_vector();
                    children.spawn((
                        PbrBundle {
                            transform: Transform::from_translation(hinge.anchor).with_rotation(
                                Quat::from_mat3(&Mat3::from_cols(
                                    dir_x,
                                    dir_y,
                                    dir_x.cross(dir_y).normalize(),
                                )),
                            ),
                            mesh: meshes.add(Mesh::from(Cylinder {
                                radius: 0.1,
                                height: hinge.length - 0.01,
                                resolution: 5,
                                segments: 1,
                            })),
                            material: material.clone(),
                            ..default()
                        },
                        KinematicHingeMesh,
                    ));
                });
            }
        } else {
            for c in children {
                if q_mesh.contains(*c) {
                    cmd.entity(*c).despawn_recursive();
                }
            }
        }
    }
}
