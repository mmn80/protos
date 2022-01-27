use bevy::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct GridPos {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug)]
pub struct SparseGrid<V: 'static> {
    width: u32,
    po2_width: u32,
    height: u32,
    values: Vec<Option<V>>,
}

impl<V: 'static> SparseGrid<V> {
    pub fn new(width: u32, height: u32, fill: Option<V>) -> Self
    where
        V: Clone,
    {
        let po2_width = Self::po2_width(width);
        let mut values = Vec::new();
        values.resize((po2_width as usize) * (height as usize), fill);
        Self {
            width,
            po2_width,
            height,
            values,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn po2_width(width: u32) -> u32 {
        if (width & (width - 1)) == 0 {
            width
        } else {
            let zeros = width.leading_zeros() as u32;
            assert!(zeros > 0, "width too large");
            1 << (32 - zeros)
        }
    }

    #[inline]
    pub fn grid_pos(&self, pos: Vec3) -> GridPos {
        let x = f32::max(0., pos.x + self.width as f32 / 2.).floor() as u32;
        let y = f32::max(0., pos.z + self.width as f32 / 2.).floor() as u32;
        GridPos { x, y }
    }

    fn grid_idx(&self, pos: GridPos) -> usize {
        (pos.y as usize) * self.po2_width as usize + (pos.x as usize)
    }

    #[inline]
    pub fn insert(&mut self, pos: GridPos, value: V) {
        let index = self.grid_idx(pos);
        if index < self.values.len() {
            self.values[index] = Some(value);
        } else {
            warn!("out of bounds grid index {:?}, ignoring", pos);
        }
    }

    #[inline]
    pub fn contains(&self, pos: GridPos) -> bool {
        let index = self.grid_idx(pos);
        self.values.get(index).map(|v| v.is_some()).unwrap_or(false)
    }

    #[inline]
    pub fn get(&self, pos: GridPos) -> Option<&V> {
        let index = self.grid_idx(pos);
        self.values.get(index).map(|v| v.as_ref()).unwrap_or(None)
    }

    #[inline]
    pub fn get_mut(&mut self, pos: GridPos) -> Option<&mut V> {
        let index = self.grid_idx(pos);
        self.values
            .get_mut(index)
            .map(|v| v.as_mut())
            .unwrap_or(None)
    }

    #[inline]
    pub fn remove(&mut self, pos: GridPos) -> Option<V> {
        let index = self.grid_idx(pos);
        self.values.get_mut(index).and_then(|value| value.take())
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32, fill: Option<V>)
    where
        V: Clone,
    {
        self.width = new_width;
        self.height = new_height;
        self.po2_width = Self::po2_width(new_width);
        self.values
            .resize((self.po2_width as usize) * (new_height as usize), fill);
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}
