use bevy::prelude::*;

pub struct MainLightsPlugin;

impl Plugin for MainLightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_main_lights)
            .add_system(animate_light_direction);
    }
}

const LIGHT_SZ: f32 = 100.;

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
            shadow_projection: OrthographicProjection {
                left: -LIGHT_SZ,
                right: LIGHT_SZ,
                bottom: -LIGHT_SZ,
                top: LIGHT_SZ,
                near: -10.0 * LIGHT_SZ,
                far: 10.0 * LIGHT_SZ,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
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

pub const INFINITE_TEMP_COLOR: Color = Color::rgb(
    148. / u8::MAX as f32,
    177. / u8::MAX as f32,
    255. / u8::MAX as f32,
);
