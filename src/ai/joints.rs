use bevy::{prelude::*, transform::TransformSystem};
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

use crate::{mesh::cylinder::Cylinder, ui::basic_materials::BasicMaterialsRes};

use super::kinematic_rig::{KinematicRigCollider, KinematicRigMesh};

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
    pub last_non_colliding_angle: f32,
    pub stop_at_collisions: bool,
}

fn process_joints(
    rapier: Res<RapierContext>,
    mut q_hinge: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &KinematicRigMesh,
        &KinematicHinge,
        &mut KinematicHingeCommand,
    )>,
    q_parent: Query<&Parent>,
    q_collider: Query<(Entity, &Collider), With<KinematicRigCollider>>,
    mut cmd: Commands,
) {
    for (entity, gtr, mut tr, rig_mesh, hinge, mut hinge_cmd) in &mut q_hinge {
        let srt = gtr.to_scale_rotation_translation();

        let (coll_ent, coll) = q_collider.get(rig_mesh.collider).unwrap();
        let parent = q_parent.iter_ancestors(coll_ent).next().unwrap();

        let mut colliding = false;
        if hinge_cmd.stop_at_collisions {
            rapier.intersections_with_shape(
                srt.2,
                srt.1,
                &coll,
                QueryFilter::new().exclude_sensors(),
                |colliding_ent| {
                    if colliding_ent == entity
                        || parent == colliding_ent
                        || q_parent.iter_ancestors(colliding_ent).next() == Some(parent)
                    {
                        true
                    } else {
                        warn!("We hit something: {:?}", colliding_ent);
                        colliding = true;
                        false
                    }
                },
            );
        }
        let mut cmd_finished = colliding;

        hinge_cmd.current_angle = hinge.get_angle(&tr);
        hinge_cmd.target_angle = hinge_cmd.target_angle.clamp(-PI + 0.01, PI);

        if colliding {
            hinge_cmd.target_angle = hinge_cmd.last_non_colliding_angle;
        } else {
            hinge_cmd.last_non_colliding_angle = hinge_cmd.current_angle;
        }

        let diff = hinge_cmd.target_angle - hinge_cmd.current_angle;
        if diff.abs() > 0.001 || colliding {
            let rot_angle = if colliding {
                diff
            } else {
                hinge.speed.min(diff.abs()) * diff.signum()
            };
            let anchor = tr.transform_point(hinge.anchor);
            tr.rotate_around(anchor, Quat::from_axis_angle(hinge.axis, rot_angle));
        } else {
            cmd_finished = true;
        }

        if cmd_finished {
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
