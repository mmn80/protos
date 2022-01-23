use bevy::math::Vec3;

#[derive(Clone, Debug)]
pub struct QuadKey(u32);

impl QuadKey {
    pub fn new(x: u16, y: u16) -> Self {
        Self(Self::make_quad_key(x, y))
    }

    pub fn from_vec(pos: Vec3) -> Self {
        Self::new(pos.x as u16, pos.y as u16)
    }

    fn make_quad_key(x: u16, y: u16) -> u32 {
        let mut key = 0;
        for shift in 15..0 {
            let x_b = ((x >> shift) & 1) == 1;
            let y_b = ((y >> shift) & 1) == 1;
            key = key << 1;
            if x_b {
                key += 1;
            }
            key = key << 1;
            if y_b {
                key += 1;
            }
        }
        key
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
    pub fn insert(&mut self, index: QuadKey, value: V) {
        let index = index.0 as usize;
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    #[inline]
    pub fn contains(&self, index: QuadKey) -> bool {
        let index = index.0 as usize;
        self.values.get(index).map(|v| v.is_some()).unwrap_or(false)
    }

    #[inline]
    pub fn get(&self, index: QuadKey) -> Option<&V> {
        let index = index.0 as usize;
        self.values.get(index).map(|v| v.as_ref()).unwrap_or(None)
    }

    #[inline]
    pub fn get_mut(&mut self, index: QuadKey) -> Option<&mut V> {
        let index = index.0 as usize;
        self.values
            .get_mut(index)
            .map(|v| v.as_mut())
            .unwrap_or(None)
    }

    #[inline]
    pub fn remove(&mut self, index: QuadKey) -> Option<V> {
        let index = index.0 as usize;
        self.values.get_mut(index).and_then(|value| value.take())
    }

    #[inline]
    pub fn get_or_insert_with(&mut self, index: QuadKey, func: impl FnOnce() -> V) -> &mut V {
        let index = index.0 as usize;
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
