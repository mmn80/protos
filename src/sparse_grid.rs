use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct GridPos {
    pub x: u32,
    pub y: u32,
}

impl GridPos {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug)]
pub struct SparseGrid<V: 'static> {
    pub width: u32,
    pub values: Vec<Option<V>>,
}

impl<V: 'static> SparseGrid<V> {
    pub fn new(width: u32, fill: Option<V>) -> Self
    where
        V: Clone,
    {
        let sz = width as usize;
        let mut values = Vec::new();
        values.resize(sz * sz, fill);
        Self { width, values }
    }

    pub fn grid_pos(&self, pos: Vec3) -> GridPos {
        GridPos::new(
            f32::max(0., pos.x + self.width as f32 / 2.).ceil() as u32,
            f32::max(0., pos.z + self.width as f32 / 2.).ceil() as u32,
        )
    }

    fn raw_pos(&self, pos: GridPos) -> usize {
        (pos.x as usize) * self.width as usize + (pos.y as usize)
    }

    #[inline]
    pub fn insert(&mut self, index: GridPos, value: V) {
        let index = self.raw_pos(index);
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    #[inline]
    pub fn contains(&self, index: GridPos) -> bool {
        let index = self.raw_pos(index);
        self.values.get(index).map(|v| v.is_some()).unwrap_or(false)
    }

    #[inline]
    pub fn get(&self, index: GridPos) -> Option<&V> {
        let index = self.raw_pos(index);
        self.values.get(index).map(|v| v.as_ref()).unwrap_or(None)
    }

    #[inline]
    pub fn get_mut(&mut self, index: GridPos) -> Option<&mut V> {
        let index = self.raw_pos(index);
        self.values
            .get_mut(index)
            .map(|v| v.as_mut())
            .unwrap_or(None)
    }

    #[inline]
    pub fn remove(&mut self, index: GridPos) -> Option<V> {
        let index = self.raw_pos(index);
        self.values.get_mut(index).and_then(|value| value.take())
    }

    #[inline]
    pub fn get_or_insert_with(&mut self, index: GridPos, func: impl FnOnce() -> V) -> &mut V {
        let index = self.raw_pos(index);
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
