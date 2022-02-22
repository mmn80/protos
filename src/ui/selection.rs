use bevy::{pbr::NotShadowCaster, prelude::*};

use super::side_panel::SidePanelState;
use crate::{
    ai::{ground::Ground, pathfind::MovingPath},
    camera::ScreenPosition,
};

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
            .add_system(update_select_ui_rect)
            .add_system(update_nav_path_trails);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    commands.insert_resource(LoadedFont(asset_server.load("fonts/FiraMono-Medium.ttf")));
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
        let mouse_pos = windows.get_primary().unwrap().cursor_position();
        if input_mouse.just_pressed(MouseButton::Left) {
            selection_rect.begin = mouse_pos.map(|pos| Vec2::new(pos.x, pos.y));
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
    };

    if let Some(rect) = do_select_rect {
        for (
            entity,
            ScreenPosition {
                position,
                camera_dist: _,
            },
        ) in units_query.iter_mut()
        {
            if position.x > rect.left
                && position.x < rect.right
                && position.y < rect.top
                && position.y > rect.bottom
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

        for (entity, name, screen_pos) in added_q.iter() {
            let cam_fact = 1. / screen_pos.camera_dist;
            let text_ent = cmd
                .spawn_bundle(TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: Rect {
                            left: Val::Px(screen_pos.position.x - 50. - 200. * cam_fact),
                            right: Val::Auto,
                            top: Val::Auto,
                            bottom: Val::Px(screen_pos.position.y - 3000. * cam_fact),
                        },
                        ..Default::default()
                    },
                    text: Text::with_section(
                        name.to_string(),
                        text_style.clone(),
                        text_alignment.clone(),
                    ),
                    transform: Transform::from_scale(Vec3::ONE * (50. * cam_fact)),
                    ..Default::default()
                })
                .insert(UnitNameUiNode)
                .id();
            cmd.entity(entity).insert(UnitNameUiNodeRef(text_ent));
        }
    }

    for (entity, screen_pos, UnitNameUiNodeRef(ui_node)) in moved_q.iter() {
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

#[derive(Clone, Component, Debug, Default)]
pub struct NavPathTrail {
    path: Vec<Entity>,
}

#[derive(Clone, Component, Debug, Default)]
pub struct NavPathTrailElement;

fn update_nav_path_trails(
    mut meshes: ResMut<Assets<Mesh>>,
    ui: Res<SidePanelState>,
    ground: Res<Ground>,
    selected_query: Query<
        (Entity, &Handle<StandardMaterial>, &MovingPath),
        (With<Selected>, Without<NavPathTrail>),
    >,
    all_query: Query<(
        Entity,
        &NavPathTrail,
        Option<&Selected>,
        Option<&MovingPath>,
    )>,
    mut visibility_query: Query<&mut Visibility, With<NavPathTrailElement>>,
    mut cmd: Commands,
) {
    if ui.selected_show_path {
        for (entity, material, nav_path) in selected_query.iter() {
            let mesh = meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.2,
                subdivisions: 2,
            }));
            let path: Vec<_> = nav_path
                .path
                .iter()
                .map(|p| {
                    cmd.spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Transform::from_translation(Vec3::new(p.x, 0.2, p.z)),
                        ..Default::default()
                    })
                    .insert(NavPathTrailElement)
                    .insert(NotShadowCaster)
                    .id()
                })
                .collect();
            cmd.entity(ground.entity.unwrap()).push_children(&path);
            cmd.entity(entity).insert(NavPathTrail { path });
        }
    }
    for (entity, trail, selected, path) in all_query.iter() {
        if !ui.selected_show_path || selected.is_none() || path.is_none() {
            cmd.entity(entity).remove::<NavPathTrail>();
            for marker in &trail.path {
                cmd.entity(*marker).despawn_recursive();
            }
        } else if let Some(path) = path {
            for i in 0..path.current {
                let marker_ent = trail.path[i];
                visibility_query.get_mut(marker_ent).unwrap().is_visible = false;
            }
        }
    }
}
