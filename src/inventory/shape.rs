use bevy::prelude::*;

mod collision;
pub use collision::AabbBox;

#[derive(Reflect, Default, Debug, Clone)]
pub struct Shape {
    pub offset: IVec2,
    pub orientation: Orientation,
    pub layout: Layout,
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Shape {{ offset: (x:{}, y:{}),\norientation: {:?},\nlayout: {} }}", self.offset.x, self.offset.y, self.orientation, self.layout)
    }
}

impl Shape {
    pub fn iter_cells(&self) -> impl Iterator<Item = IVec2> {
        OffsetIter {
            iter: ShapeIter {
                shape: self.layout.iter_cells(),
                orentation: self.orientation,
            },
            offset: self.offset,
        }
    }

    pub fn bounds(&self) -> AabbBox {
        let mut bounds = self.layout.bounds();
        bounds *= self.orientation;
        bounds += self.offset;
        bounds
    }

    pub fn can_fit(&self, other: &Shape) -> bool {
        other.bounds() < self.bounds()
    }

    pub fn rotation(&self) -> Rot2 {
        match self.orientation {
            Orientation::Deg0 => Rot2::IDENTITY,
            Orientation::Deg90 => Rot2::degrees(90.0),
            Orientation::Deg180 => Rot2::degrees(180.0),
            Orientation::Deg270 => Rot2::degrees(270.0),
        }
    }

    pub fn ui_transform(&self) -> UiTransform {
        let size = self.bounds().size().as_vec2();
        // 90 degree 
        // u = x - y / y * 0.5
        // v = -(x + y - 2) / x * 0.5

        let aspect = size.x as f32 / size.y as f32;

        let t = match self.orientation {
            Orientation::Deg0 | Orientation::Deg180 => Val2::ZERO,
            Orientation::Deg90 | Orientation::Deg270 => Val2::new(
                Val::Percent((size.x - size.y) / size.y * 50.0),
                Val::Percent(-(size.x + size.y - 2.) / size.x * 50.),
            ),
        };
        // let offset = 
        UiTransform {
            translation: t,
            rotation: self.rotation(),
            ..Default::default()
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

    pub fn bounds(&self) -> AabbBox {
        let size = self.size();
        AabbBox {
            min: IVec2::ZERO,
            max: IVec2::new(size.x as i32 - 1, size.y as i32 - 1),
        }
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Rect { size } => write!(f, "Rect {{ Width :{},\n Height :{} }}", size.x, size.y),
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