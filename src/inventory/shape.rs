use bevy::prelude::*;

#[derive(Reflect, Default, Debug, Clone)]
pub struct Shape {
    pub offset: IVec2,
    pub orientation: Orientation,
    pub layout: Layout,
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Shape {{ offset: x:{}, y:{}, orientation: {:?}, layout: {} }}", self.offset.x, self.offset.y, self.orientation, self.layout)
    }
}

impl Shape {
    pub fn size(&self) -> UVec2 {
        self.layout.size()
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = IVec2> {
        OffsetIter {
            iter: ShapeIter {
                shape: self.layout.iter_cells(),
                orentation: self.orientation,
            },
            offset: self.offset,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default, strum_macros::EnumIter)]
pub enum Orientation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Orientation {
    pub fn apply(&self, cell: IVec2) -> IVec2 {
        match self {
            Orientation::Deg0 => cell,
            Orientation::Deg90 => IVec2::new(cell.y, -cell.x),
            Orientation::Deg180 => IVec2::new(-cell.x, -cell.y),
            Orientation::Deg270 => IVec2::new(-cell.y, cell.x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum Layout {
    Rect { size: UVec2 },
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Rect { size: UVec2::ONE }
    }
}

impl Layout {
    pub fn size(&self) -> UVec2 {
        match self {
            Layout::Rect { size } => *size,
        }
    }

    pub fn iter_cells(&self) -> LayoutIter {
        LayoutIter {
            layout: self.clone(),
            current: IVec2::ZERO,
        }
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Rect { size } => write!(f, "Rect {{ Width :{}, Height :{} }}", size.x, size.y),
        }
    }
}

pub struct LayoutIter {
    layout: Layout,
    current: IVec2,
}

impl Iterator for LayoutIter {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        match self.layout {
            Layout::Rect { size } => {
                if self.current.y >= size.y as i32 {
                    return None;
                }
                let cell = self.current;
                self.current.x += 1;
                if self.current.x >= size.x as i32 {
                    self.current.x = 0;
                    self.current.y += 1;
                }
                Some(cell)
            }
        }
    }
}

pub struct ShapeIter {
    shape: LayoutIter,
    orentation: Orientation,
}

impl Iterator for ShapeIter {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        self.shape.next().map(|p| self.orentation.apply(p))
    }
}


pub struct OffsetIter<I: Iterator<Item = IVec2>> {
    iter: I,
    offset: IVec2,
}

impl<I: Iterator<Item = IVec2>> Iterator for OffsetIter<I> {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|cell| cell + self.offset)
    }
}