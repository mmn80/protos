use std::ops::{Add, Sub};

use bevy::prelude::*;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: GridPos) -> f32 {
        (((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f32).sqrt()
    }

    pub const VN_OFFSETS: [GridPos; 4] = [
        GridPos::new(-1, 0),
        GridPos::new(1, 0),
        GridPos::new(0, -1),
        GridPos::new(0, 1),
    ];
}

impl Add for GridPos {
    type Output = Self;

    fn add(self, rhs: GridPos) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for GridPos {
    type Output = Self;

    fn sub(self, rhs: GridPos) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl From<Vec3> for GridPos {
    fn from(pos: Vec3) -> Self {
        Self {
            x: pos.x as i32,
            y: pos.z as i32,
        }
    }
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
        let po2_width = width.next_power_of_two();
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

    pub fn reset(&mut self, new_width: u32, new_height: u32, fill: Option<V>)
    where
        V: Clone,
    {
        self.values.clear();
        self.width = new_width;
        self.height = new_height;
        self.po2_width = new_width.next_power_of_two();
        self.values
            .resize((self.po2_width as usize) * (new_height as usize), fill);
    }
}
