use bevy::prelude::*;

use super::{
    basic_materials::BasicMaterialsRes,
    handle_gizmo::{AddHandleGizmo, HandleGizmoAxis, HandleGizmoDragged, RemoveHandleGizmo},
    selection::Selected,
    side_panel::{SidePanelState, UiMode},
};

pub struct MoveGizmoPlugin;

impl Plugin for MoveGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_move_gizmos);
    }
}

#[derive(Component)]
pub struct HasMoveGizmos;

fn update_move_gizmos(
    mut ui: ResMut<SidePanelState>,
    materials: Res<BasicMaterialsRes>,
    mut ev_add: EventWriter<AddHandleGizmo>,
    mut ev_remove: EventWriter<RemoveHandleGizmo>,
    mut ev_drag: EventReader<HandleGizmoDragged>,
    q_selected: Query<Entity, With<Selected>>,
    mut q_gizmo: Query<(Entity, &mut Transform, &GlobalTransform), With<HasMoveGizmos>>,
    mut cmd: Commands,
) {
    if ui.selected_show_move_gizmo {
        for sel in &q_selected {
            if !q_gizmo.contains(sel) {
                for (axis, material) in [
                    (HandleGizmoAxis::X, materials.ui_red.clone()),
                    (HandleGizmoAxis::Y, materials.ui_green.clone()),
                    (HandleGizmoAxis::Z, materials.ui_blue.clone()),
                ] {
                    ev_add.send(AddHandleGizmo {
                        entity: sel,
                        axis,
                        material,
                    });
                    cmd.entity(sel).insert(HasMoveGizmos);
                }
            }
        }

        for (entity, _, _) in &q_gizmo {
            if !q_selected.contains(entity) {
                for axis in [HandleGizmoAxis::X, HandleGizmoAxis::Y, HandleGizmoAxis::Z] {
                    ev_remove.send(RemoveHandleGizmo { entity, axis });
                }
                cmd.entity(entity).remove::<HasMoveGizmos>();
            }
        }

        for HandleGizmoDragged {
            entity,
            axis: _,
            direction,
            drag_delta,
        } in ev_drag.iter()
        {
            if let Ok((_, mut trans, global_trans)) = q_gizmo.get_mut(*entity) {
                trans.translation += global_trans.transform_point(*drag_delta * *direction);
                ui.mode = UiMode::Select;
            }
        }
    } else {
        for (entity, _, _) in &q_gizmo {
            for axis in [HandleGizmoAxis::X, HandleGizmoAxis::Y, HandleGizmoAxis::Z] {
                ev_remove.send(RemoveHandleGizmo { entity, axis });
            }
            cmd.entity(entity).remove::<HasMoveGizmos>();
        }
    }
}
