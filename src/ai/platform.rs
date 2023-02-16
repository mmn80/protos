use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use crate::{camera::MainCamera, ui::side_panel::SidePanelState};

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AddPlatformUi::default())
            .add_system(add_platform);
    }
}

#[derive(PartialEq)]
enum AddPlatformState {
    SelectingRectStart,
    SelectingRectEnd,
    SelectingDepth,
}

#[derive(Resource)]
struct AddPlatformUi {
    pub state: AddPlatformState,
    pub attach_p0: Option<Vec3>,
    pub attach_p0_normal: Option<Vec3>,
    pub attach_p1: Option<Vec3>,
}

impl Default for AddPlatformUi {
    fn default() -> Self {
        Self {
            state: AddPlatformState::SelectingRectStart,
            attach_p0: None,
            attach_p0_normal: None,
            attach_p1: None,
        }
    }
}

fn add_platform(
    ui_global: Res<SidePanelState>,
    mut ui: ResMut<AddPlatformUi>,
    input_mouse: Res<Input<MouseButton>>,
    rapier_ctx: Res<RapierContext>,
    camera_q: Query<&MainCamera>,
) {
    if ui_global.add_platform {
        if input_mouse.just_pressed(MouseButton::Right) {
            ui.state = AddPlatformState::SelectingRectStart;
            ui.attach_p0 = None;
            ui.attach_p0_normal = None;
            ui.attach_p1 = None;
            return;
        }
        let ray = {
            if let Ok(camera) = camera_q.get_single() {
                if let Some(ray) = camera.mouse_ray.clone() {
                    ray
                } else {
                    return;
                }
            } else {
                return;
            }
        };
        if ui.state == AddPlatformState::SelectingRectStart {
            if input_mouse.just_pressed(MouseButton::Left) {
                if let Some((_entity, intersection)) = rapier_ctx.cast_ray_and_get_normal(
                    ray.origin,
                    ray.direction,
                    1000.,
                    false,
                    QueryFilter::new(),
                ) {
                    println!(
                        "Rect started at point {} with normal {}",
                        intersection.point, intersection.normal
                    );
                    ui.attach_p0 = Some(intersection.point);
                    ui.attach_p0_normal = Some(intersection.normal);
                    ui.state = AddPlatformState::SelectingRectEnd;
                }
            }
        } else if ui.state == AddPlatformState::SelectingRectEnd {
            let center = ui.attach_p0.unwrap();
            let normal = ui.attach_p0_normal.unwrap();
            let ray_parry = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());
            if let Some(toi) = ray_toi_with_halfspace(&center.into(), &normal.into(), &ray_parry) {
                let p1 = ray.origin + toi * ray.direction;
                ui.attach_p1 = Some(p1);

                if input_mouse.just_pressed(MouseButton::Left) {
                    println!("Rect completed at point {}", ui.attach_p1.unwrap());
                    ui.state = AddPlatformState::SelectingDepth;
                }
            }
        }
    }
}
