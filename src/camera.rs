use bevy::{
    core_pipeline::bloom::BloomSettings,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use crate::ui::{selection::Selected, side_panel::SidePanelState};

pub struct MainCameraPlugin;

impl Plugin for MainCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_system(update_screen_position.in_base_set(CoreSet::PreUpdate))
            .add_system(main_camera);
    }
}

#[derive(Component)]
pub struct MainCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub mouse_ray: Option<Ray>,
}

const START_DIST: f32 = 40.0;

impl Default for MainCamera {
    fn default() -> Self {
        MainCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
            mouse_ray: None,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(-START_DIST, START_DIST, START_DIST);
    let radius = translation.length();

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        BloomSettings::default(),
        //bevy::core_pipeline::fxaa::Fxaa::default(),
        MainCamera {
            radius,
            ..default()
        },
    ));
}

/// Move with WASD, zoom with scroll wheel, orbit with right mouse click.
fn main_camera(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    ui: Res<SidePanelState>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut ev_cursor: EventReader<CursorMoved>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(&mut MainCamera, &mut Transform, &GlobalTransform, &Camera)>,
    q_selection: Query<&GlobalTransform, With<Selected>>,
) {
    let orbit_button = MouseButton::Right;

    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if !ui.mouse_over {
        if mouse.pressed(orbit_button) {
            for ev in ev_motion.iter() {
                rotation_move += ev.delta;
            }
        }
        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }
        if mouse.just_released(orbit_button) || mouse.just_pressed(orbit_button) {
            orbit_button_changed = true;
        }
    }

    let cursor_pos = ev_cursor.iter().last().map(|p| p.position);

    for (mut main_camera, mut camera_tr, camera_gtr, camera) in &mut q_camera {
        if let Some(pos) = cursor_pos {
            main_camera.mouse_ray = get_camera_mouse_ray(pos, camera, camera_gtr);
        }

        if keyboard.any_pressed([KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D]) {
            let mut ds = time.delta_seconds() * 10.;
            if keyboard.pressed(KeyCode::LShift) {
                ds *= 4.;
            }
            let mut forward = camera_tr.forward();
            if forward.x.abs() < f32::EPSILON && forward.z.abs() < f32::EPSILON {
                forward = camera_tr.up();
            }
            forward.y = 0.;
            forward = forward.normalize();
            let right = camera_tr.right().normalize();

            if keyboard.pressed(KeyCode::W) {
                main_camera.focus += ds * forward;
                camera_tr.translation += ds * forward;
            } else if keyboard.pressed(KeyCode::S) {
                main_camera.focus -= ds * forward;
                camera_tr.translation -= ds * forward;
            }
            if keyboard.pressed(KeyCode::A) {
                main_camera.focus -= ds * right;
                camera_tr.translation -= ds * right;
            } else if keyboard.pressed(KeyCode::D) {
                main_camera.focus += ds * right;
                camera_tr.translation += ds * right;
            }
        } else if keyboard.just_pressed(KeyCode::F) {
            let current_focus = main_camera.focus;
            main_camera.focus = Vec3::ZERO;
            let selected: Vec<_> = q_selection.iter().map(|gtr| gtr.translation()).collect();
            if !selected.is_empty() {
                main_camera.focus = selected.iter().sum::<Vec3>() / selected.len() as f32;
            }
            camera_tr.translation += main_camera.focus - current_focus;
        }

        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = camera_tr.rotation * Vec3::Y;
            main_camera.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = {
                let window = q_window.single();
                Vec2::new(window.width(), window.height())
            };
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if main_camera.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            camera_tr.rotation = yaw * camera_tr.rotation; // rotate around global y axis
            camera_tr.rotation = camera_tr.rotation * pitch; // rotate around local x axis
        } else if scroll.abs() > 0.0 {
            any = true;
            main_camera.radius -= scroll * main_camera.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            main_camera.radius = f32::max(main_camera.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(camera_tr.rotation);
            camera_tr.translation =
                main_camera.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, main_camera.radius));
        }
    }
}

#[derive(Clone, Component, Debug, Default)]
pub struct ScreenPosition {
    pub position: Vec2,
    pub camera_dist: f32,
}

pub fn update_screen_position(
    q_camera: Query<(&GlobalTransform, &Camera), With<MainCamera>>,
    mut q_selectable: Query<(&GlobalTransform, &mut ScreenPosition)>,
) {
    let Some((camera_transform, camera)) = q_camera.iter().next() else { return };
    for (transform, mut screen_position) in &mut q_selectable {
        let Some(pos) = camera.world_to_viewport(camera_transform, transform.translation()) else { continue };
        screen_position.position = pos;
        screen_position.camera_dist =
            (transform.translation() - camera_transform.translation()).length();
    }
}

fn get_camera_mouse_ray(
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray> {
    let view = camera_transform.compute_matrix();

    let (viewport_min, viewport_max) = camera.logical_viewport_rect()?;
    let screen_size = camera.logical_target_size()?;
    let viewport_size = viewport_max - viewport_min;
    let adj_cursor_pos =
        cursor_pos_screen - Vec2::new(viewport_min.x, screen_size.y - viewport_max.y);

    let projection = camera.projection_matrix();
    let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
    let near_ndc = projection.project_point3(Vec3::Z).z;
    let cursor_ndc = (adj_cursor_pos / viewport_size) * 2.0 - Vec2::ONE;
    let ndc_to_world: Mat4 = view * projection.inverse();
    let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
    let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
    let ray_direction = far - near;
    Some(Ray {
        origin: near,
        direction: ray_direction,
    })
}
