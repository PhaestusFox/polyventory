use bevy::math::{IVec2, UVec2, bounding::Aabb2d};

use crate::inventory::slot::shape_iters::RectIter;

pub fn intersection(a: &Aabb2d, b: &Aabb2d) -> impl Iterator<Item = IVec2> {
    let min = a.min.max(b.min);
    let max = a.max.min(b.max);
    if min.x < max.x && min.y < max.y {
        RectIter::new((max - min).as_uvec2()).with_offset(min.as_ivec2())
    } else {
        RectIter::new(UVec2::ZERO)
    }
}
