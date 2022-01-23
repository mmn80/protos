use bevy::prelude::*;

pub struct MainLightsPlugin {
    dir_light_size: f32,
    dir_light_color: Color,
}

impl Default for MainLightsPlugin {
    fn default() -> Self {
        MainLightsPlugin {
            dir_light_size: 100.0,
            dir_light_color: Color::ORANGE_RED,
        }
    }
}

pub struct MainLightsState {
    pub dir_light_size: f32,
    pub dir_light_color: Color,
}

impl Plugin for MainLightsPlugin {
    fn build(&self, app: &mut App) {
        let state = MainLightsState {
            dir_light_size: self.dir_light_size,
            dir_light_color: self.dir_light_color,
        };
        app.insert_resource(state)
            .add_startup_system(spawn_main_lights)
            .add_system(animate_light_direction);
    }
}

fn spawn_main_lights(mut commands: Commands, state: Res<MainLightsState>) {
    // ambient light
    commands.insert_resource(AmbientLight {
        color: state.dir_light_color,
        brightness: 0.02,
    });
    // directional 'sun' light
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            shadow_projection: OrthographicProjection {
                left: -state.dir_light_size,
                right: state.dir_light_size,
                bottom: -state.dir_light_size,
                top: state.dir_light_size,
                near: -10.0 * state.dir_light_size,
                far: 10.0 * state.dir_light_size,
                ..Default::default()
            },
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn animate_light_direction(
    time: Res<Time>,
    state: Res<MainLightsState>,
    mut query: Query<(&mut Transform, &mut DirectionalLight)>,
) {
    for (mut transform, mut light) in query.iter_mut() {
        light.shadow_projection = OrthographicProjection {
            left: -state.dir_light_size,
            right: state.dir_light_size,
            bottom: -state.dir_light_size,
            top: state.dir_light_size,
            near: -10.0 * state.dir_light_size,
            far: 10.0 * state.dir_light_size,
            ..Default::default()
        };
        transform.rotate(Quat::from_rotation_y(time.delta_seconds() * 0.5));
    }
}

pub const INFINITE_TEMP_COLOR: Color = Color::rgb(
    148. / u8::MAX as f32,
    177. / u8::MAX as f32,
    255. / u8::MAX as f32,
);
