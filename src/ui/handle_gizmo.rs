use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use parry3d::query::details::ray_toi_with_halfspace;

use super::{
    selection::Selected,
    side_panel::{SidePanelState, UiMode},
};
use crate::{camera::MainCamera, mesh::cone::Cone};

pub struct HandleGizmoPlugin;

impl Plugin for HandleGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HandleGizmosRes::default())
            .add_startup_system(setup_gizmo);
    }
}

#[derive(Resource)]
struct HandleGizmosRes {
    pub bar: Option<Handle<Mesh>>,
    pub cone: Option<Handle<Mesh>>,
    pub active_gizmo: Option<Entity>,
    pub drag_start_y: Option<f32>,
    pub drag_start_pos: Option<Vec3>,
}

impl Default for HandleGizmosRes {
    fn default() -> Self {
        Self {
            bar: None,
            cone: None,
            active_gizmo: None,
            drag_start_y: None,
            drag_start_pos: None,
        }
    }
}

const BAR_H: f32 = 2.0;
const BAR_W: f32 = 0.1;
const CONE_W: f32 = 0.8;
const CONE_H: f32 = 1.0;

fn setup_gizmo(mut res: ResMut<HandleGizmosRes>, mut meshes: ResMut<Assets<Mesh>>) {
    res.bar = Some(meshes.add(Mesh::from(shape::Box::new(BAR_W, BAR_H, BAR_W))));
    res.cone = Some(meshes.add(Mesh::from(Cone::new(CONE_W / 2., CONE_H, 10))));
}
