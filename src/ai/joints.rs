use bevy::prelude::*;

pub struct JointsPlugin;

impl Plugin for JointsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HingeParent>()
            .register_type::<HingeChild>()
            .add_system(process_joints);
    }
}

#[derive(Component, Reflect)]
pub struct HingeChild {
    pub angle: f32,
    pub axis: Vec3,
    pub local_anchor_child: Vec3,
    pub local_anchor_parent: Vec3,
    pub parent: Entity,
}

#[derive(Component, Reflect)]
pub struct HingeParent;

fn process_joints(
    q_parent: Query<&GlobalTransform, (With<HingeParent>, Changed<GlobalTransform>)>,
    mut q_child: Query<(&mut Transform, &GlobalTransform, &HingeChild)>,
) {
    for (mut child_tr, child_global_tr, child_h) in &mut q_child {
        if let Ok(parent_tr) = q_parent.get(child_h.parent) {
            let parent_anchor = parent_tr.transform_point(child_h.local_anchor_parent);
            let child_anchor = child_global_tr.transform_point(child_h.local_anchor_child);
            let diff = parent_anchor - child_anchor;
            if diff.length() > 0.001 {
                child_tr.translation += diff;
            }
        }
    }
}
