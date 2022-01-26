use bevy::prelude::*;

use crate::camera::ScreenPosition;

pub struct MultiSelectPlugin;

impl Plugin for MultiSelectPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectionRect::default())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_units_selected.after("update_screen_position"),
            );
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectionRect {
    pub clear_previous: bool,
    pub rect: Option<Rect<f32>>,
}

#[derive(Clone, Component, Debug, Default)]
pub struct Selected {
    pub selected: bool,
}

fn update_units_selected(
    mut selection_rect: ResMut<SelectionRect>,
    keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut units_query: Query<(&ScreenPosition, &mut Selected)>,
) {
    let do_select_rect = {
        selection_rect.clear_previous = !keyboard.pressed(KeyCode::LShift);
        let mouse_pos = windows.get_primary().unwrap().cursor_position();
        if input_mouse.just_pressed(MouseButton::Left) {
            selection_rect.rect = mouse_pos.map(|pos| Rect {
                left: pos.x,
                right: pos.x,
                top: pos.y,
                bottom: pos.y,
            });
            // info!("start selecting at {:?}", selection_rect.rect);
        } else if let Some(mut rect) = selection_rect.rect {
            if input_mouse.pressed(MouseButton::Left) && mouse_pos.is_some() {
                let pos = mouse_pos.unwrap();
                rect.right = pos.x;
                rect.bottom = pos.y;
                selection_rect.rect = Some(rect);
            } else if !input_mouse.just_released(MouseButton::Left) || mouse_pos.is_none() {
                // info!("cancel selecting at {:?}", selection_rect.rect);
                selection_rect.rect = None;
            }
        }
        if input_mouse.just_released(MouseButton::Left) {
            // info!("end selecting at {:?}", selection_rect.rect);
            selection_rect.rect
        } else {
            None
        }
    };
    if let Some(rect) = do_select_rect {
        for (ScreenPosition { position }, mut selected) in units_query.iter_mut() {
            let left = f32::min(rect.left, rect.right);
            let right = f32::max(rect.left, rect.right);
            let top = f32::min(rect.top, rect.bottom);
            let bottom = f32::max(rect.top, rect.bottom);
            if position.x > left && position.x < right && position.y > top && position.y < bottom {
                selected.selected = true;
            } else if selection_rect.clear_previous {
                selected.selected = false;
            }
        }
        selection_rect.rect = None;
    }
}
