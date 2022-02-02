use bevy::prelude::*;

use crate::camera::ScreenPosition;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .insert_resource(SelectionRect::default())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_units_selected.after("update_screen_position"),
            )
            .add_system(update_select_ui_rect);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        ..Default::default()
                    },
                    color: Color::rgba(0.1, 0.8, 0.1, 0.1).into(),
                    visibility: Visibility { is_visible: false },
                    ..Default::default()
                })
                .insert(SelectionRectUiNode);
        });
}

#[derive(Clone, Component, Debug, Default)]
pub struct Selectable;

#[derive(Clone, Component, Debug, Default)]
pub struct Selected;

#[derive(Clone, Component, Debug, Default)]
pub struct SelectionRectUiNode;

#[derive(Debug, Clone, Default)]
pub struct SelectionRect {
    pub clear_previous: bool,
    pub begin: Option<Vec2>,
    pub end: Option<Vec2>,
}

impl SelectionRect {
    pub fn get_rect(&self) -> Option<Rect<f32>> {
        if let (Some(begin), Some(end)) = (self.begin, self.end) {
            Some(Rect {
                left: f32::min(begin.x, end.x),
                right: f32::max(begin.x, end.x),
                top: f32::max(begin.y, end.y),
                bottom: f32::min(begin.y, end.y),
            })
        } else {
            None
        }
    }
}

fn update_units_selected(
    mut selection_rect: ResMut<SelectionRect>,
    keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    egui_ctx: ResMut<bevy_egui::EguiContext>,
    mut units_query: Query<(Entity, &ScreenPosition), With<Selectable>>,
    mut cmd: Commands,
) {
    if egui_ctx.ctx().wants_pointer_input() {
        return;
    }
    let do_select_rect = {
        selection_rect.clear_previous = !keyboard.pressed(KeyCode::LShift);
        let mouse_pos = windows.get_primary().unwrap().cursor_position();
        if input_mouse.just_pressed(MouseButton::Left) {
            selection_rect.begin = mouse_pos.map(|pos| Vec2::new(pos.x, pos.y));
            selection_rect.end = selection_rect.begin;
            // info!("start selecting at {:?}", selection.begin);
        } else if selection_rect.begin.is_some() {
            if input_mouse.pressed(MouseButton::Left) && mouse_pos.is_some() {
                selection_rect.end = Some(mouse_pos.unwrap());
            } else if !input_mouse.just_released(MouseButton::Left) || mouse_pos.is_none() {
                // info!("cancel selecting at {:?}", selection.end);
                selection_rect.begin = None;
                selection_rect.end = None;
            }
        }
        if input_mouse.just_released(MouseButton::Left) {
            // info!("end selecting at {:?}", selection.end);
            selection_rect.get_rect()
        } else {
            None
        }
    };

    if let Some(rect) = do_select_rect {
        for (entity, ScreenPosition { position }) in units_query.iter_mut() {
            if position.x > rect.left
                && position.x < rect.right
                && position.y < rect.top
                && position.y > rect.bottom
            {
                cmd.entity(entity).insert(Selected);
            } else if selection_rect.clear_previous {
                cmd.entity(entity).remove::<Selected>();
            }
        }
        selection_rect.begin = None;
        selection_rect.end = None;
    }
}

fn update_select_ui_rect(
    selection_rect: Res<SelectionRect>,
    mut ui_query: Query<(&mut Style, &mut Visibility), With<SelectionRectUiNode>>,
) {
    for (mut style, mut visibility) in ui_query.iter_mut() {
        if let Some(rect) = selection_rect.get_rect() {
            style.size.width = Val::Px(rect.right - rect.left);
            style.size.height = Val::Px(rect.top - rect.bottom);
            style.position.left = Val::Px(rect.left);
            style.position.right = Val::Px(rect.right);
            style.position.bottom = Val::Px(rect.bottom);
            style.position.top = Val::Px(rect.top);
            visibility.is_visible = true;
        } else {
            visibility.is_visible = false;
        }
    }
}