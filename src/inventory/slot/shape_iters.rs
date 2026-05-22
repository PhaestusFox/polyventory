use bevy::{ecs::entity::hash_set::Iter, math::UVec2};

pub struct RectIter {
    offset: IVec2,
    width: i32,
    height: i32,
    current: IVec2,
}

impl RectIter {
    pub fn new(size: UVec2) -> Self {
        Self {
            offset: IVec2::ZERO,
            width: size.x as i32,
            height: size.y as i32,
            current: IVec2::ZERO,
        }
    }

    pub fn with_offset(mut self, offset: IVec2) -> Self {
        self.offset = offset;
        self
    }
}

impl Iterator for RectIter {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y >= self.height {
            return None;
        }
        let pos = self.current + self.offset;
        self.current.x += 1;
        if self.current.x >= self.width {
            self.current.x = 0;
            self.current.y += 1;
        }
        Some(pos)
    }
}

pub struct BoundsIter {
    min: UVec2,
    max: UVec2,
    current: UVec2,
}

impl BoundsIter {
    pub fn new(min: UVec2, max: UVec2) -> Self {
        Self {
            min,
            max,
            current: min,
        }
    }
}

impl Iterator for BoundsIter {
    type Item = UVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y >= self.max.y {
            return None;
        }
        let pos = self.current;
        self.current.x += 1;
        if self.current.x >= self.max.x {
            self.current.x = self.min.x;
            self.current.y += 1;
        }
        Some(pos)
    }
}

use super::*;

pub struct OrientationIter {
    orientations: Vec<Orientation>,
}

impl Iterator for OrientationIter {
    type Item = Orientation;

    fn next(&mut self) -> Option<Self::Item> {
        self.orientations.pop()
    }
}

impl OrientationIter {
    pub fn all() -> Self {
        Self {
            orientations: vec![
                Orientation::Rot270,
                Orientation::Rot180,
                Orientation::Rot90,
                Orientation::Rot0,
            ],
        }
    }

    pub fn start_with(orientation: Orientation) -> Self {
        if orientation == Orientation::Identity {
            return Self::all();
        }
        let mut orientations = vec![
            Orientation::Rot270,
            Orientation::Rot180,
            Orientation::Rot90,
            Orientation::Rot0,
        ];
        orientations.rotate_left(orientation as usize - 1);
        Self { orientations }
    }
}
