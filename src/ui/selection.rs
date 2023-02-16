use bevy::prelude::*;

use super::side_panel::SidePanelState;
use crate::camera::ScreenPosition;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_event::<DeselectedEvent>()
            .insert_resource(SelectionRect::default())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_units_selected.after("update_screen_position"),
            )
            .add_system(update_selected_unit_names)
            .add_system(update_select_ui_rect);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    //commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    background_color: Color::rgba(0.1, 0.8, 0.1, 0.1).into(),
                    visibility: Visibility { is_visible: false },
                    ..default()
                },
                SelectionRectUiNode,
            ));
        });
    commands.insert_resource(LoadedFont(asset_server.load("fonts/FiraMono-Medium.ttf")));
}

#[derive(Clone, Component, Debug, Default)]
pub struct Selectable;

#[derive(Clone, Component, Debug, Default)]
pub struct Selected;

#[derive(Clone, Component, Debug, Default)]
pub struct SelectionRectUiNode;

#[derive(Debug, Clone, Default, Resource)]
pub struct SelectionRect {
    pub clear_previous: bool,
    pub begin: Option<Vec2>,
    pub end: Option<Vec2>,
}

impl SelectionRect {
    pub fn get_rect(&self) -> Option<Rect> {
        if let (Some(begin), Some(end)) = (self.begin, self.end) {
            Some(Rect::from_corners(begin, end))
        } else {
            None
        }
    }
}

struct DeselectedEvent(Entity);

fn update_units_selected(
    mut selection_rect: ResMut<SelectionRect>,
    keyboard: Res<Input<KeyCode>>,
    input_mouse: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut egui_ctx: ResMut<bevy_egui::EguiContext>,
    mut units_query: Query<(Entity, &ScreenPosition), With<Selectable>>,
    mut ev_deselected: EventWriter<DeselectedEvent>,
    mut cmd: Commands,
) {
    if egui_ctx.ctx_mut().wants_pointer_input() {
        return;
    }
    let do_select_rect = {
        selection_rect.clear_previous = !keyboard.pressed(KeyCode::LShift);
        if let Some(window) = windows.get_primary() {
            let mouse_pos = window.cursor_position();
            if input_mouse.just_pressed(MouseButton::Left) {
                selection_rect.begin = mouse_pos.clone();
                selection_rect.end = selection_rect.begin;
                // info!("start selecting at {begin:?}", begin = selection_rect.begin);
            } else if selection_rect.begin.is_some() {
                if input_mouse.pressed(MouseButton::Left) && mouse_pos.is_some() {
                    selection_rect.end = Some(mouse_pos.unwrap());
                } else if !input_mouse.just_released(MouseButton::Left) || mouse_pos.is_none() {
                    // info!("cancel selecting at {end:?}", end = selection_rect.end);
                    selection_rect.begin = None;
                    selection_rect.end = None;
                }
            }
            if input_mouse.just_released(MouseButton::Left) {
                // info!("end selecting at {end:?}", end = selection_rect.end);
                selection_rect.get_rect()
            } else {
                None
            }
        } else {
            return;
        }
    };

    if let Some(rect) = do_select_rect {
        for (
            entity,
            ScreenPosition {
                position,
                camera_dist: _,
            },
        ) in &mut units_query
        {
            if position.x > rect.min.x
                && position.x < rect.max.x
                && position.y < rect.max.y
                && position.y > rect.min.y
            {
                cmd.entity(entity).insert(Selected);
            } else if selection_rect.clear_previous {
                cmd.entity(entity).remove::<Selected>();
                ev_deselected.send(DeselectedEvent(entity));
            }
        }
        selection_rect.begin = None;
        selection_rect.end = None;
    }
}

fn update_select_ui_rect(
    selection_rect: Res<SelectionRect>,
    windows: Res<Windows>,
    mut ui_query: Query<(&mut Style, &mut Visibility), With<SelectionRectUiNode>>,
) {
    if let Some(window) = windows.get_primary() {
        let window_height = window.height();
        for (mut style, mut visibility) in &mut ui_query {
            if let Some(rect) = selection_rect.get_rect() {
                style.size.width = Val::Px(rect.width());
                style.size.height = Val::Px(rect.height());
                style.position.left = Val::Px(rect.min.x);
                style.position.right = Val::Px(rect.max.x);
                style.position.bottom = Val::Px(window_height - rect.min.y);
                style.position.top = Val::Px(window_height - rect.max.y);
                visibility.is_visible = true;
            } else {
                visibility.is_visible = false;
            }
        }
    }
}

#[derive(Resource)]
struct LoadedFont(Handle<Font>);

#[derive(Clone, Component, Debug, Default)]
pub struct UnitNameUiNode;

#[derive(Clone, Component, Debug)]
pub struct UnitNameUiNodeRef(Entity);

fn update_selected_unit_names(
    panel: Res<SidePanelState>,
    loaded_font: Res<LoadedFont>,
    added_q: Query<(Entity, &Name, &ScreenPosition), Added<Selected>>,
    moved_q: Query<(Entity, &ScreenPosition, &UnitNameUiNodeRef)>,
    mut nodes_q: Query<(&mut Transform, &mut Style), With<UnitNameUiNode>>,
    mut ev_deselected: EventReader<DeselectedEvent>,
    mut cmd: Commands,
) {
    if panel.selected_show_names {
        let text_alignment = TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        };
        let text_style = TextStyle {
            font: loaded_font.0.clone(),
            font_size: 20.0,
            color: Color::SILVER,
        };

        for (entity, name, screen_pos) in &added_q {
            let cam_fact = 1. / screen_pos.camera_dist;
            let text_ent = cmd
                .spawn((
                    TextBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                left: Val::Px(screen_pos.position.x - 50. - 200. * cam_fact),
                                right: Val::Auto,
                                top: Val::Auto,
                                bottom: Val::Px(screen_pos.position.y - 3000. * cam_fact),
                            },
                            ..default()
                        },
                        text: Text::from_section(name.to_string(), text_style.clone())
                            .with_alignment(text_alignment.clone()),
                        transform: Transform::from_scale(Vec3::ONE * (50. * cam_fact)),
                        ..default()
                    },
                    UnitNameUiNode,
                ))
                .id();
            cmd.entity(entity).insert(UnitNameUiNodeRef(text_ent));
        }
    }

    for (entity, screen_pos, UnitNameUiNodeRef(ui_node)) in &moved_q {
        if panel.selected_show_names {
            if let Ok((mut transform, mut style)) = nodes_q.get_mut(*ui_node) {
                let cam_fact = 1. / screen_pos.camera_dist;
                style.position.left = Val::Px(screen_pos.position.x - 50. - 200. * cam_fact);
                style.position.bottom = Val::Px(screen_pos.position.y - 3000. * cam_fact);
                transform.scale = Vec3::ONE * (50. * cam_fact);
            }
        } else {
            cmd.entity(*ui_node).despawn_recursive();
            cmd.entity(entity).remove::<UnitNameUiNodeRef>();
        }
    }

    for DeselectedEvent(unit_ent) in ev_deselected.iter() {
        if let Ok((_, _, UnitNameUiNodeRef(ui_node))) = moved_q.get(*unit_ent) {
            cmd.entity(*ui_node).despawn_recursive();
            cmd.entity(*unit_ent).remove::<UnitNameUiNodeRef>();
        }
    }
}
