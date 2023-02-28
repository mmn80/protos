use bevy::{prelude::*, transform::TransformSystem};
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

use crate::{mesh::cylinder::Cylinder, ui::basic_materials::BasicMaterialsRes};

use super::kinematic_rig::{KinematicRigCollider, KinematicRigMesh};

pub struct KinematicJointsPlugin;

impl Plugin for KinematicJointsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RevoluteJoint>()
            .register_type::<RevoluteJointCommand>()
            .register_type::<SphericalJoint>()
            .register_type::<SphericalJointCommand>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                update_revolute_joints.before(TransformSystem::TransformPropagate),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                update_spherical_joints.before(TransformSystem::TransformPropagate),
            )
            .add_system(update_revolute_joint_mesh)
            .add_system(update_spherical_joint_mesh);
    }
}

#[derive(PartialEq, Eq)]
pub enum KinematicJointType {
    Revolute,
    Spherical,
}

#[derive(Component, Reflect)]
pub struct RevoluteJoint {
    pub axis: Vec3,
    pub anchor: Vec3,
    pub length: f32,
    pub start_dir: Vec3,
    pub show_mesh: bool,
}

impl RevoluteJoint {
    pub fn get_angle(&self, transform: &Transform) -> f32 {
        let sign = {
            let dir = self.start_dir.cross(transform.up());
            if dir.length() > 0.01 && dir.dot(transform.right()) < 0.01 {
                -1.
            } else {
                1.
            }
        };
        sign * self.start_dir.angle_between(transform.up())
    }
}

#[derive(Component, Reflect)]
pub struct RevoluteJointCommand {
    pub target_angle: f32,
    pub speed: f32,
    pub stop_at_collisions: bool,
    current_angle: f32,
    last_non_colliding_angle: f32,
}

impl RevoluteJointCommand {
    pub fn new(target_angle: f32, speed: f32, stop_at_collisions: bool) -> Self {
        Self {
            target_angle,
            speed,
            stop_at_collisions,
            current_angle: 0.0,
            last_non_colliding_angle: 0.0,
        }
    }
}

