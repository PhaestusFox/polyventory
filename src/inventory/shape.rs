use bevy::prelude::*;

#[derive(Reflect)]
pub struct Shape {
    offset: IVec2,
    orientation: Orientation,
    layout: Layout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Orientation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum Layout {
    Rect { size: UVec2 },
}

impl From<super::slot::Shape> for Shape {
    fn from(value: super::slot::Shape) -> Self {
        let layout = match value.cells {
            super::slot::Cells::Rect { size } => Layout::Rect { size },
        };
        let offset = value.position;
        let orientation = match value.orientation {
            super::slot::Orientation::Rot0 | super::slot::Orientation::Identity => Orientation::Deg0,
            super::slot::Orientation::Rot90 => Orientation::Deg90,
            super::slot::Orientation::Rot180 => Orientation::Deg180,
            super::slot::Orientation::Rot270 => Orientation::Deg270,
        };
        Self { offset, orientation, layout }
    }
}