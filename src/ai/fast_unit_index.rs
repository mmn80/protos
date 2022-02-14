use bevy::{prelude::*, tasks::ComputeTaskPool};
use big_brain::{thinker::HasThinker, BigBrainStage};
use kiddo::{distance::squared_euclidean, KdTree};

pub struct FastUnitIndexPlugin;

impl Plugin for FastUnitIndexPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FastUnitIndex::new())
            .add_stage_before(
                BigBrainStage::Scorers,
                "update_grid",
                SystemStage::parallel(),
            )
            .add_system_to_stage("update_grid", update_grid.label("update_grid_system"))
            .add_system_to_stage("update_grid", find_neighbours.after("update_grid_system"));
    }
}

pub struct FastUnitIndex {
    pub grid: KdTree<f32, Entity, 2>,
}

impl FastUnitIndex {
    pub fn new() -> Self {
        Self {
            grid: KdTree::new(),
        }
    }
}

pub fn update_grid(
    mut res: ResMut<FastUnitIndex>,
    query: Query<(Entity, &Transform), With<HasThinker>>,
) {
    //let start = std::time::Instant::now();
    res.grid = KdTree::new();
    for (entity, transform) in query.iter() {
        res.grid
            .add(&[transform.translation.x, transform.translation.z], entity)
            .ok();
    }
    // let dt = (std::time::Instant::now() - start).as_micros();
    // info!("grid construction time: {dt}μs, len={}", res.grid.size());
}

#[derive(Clone, Debug)]
pub struct Neighbour {
    pub entity: Entity,
    pub distance: f32,
}

#[derive(Clone, Component, Debug)]
pub struct Neighbours {
    pub range: f32,
    pub neighbours: Vec<Neighbour>,
}

impl Default for Neighbours {
    fn default() -> Self {
        Self {
            range: 10.,
            neighbours: Default::default(),
        }
    }
}

pub fn find_neighbours(
    pool: Res<ComputeTaskPool>,
    space: Res<FastUnitIndex>,
    mut query: Query<(Entity, &Transform, &mut Neighbours)>,
) {
    // let start = std::time::Instant::now();
    query.par_for_each_mut(&pool, 32, |(src_entity, transform, mut neighbours)| {
        let ns = space.grid.within_unsorted(
            &[transform.translation.x, transform.translation.z],
            neighbours.range * neighbours.range,
            &squared_euclidean,
        );
        neighbours.neighbours.clear();
        for (distance, entity) in ns.ok().unwrap_or_default() {
            if *entity != src_entity {
                neighbours.neighbours.push(Neighbour {
                    entity: *entity,
                    distance: distance.sqrt(),
                })
            }
        }
        neighbours
            .neighbours
            .sort_unstable_by(|n1, n2| n1.distance.partial_cmp(&n2.distance).unwrap());
    });
    // let dt = (std::time::Instant::now() - start).as_micros();
    // info!("Neighbours update time: {dt}μs");
}