fn update_revolute_joints(
    rapier: Res<RapierContext>,
    mut q_joint: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &KinematicRigMesh,
        &RevoluteJoint,
        &mut RevoluteJointCommand,
    )>,
    q_parent: Query<&Parent>,
    q_collider: Query<(Entity, &Collider), With<KinematicRigCollider>>,
    mut cmd: Commands,
) {
    for (entity, gtr, mut tr, rig_mesh, joint, mut joint_cmd) in &mut q_joint {
        let srt = gtr.to_scale_rotation_translation();

        let (coll_ent, coll) = q_collider.get(rig_mesh.collider).unwrap();
        let parent = q_parent.iter_ancestors(coll_ent).next().unwrap();

        let mut colliding = false;
        if joint_cmd.stop_at_collisions {
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

        joint_cmd.current_angle = joint.get_angle(&tr);
        joint_cmd.target_angle = joint_cmd.target_angle.clamp(-PI + 0.01, PI);

        if colliding {
            joint_cmd.target_angle = joint_cmd.last_non_colliding_angle;
        } else {
            joint_cmd.last_non_colliding_angle = joint_cmd.current_angle;
        }

        let diff = joint_cmd.target_angle - joint_cmd.current_angle;
        if diff.abs() > 0.001 || colliding {
            let rot_angle = if colliding {
                diff
            } else {
                joint_cmd.speed.min(diff.abs()) * diff.signum()
            };
            let anchor = tr.transform_point(joint.anchor);
            tr.rotate_around(anchor, Quat::from_axis_angle(joint.axis, rot_angle));
        } else {
            cmd_finished = true;
        }

        if cmd_finished {
            cmd.entity(entity).remove::<RevoluteJointCommand>();
        }
    }
}

#[derive(Component, Reflect)]
pub struct RevoluteJointMesh;

fn update_revolute_joint_mesh(
    materials: Res<BasicMaterialsRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_joint: Query<(
        Entity,
        &Transform,
        &Handle<StandardMaterial>,
        &RevoluteJoint,
        Option<&RevoluteJointCommand>,
        &Children,
    )>,
    mut q_mesh: Query<
        &mut Handle<StandardMaterial>,
        (With<RevoluteJointMesh>, Without<RevoluteJoint>),
    >,
    mut cmd: Commands,
) {
    for (entity, tr, material, joint, joint_cmd, children) in &q_joint {
        if joint.show_mesh {
            let material = if joint_cmd.is_some() {
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
                    let dir_y = tr.compute_affine().inverse().transform_vector3(joint.axis);
                    let dir_x = dir_y.any_orthonormal_vector();
                    children.spawn((
                        PbrBundle {
                            transform: Transform::from_translation(joint.anchor).with_rotation(
                                Quat::from_mat3(&Mat3::from_cols(
                                    dir_x,
                                    dir_y,
                                    dir_x.cross(dir_y).normalize(),
                                )),
                            ),
                            mesh: meshes.add(Mesh::from(Cylinder {
                                radius: 0.1,
                                height: joint.length - 0.01,
                                resolution: 5,
                                segments: 1,
                            })),
                            material: material.clone(),
                            ..default()
                        },
                        RevoluteJointMesh,
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

#[derive(Component, Reflect)]
pub struct SphericalJoint {
    pub anchor: Vec3,
    pub show_mesh: bool,
    pub start_rot: Quat,
    pub start_pos: Vec3,
}

#[derive(Component, Reflect)]
pub struct SphericalJointCommand {
    start_rot: Option<Quat>,
    pub target_rot: Quat,
    pub speed: f32,
    pub stop_at_collisions: bool,
    current: f32,
    last_non_colliding: f32,
    delta: f32,
}

impl SphericalJointCommand {
    pub fn new(target: Quat, speed: f32, stop_at_collisions: bool) -> Self {
        Self {
            start_rot: None,
            target_rot: target,
            speed,
            stop_at_collisions,
            current: 0.,
            last_non_colliding: 0.,
            delta: 0.,
        }
    }

    pub fn new_euler(
        target_x: f32,
        target_y: f32,
        target_z: f32,
        speed: f32,
        stop_at_collisions: bool,
    ) -> Self {
        Self {
            start_rot: None,
            target_rot: Quat::from_euler(EulerRot::XZY, target_x, target_z, target_y),
            speed,
            stop_at_collisions,
            current: 0.,
            last_non_colliding: 0.,
            delta: 0.,
        }
    }
}

fn update_spherical_joints(
    rapier: Res<RapierContext>,
    mut q_joint: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &KinematicRigMesh,
        &SphericalJoint,
        &mut SphericalJointCommand,
    )>,
    q_parent: Query<&Parent>,
    q_collider: Query<(Entity, &Collider), With<KinematicRigCollider>>,
    mut cmd: Commands,
) {
    for (entity, gtr, mut tr, rig_mesh, joint, mut joint_cmd) in &mut q_joint {
        let srt = gtr.to_scale_rotation_translation();

        let (coll_ent, coll) = q_collider.get(rig_mesh.collider).unwrap();
        let parent = q_parent.iter_ancestors(coll_ent).next().unwrap();

        let mut colliding = false;
        if joint_cmd.stop_at_collisions {
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

        if joint_cmd.start_rot.is_none() {
            joint_cmd.start_rot = Some((tr.rotation * joint.start_rot.inverse()).normalize());
            let diff = {
                let (t_ax, t_an) = joint_cmd.target_rot.to_axis_angle();
                let (c_ax, c_an) = joint_cmd.start_rot.unwrap().to_axis_angle();
                t_ax.angle_between(c_ax).abs() + (t_an - c_an).abs()
            };
            joint_cmd.delta = (joint_cmd.speed / diff).clamp(0., 1.);
        } else {
            joint_cmd.current = (joint_cmd.current + joint_cmd.delta).clamp(0., 1.);
        }
        if (1. - joint_cmd.current).abs() < 0.001 {
            cmd_finished = true;
        }

        if colliding {
            joint_cmd.current = joint_cmd.last_non_colliding;
        } else {
            joint_cmd.last_non_colliding = joint_cmd.current;
        }

        let current_rot = joint_cmd
            .start_rot
            .unwrap()
            .slerp(joint_cmd.target_rot, joint_cmd.current)
            .normalize();

        tr.translation = joint.start_pos;
        tr.rotation = joint.start_rot;
        let anchor = tr.transform_point(joint.anchor);
        tr.rotate_around(anchor, current_rot);

        if cmd_finished {
            cmd.entity(entity).remove::<SphericalJointCommand>();
        }
    }
}

#[derive(Component, Reflect)]
pub struct SphericalJointMesh;

fn update_spherical_joint_mesh(
    materials: Res<BasicMaterialsRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_joint: Query<(
        Entity,
        &Handle<StandardMaterial>,
        &SphericalJoint,
        Option<&SphericalJointCommand>,
        &Children,
    )>,
    mut q_mesh: Query<
        &mut Handle<StandardMaterial>,
        (With<SphericalJointMesh>, Without<SphericalJoint>),
    >,
    mut cmd: Commands,
) {
    for (entity, material, joint, joint_cmd, children) in &q_joint {
        if joint.show_mesh {
            let material = if joint_cmd.is_some() {
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
                    children.spawn((
                        PbrBundle {
                            transform: Transform::from_translation(joint.anchor),
                            mesh: meshes.add(Mesh::from(shape::Icosphere {
                                radius: 0.2,
                                subdivisions: 5,
                            })),
                            material: material.clone(),
                            ..default()
                        },
                        SphericalJointMesh,
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
