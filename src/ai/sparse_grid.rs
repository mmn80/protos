use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct GridPos(usize);

impl GridPos {
    fn new(width: u32, x: u32, y: u32) -> Self {
        Self((x as usize) * width as usize + (y as usize))
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
        assert!((width & (width - 1)) == 0, "grid with must be power of 2");
        let sz = width as usize;
        let mut values = Vec::new();
        values.resize(sz * sz, fill);
        Self { width, values }
    }

    pub fn grid_pos(&self, pos: Vec3) -> GridPos {
        let (x, y) = self.grid_coords(pos);
        GridPos::new(self.width, x, y)
    }

    #[inline]
    pub fn grid_coords(&self, pos: Vec3) -> (u32, u32) {
        let x = f32::max(0., pos.x + self.width as f32 / 2.).floor() as u32;
        let y = f32::max(0., pos.z + self.width as f32 / 2.).floor() as u32;
        (x, y)
    }

    #[inline]
    pub fn grid_pos_by_coords(&self, x: u32, y: u32) -> GridPos {
        GridPos::new(self.width, x, y)
    }

    #[inline]
    pub fn insert(&mut self, index: GridPos, value: V) {
        if index.0 >= self.values.len() {
            self.values.resize_with(index.0 + 1, || None);
        }
        self.values[index.0] = Some(value);
    }

    #[inline]
    pub fn contains(&self, index: GridPos) -> bool {
        self.values
            .get(index.0)
            .map(|v| v.is_some())
            .unwrap_or(false)
    }

    #[inline]
    pub fn get(&self, index: GridPos) -> Option<&V> {
        self.values.get(index.0).map(|v| v.as_ref()).unwrap_or(None)
    }

    #[inline]
    pub fn get_mut(&mut self, index: GridPos) -> Option<&mut V> {
        self.values
            .get_mut(index.0)
            .map(|v| v.as_mut())
            .unwrap_or(None)
    }

    #[inline]
    pub fn remove(&mut self, index: GridPos) -> Option<V> {
        self.values.get_mut(index.0).and_then(|value| value.take())
    }

    #[inline]
    pub fn get_or_insert_with(&mut self, index: GridPos, func: impl FnOnce() -> V) -> &mut V {
        if index.0 < self.values.len() {
            return self.values[index.0].get_or_insert_with(func);
        }
        self.values.resize_with(index.0 + 1, || None);
        let value = &mut self.values[index.0];
        *value = Some(func());
        value.as_mut().unwrap()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn iter_pos(&self) -> GridPosIterator {
        GridPosIterator {
            curr_x: 0,
            curr_y: 0,
            width: self.width,
        }
    }
}

pub struct GridPosIterator {
    curr_x: u32,
    curr_y: u32,
    width: u32,
}

impl Iterator for GridPosIterator {
    type Item = (GridPos, u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_y >= self.width {
            None
        } else {
            let pos = (
                GridPos::new(self.width, self.curr_x, self.curr_y),
                self.curr_x,
                self.curr_y,
            );
            self.curr_x += 1;
            if self.curr_x >= self.width {
                self.curr_x = 0;
                self.curr_y += 1;
            }
            Some(pos)
        }
    }
}
