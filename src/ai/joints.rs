use bevy::{prelude::*, transform::TransformSystem};

pub struct JointsPlugin;

impl Plugin for JointsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicHinge>().add_system_to_stage(
            CoreStage::PostUpdate,
            process_joints.after(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Component, Reflect)]
pub struct KinematicHinge {
    pub angle: f32,
    pub axis: Vec3,
    pub anchor: Vec3,
}

fn process_joints(mut q_child: Query<(&mut Transform, &GlobalTransform, &KinematicHinge)>) {
    for (mut child_tr, child_gtr, child_h) in &mut q_child {
        // let parent_anchor = parent_gtr.transform_point(child_h.local_anchor_parent);
        // let child_anchor = child_gtr.transform_point(child_h.local_anchor_child);
        // let diff = parent_anchor - child_anchor;
        // child_tr.translation += diff;

        // let parent_axis = parent_gtr
        //     .affine()
        //     .transform_vector3(child_h.local_axis_parent)
        //     .normalize();
        // let child_axis = child_gtr
        //     .affine()
        //     .transform_vector3(child_h.local_axis_child)
        //     .normalize();
        // let rotation = Quat::from_rotation_arc(child_axis, parent_axis);
        // child_tr.rotate(rotation);
    }
}
