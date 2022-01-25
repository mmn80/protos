use bevy::prelude::*;

pub struct SlowUnitPlugin;

impl Plugin for SlowUnitPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Ground::default())
            .add_startup_system(setup);
    }
}

pub const MAP_SIZE: f32 = 1000.;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ground: ResMut<Ground>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sz = MAP_SIZE / 2.;
    ground.entity = Some(
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box {
                    min_x: -sz,
                    max_x: sz,
                    min_y: -5.,
                    max_y: 0.,
                    min_z: -sz,
                    max_z: sz,
                })),
                material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                ..Default::default()
            })
            .insert(Name::new("Ground"))
            .id(),
    );
}

#[derive(Debug, Clone, Default)]
pub struct Ground {
    pub entity: Option<Entity>,
}

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
