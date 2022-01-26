use bevy::prelude::*;

pub struct SlowUnitPlugin;

impl Plugin for SlowUnitPlugin {
    fn build(&self, _app: &mut App) {
        // app.insert_resource(Ground::new(1024))
        //     .add_plugin(DefaultRaycastingPlugin::<GroundRaycastSet>::default())
        //     .add_startup_system(setup)
        //     .add_system_set_to_stage(
        //         CoreStage::PreUpdate,
        //         SystemSet::new()
        //             .with_system(update_ground_raycast.before(RaycastSystem::BuildRays))
        //             .with_system(ground_painter.after(RaycastSystem::UpdateRaycast)),
        //     )
        //     .add_system(update_ground_texture);
    }
}
