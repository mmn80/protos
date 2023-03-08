use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};

pub struct MainLightsPlugin;

impl Plugin for MainLightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_main_lights)
            .add_system(animate_light_direction);
    }
}

fn spawn_main_lights(mut commands: Commands) {
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });
    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 1.0,
            maximum_distance: 1000.0,
            first_cascade_far_bound: 100.0,
            overlap_proportion: 0.2,
        }
        .build(),
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotate(Quat::from_rotation_y(time.delta_seconds() * 0.1));
    }
}

pub const INFINITE_TEMP_COLOR: Color = Color::rgb_linear(
    148. / u8::MAX as f32,
    177. / u8::MAX as f32,
    255. / u8::MAX as f32,
);
