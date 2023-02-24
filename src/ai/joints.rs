use bevy::{prelude::*, transform::TransformSystem};
use std::f32::consts::PI;

pub struct JointsPlugin;

impl Plugin for JointsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicHinge>().add_system_to_stage(
            CoreStage::PostUpdate,
            process_joints.before(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Component, Reflect)]
pub struct KinematicHinge {
    pub target_angle: f32,
    pub axis: Vec3,
    pub anchor: Vec3,
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
