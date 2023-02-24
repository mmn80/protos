use bevy::{prelude::*, transform::TransformSystem};
use std::f32::consts::TAU;

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
        self.start_dir_up.angle_between(transform.up())
    }
}

fn process_joints(mut q_child: Query<(&mut Transform, &KinematicHinge)>) {
    for (mut child_tr, child_h) in &mut q_child {
        let anchor = child_tr.transform_point(child_h.anchor);
        let angle = child_h.get_angle(&child_tr);
        let target_angle = child_h.target_angle.clamp(0., TAU);
        let diff = target_angle - angle;
        if diff.abs() > 0.001 {
            let rotation =
                Quat::from_axis_angle(child_h.axis, child_h.speed.min(diff.abs()) * diff.signum());
            child_tr.rotate_around(anchor, rotation);
        }
    }
}
