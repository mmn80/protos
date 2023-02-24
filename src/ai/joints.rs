use bevy::{prelude::*, transform::TransformSystem};
use std::f32::consts::PI;

use crate::mesh::cylinder::Cylinder;

pub struct JointsPlugin;

impl Plugin for JointsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicHinge>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                process_joints.before(TransformSystem::TransformPropagate),
            )
            .add_system(add_joint_mesh);
    }
}

#[derive(Component, Reflect)]
pub struct KinematicHinge {
    pub target_angle: f32,
    pub axis: Vec3,
    pub anchor: Vec3,
    pub length: f32,
    pub start_dir_up: Vec3,
    pub speed: f32,
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

#[derive(Component)]
pub struct KinematicHingeMoving;

fn process_joints(
    q_command: Query<Entity, Changed<KinematicHinge>>,
    mut q_hinge: Query<(Entity, &mut Transform, &KinematicHinge), With<KinematicHingeMoving>>,
    mut cmd: Commands,
) {
    for hinge in &q_command {
        cmd.entity(hinge).insert(KinematicHingeMoving);
    }

    for (entity, mut platform_tr, hinge) in &mut q_hinge {
        let anchor = platform_tr.transform_point(hinge.anchor);
        let angle = hinge.get_angle(&platform_tr);
        let target_angle = hinge.target_angle.clamp(-PI + 0.01, PI);
        let diff = target_angle - angle;
        if diff.abs() > 0.001 {
            let rotation =
                Quat::from_axis_angle(hinge.axis, hinge.speed.min(diff.abs()) * diff.signum());
            platform_tr.rotate_around(anchor, rotation);
        } else {
            cmd.entity(entity).remove::<KinematicHingeMoving>();
        }
    }
}

fn add_joint_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    q_hinge: Query<
        (
            Entity,
            &Transform,
            &Handle<StandardMaterial>,
            &KinematicHinge,
        ),
        Added<KinematicHinge>,
    >,
    mut cmd: Commands,
) {
    for (entity, tr, material, hinge) in &q_hinge {
        cmd.entity(entity).with_children(|children| {
            let dir_y = tr.compute_affine().inverse().transform_vector3(hinge.axis);
            let dir_x = dir_y.any_orthonormal_vector();
            children.spawn(PbrBundle {
                transform: Transform::from_translation(hinge.anchor).with_rotation(
                    Quat::from_mat3(&Mat3::from_cols(
                        dir_x,
                        dir_y,
                        dir_x.cross(dir_y).normalize(),
                    )),
                ),
                mesh: meshes.add(Mesh::from(Cylinder {
                    radius: 0.1,
                    height: hinge.length,
                    resolution: 5,
                    segments: 1,
                })),
                material: material.clone(),
                ..default()
            });
        });
    }
}
