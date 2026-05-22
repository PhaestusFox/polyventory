use std::{borrow::Cow, convert::Infallible, str::FromStr};

use bevy::{
    ecs::{lifecycle::HookContext, relationship::Relationship, world::DeferredWorld},
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
};

use crate::inventory::slot::shape_iters::OrientationIter;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SlotType {
    Untyped,
    WaterTight,
    Small,
    Custom(Cow<'static, str>),
}

impl FromStr for SlotType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Untyped" => Ok(SlotType::Untyped),
            "WaterTight" => Ok(SlotType::WaterTight),
            "Small" => Ok(SlotType::Small),
            custom => Ok(SlotType::Custom(Cow::Owned(custom.to_string()))),
        }
    }
}

#[derive(Clone, Deref)]
pub struct Entry {
    pub entity: Entity, // todo: Move this a layer up? have shape, position, rotation as a sub structure?
    #[deref]
    pub shape: Shape,
}

impl PartialEq<Entity> for Entry {
    fn eq(&self, other: &Entity) -> bool {
        self.entity.eq(other)
    }
}

pub struct Slot {
    pub slot_type: Vec<SlotType>,
    pub position: IVec2,
    pub size: UVec2,
    pub entries: Vec<Entry>,
}

impl super::traits::Searchable<Entity> for Slot {
    type Index = usize;

    fn contains(&self, filter: Entity) -> bool {
        self.entries.iter().any(|entry| entry.eq(&filter))
    }

