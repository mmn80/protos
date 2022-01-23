use bevy::math::Vec3;

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
