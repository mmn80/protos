use bevy::prelude::*;
use bevy_mod_picking::Selection;
use rand::{thread_rng, Rng};

use crate::ui::UiState;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(random_walk);
    }
}

pub fn random_walk(ui: Res<UiState>, mut query: Query<(&Selection, &mut Transform)>) {
    if ui.random_walk_selected || ui.random_walk_all {
        let mut rng = thread_rng();
        for (sel, mut transform) in query.iter_mut() {
            if ui.random_walk_all || sel.selected() {
                transform.translation +=
                    Vec3::new(rng.gen_range(-0.1..0.1), 0., rng.gen_range(-0.1..0.1));
            }
        }
    }
}
