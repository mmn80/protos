use bevy::{prelude::*, tasks::ComputeTaskPool};
use big_brain::thinker::HasThinker;

#[derive(Clone, Debug)]
pub struct GridPos {
    pub x: u16,
    pub y: u16,
}

impl GridPos {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn from_vec(pos: Vec3) -> Self {
        Self::new(
            f32::max(0., pos.x + 500.).ceil() as u16,
            f32::max(0., pos.z + 500.).ceil() as u16,
        )
    }

    pub fn as_raw(&self) -> usize {
        (self.x as usize) * 1024 + (self.y as usize)
        // let mut key = 0;
        // let mut shift = 15;
        // while shift >= 0 {
        //     key = key << 1;
        //     if ((self.x >> shift) & 1) == 1 {
        //         key += 1;
        //     }
        //     key = key << 1;
        //     if ((self.y >> shift) & 1) == 1 {
        //         key += 1;
        //     }
        //     shift -= 1;
        // }
        // key
    }
}

#[derive(Debug)]
pub struct Grid<V: 'static> {
    pub values: Vec<Option<V>>,
}

impl<V: 'static> Grid<V> {
    #[inline]
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn insert(&mut self, index: GridPos, value: V) {
        let index = index.as_raw();
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    #[inline]
    pub fn contains(&self, index: GridPos) -> bool {
        let index = index.as_raw();
        self.values.get(index).map(|v| v.is_some()).unwrap_or(false)
    }

    #[inline]
    pub fn get(&self, index: GridPos) -> Option<&V> {
        let index = index.as_raw();
        self.values.get(index).map(|v| v.as_ref()).unwrap_or(None)
    }

    #[inline]
    pub fn get_mut(&mut self, index: GridPos) -> Option<&mut V> {
        let index = index.as_raw();
        self.values
            .get_mut(index)
            .map(|v| v.as_mut())
            .unwrap_or(None)
    }

    #[inline]
    pub fn remove(&mut self, index: GridPos) -> Option<V> {
        let index = index.as_raw();
        self.values.get_mut(index).and_then(|value| value.take())
    }

    #[inline]
    pub fn get_or_insert_with(&mut self, index: GridPos, func: impl FnOnce() -> V) -> &mut V {
        let index = index.as_raw();
        if index < self.values.len() {
            return self.values[index].get_or_insert_with(func);
        }
        self.values.resize_with(index + 1, || None);
        let value = &mut self.values[index];
        *value = Some(func());
        value.as_mut().unwrap()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}

pub struct SpaceIndex {
    pub grid: kiddo::KdTree<f32, Entity, 2>, //Grid<Entity>,
}

impl SpaceIndex {
    pub fn new() -> Self {
        Self {
            grid: kiddo::KdTree::new(),
        }
    }
}

pub fn update_grid(
    mut res: ResMut<SpaceIndex>,
    query: Query<(Entity, &Transform), With<HasThinker>>,
) {
    //let start = std::time::Instant::now();
    res.grid = kiddo::KdTree::new();
    for (entity, transform) in query.iter() {
        res.grid
            .add(&[transform.translation.x, transform.translation.z], entity)
            .ok();
    }
    // let dt = (std::time::Instant::now() - start).as_micros();
    // info!("grid construction time: {}μs, len={}", dt, res.grid.size(),);
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
    space: Res<SpaceIndex>,
    mut query: Query<(Entity, &Transform, &mut Neighbours)>,
) {
    // let start = std::time::Instant::now();
    query.par_for_each_mut(&pool, 32, |(src_entity, transform, mut neighbours)| {
        let ns = space.grid.within_unsorted(
            &[transform.translation.x, transform.translation.z],
            neighbours.range * neighbours.range,
            &kiddo::distance::squared_euclidean,
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
    // info!("Neighbours update time: {}μs", dt);
}