    fn find(&self, filter: Entity) -> Option<Self::Index> {
        self.entries.iter().position(|entry| entry.eq(&filter))
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum Orientation {
    #[default]
    Identity,
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}

impl Orientation {
    pub fn offset(&self) -> Vec2 {
        match self {
            Orientation::Identity | Orientation::Rot0 => Vec2::ZERO,
            Orientation::Rot90 => Vec2::new(1., 0.),
            Orientation::Rot180 => Vec2::new(1., 1.),
            Orientation::Rot270 => Vec2::new(0., 1.),
        }
    }

    pub fn rev(self) -> Self {
        match self {
            Orientation::Identity | Orientation::Rot0 => Orientation::Identity,
            Orientation::Rot90 => Orientation::Rot270,
            Orientation::Rot180 => Orientation::Rot180,
            Orientation::Rot270 => Orientation::Rot90,
        }
    }

    pub fn rotate_ccw(self) -> Self {
        match self {
            Orientation::Identity | Orientation::Rot0 => Orientation::Rot90,
            Orientation::Rot90 => Orientation::Rot180,
            Orientation::Rot180 => Orientation::Rot270,
            Orientation::Rot270 => Orientation::Identity,
        }
    }

    pub fn rotate_cw(self) -> Self {
        match self {
            Orientation::Identity | Orientation::Rot0 => Orientation::Rot270,
            Orientation::Rot90 => Orientation::Identity,
            Orientation::Rot180 => Orientation::Rot90,
            Orientation::Rot270 => Orientation::Rot180,
        }
    }

    pub fn rotate_by_scroll(self, scroll: f32) -> Self {
        if scroll > 0.0 {
            self.rotate_cw()
        } else if scroll < 0.0 {
            self.rotate_ccw()
        } else {
            self
        }
    }
}

impl Into<Quat> for Orientation {
    fn into(self) -> Quat {
        match self {
            Orientation::Identity => Quat::IDENTITY,
            Orientation::Rot0 => Quat::IDENTITY,
            Orientation::Rot90 => Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            Orientation::Rot180 => Quat::from_rotation_z(std::f32::consts::PI),
            Orientation::Rot270 => Quat::from_rotation_z(3.0 * std::f32::consts::FRAC_PI_2),
        }
    }
}

impl Into<Rot2> for Orientation {
    fn into(self) -> Rot2 {
        match self {
            Orientation::Identity => Rot2::IDENTITY,
            Orientation::Rot0 => Rot2::IDENTITY,
            Orientation::Rot90 => Rot2::turn_fraction(0.25),
            Orientation::Rot180 => Rot2::turn_fraction(0.5),
            Orientation::Rot270 => Rot2::turn_fraction(0.75),
        }
    }
}

impl Slot {
    pub fn fit(&self, shape: &Shape) -> Option<Shape> {
        let mut shape = shape.clone();

        for orientation in OrientationIter::start_with(shape.orientation) {
            shape.orientation = orientation;
            for cell in self.iter_cells() {
                shape.position = cell;
                if self.fit_at(&shape).is_ok() {
                    return Some(shape);
                }
            }
        }
        None
    }

    pub fn fit_at(&self, entry: &Shape) -> Result<(), FitFailure> {
        let bounds = entry.bounds();
        let slot = self.bounds();
        if !slot.contains(&bounds) {
            return Err(FitFailure::NotInBounds(bounds, slot));
        }
        for (i, other) in self.entries.iter().enumerate() {
            let other_bounds = other.bounds();
            if !bounds.intersects(&other_bounds) {
                continue;
            }
            for cell in collision::intersection(&bounds, &other_bounds) {
                if entry.ocupies(cell) && other.ocupies(cell) {
                    return Err(FitFailure::OverlapsWith(i, cell));
                }
            }
        }
        Ok(())
    }

    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    pub fn bounds(&self) -> Aabb2d {
        Aabb2d {
            min: Vec2::ZERO,
            max: self.size.as_vec2(),
        }
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = IVec2> {
        shape_iters::RectIter::new(self.size)
    }

    pub fn remove(&mut self, index: usize) -> Option<Entry> {
        if index < self.entries.len() {
            Some(self.entries.swap_remove(index))
        } else {
            None
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FitFailure {
    #[error("Item {0:?} does not fit within slot {1:?}")]
    NotInBounds(Aabb2d, Aabb2d),
    #[error("Item overlaps with another entry: {0} @ cell {1:?}")]
    OverlapsWith(usize, IVec2),
}

#[derive(Debug, Clone, Reflect, Component)]
pub struct Shape {
    pub cells: Cells,
    pub position: IVec2,
    pub orientation: Orientation,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            cells: Cells::Rect {
                size: UVec2::new(1, 1),
            },
            position: IVec2::ZERO,
            orientation: Orientation::Identity,
        }
    }
}

impl Shape {
    pub fn bounds(&self) -> Aabb2d {
        let Cells::Rect { size } = self.cells;

        let px = self.position.x as i32;
        let py = self.position.y as i32;
        let w = size.x as i32;
        let h = size.y as i32;

        let (min, max) = match self.orientation {
            Orientation::Identity | Orientation::Rot0 => {
                (IVec2::new(px, py), IVec2::new(px + w, py + h))
            }
            Orientation::Rot90 => (IVec2::new(px - h + 1, py), IVec2::new(px + 1, py + w)),
            Orientation::Rot180 => (
                IVec2::new(px - w + 1, py - h + 1),
                IVec2::new(px + 1, py + 1),
            ),
            Orientation::Rot270 => (IVec2::new(px, py - w + 1), IVec2::new(px + h, py + 1)),
        };

        Aabb2d {
            min: min.as_vec2(),
            max: max.as_vec2(),
        }
    }

    pub fn transform(&self, cell_size: Vec2) -> Transform {
        Transform {
            translation: ((self.position.as_vec2() + self.orientation.offset()) * cell_size)
                .extend(1.0),
            rotation: self.orientation.into(),
            ..Default::default()
        }
    }

    pub fn ocupies(&self, cell: IVec2) -> bool {
        let relative = cell - self.position;
        let local = match self.orientation {
            Orientation::Identity | Orientation::Rot0 => relative,
            // local -> world for Rot90 is (-y, x), so invert with (y, -x)
            Orientation::Rot90 => IVec2::new(relative.y, -relative.x),
            Orientation::Rot180 => IVec2::new(-relative.x, -relative.y),
            // local -> world for Rot270 is (y, -x), so invert with (-y, x)
            Orientation::Rot270 => IVec2::new(-relative.y, relative.x),
        };

        self.cells.contains(local)
    }
}

mod test;

/// Represents the space take up
#[derive(Debug, Clone, Reflect)]
pub enum Cells {
    Rect { size: UVec2 },
}

impl Cells {
    pub fn bounds(&self) -> Aabb2d {
        match self {
            Cells::Rect { size } => Aabb2d {
                min: Vec2::ZERO,
                max: size.as_vec2(),
            },
        }
    }

    pub fn contains(&self, pos: IVec2) -> bool {
        if pos.x < 0 || pos.y < 0 {
            return false;
        }
        match self {
            Cells::Rect { size } => pos.x < size.x as i32 && pos.y < size.y as i32,
        }
    }
}

impl Entry {
    // #[inline(always)]
    // pub fn iter_cells(&self) -> Box<dyn Iterator<Item = UVec2>> {
    //     self.cells.iter_cells()
    // }

    #[inline(always)]
    pub fn bounds(&self) -> Aabb2d {
        self.shape.bounds()
    }
}

mod shape_iters;

mod collision;
