use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};
use bevy_egui::EguiContext;
use bevy_mod_raycast::RayCastSource;

use crate::{ai::ground::GroundRaycastSet, light::MainLightsState};

pub struct MainCameraPlugin;

impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_screen_position.label("update_screen_position"),
            )
            .add_system(main_camera);
    }
}

#[derive(Component)]
pub struct MainCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

const START_DIST: f32 = 30.0;

impl Default for MainCamera {
    fn default() -> Self {
        MainCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(-START_DIST, START_DIST, START_DIST);
    let radius = translation.length();

    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(MainCamera {
            radius,
            ..default()
        })
        .insert(RayCastSource::<GroundRaycastSet>::new());
}

/// Move with WASD, zoom with scroll wheel, orbit with right mouse click.
fn main_camera(
    windows: Res<Windows>,
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut light: ResMut<MainLightsState>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut MainCamera, &mut Transform)>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    let orbit_button = MouseButton::Right;

    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if !egui_ctx.ctx_mut().wants_pointer_input() {
        if input_mouse.pressed(orbit_button) {
            for ev in ev_motion.iter() {
                rotation_move += ev.delta;
            }
        }
        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }
        if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
            orbit_button_changed = true;
        }
    }

    for (mut camera, mut transform) in &mut query {
        if keyboard.any_pressed([KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D]) {
            let mut ds = time.delta_seconds() * 10.;
            if keyboard.pressed(KeyCode::LShift) {
                ds *= 4.;
            }
            let mut forward = transform.forward();
            if forward.x.abs() < f32::EPSILON && forward.z.abs() < f32::EPSILON {
                forward = transform.up();
            }
            forward.y = 0.;
            forward = forward.normalize();
            let right = transform.right().normalize();

            if keyboard.pressed(KeyCode::W) {
                camera.focus += ds * forward;
                transform.translation += ds * forward;
            } else if keyboard.pressed(KeyCode::S) {
                camera.focus -= ds * forward;
                transform.translation -= ds * forward;
            }
            if keyboard.pressed(KeyCode::A) {
                camera.focus -= ds * right;
                transform.translation -= ds * right;
            } else if keyboard.pressed(KeyCode::D) {
                camera.focus += ds * right;
                transform.translation += ds * right;
            }
        }

        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            camera.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if camera.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if scroll.abs() > 0.0 {
            any = true;
            camera.radius -= scroll * camera.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            camera.radius = f32::max(camera.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                camera.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, camera.radius));
        }

        light.dir_light_size = transform.translation.y * 100. / START_DIST;
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

#[derive(Clone, Component, Debug, Default)]
pub struct ScreenPosition {
    pub position: Vec2,
    pub camera_dist: f32,
}

fn update_screen_position(
    camera_query: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    mut units_query: Query<(&GlobalTransform, &mut ScreenPosition)>,
) {
    for (camera_transform, camera) in &camera_query {
        for (transform, mut screen_position) in &mut units_query {
            if let Some(pos) = camera.world_to_viewport(camera_transform, transform.translation()) {
                screen_position.position = pos;
                screen_position.camera_dist =
                    (transform.translation() - camera_transform.translation()).length();
            }
        }
        break;
    }
}
