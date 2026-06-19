use bevy::prelude::*;

mod collision;
pub use collision::AabbBox;

mod orientation;
pub use orientation::{Operations, Orientation};

#[derive(Reflect, Default, Debug, Clone)]
pub struct Shape {
    pub offset: IVec2,
    #[reflect(ignore)]
    pub orientation: Orientation,
    pub layout: Layout,
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Shape {{ offset: (x:{}, y:{}),\norientation: {:?},\nlayout: {} }}",
            self.offset.x, self.offset.y, self.orientation, self.layout
        )
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
        Rot2::degrees(self.orientation.degrees())
    }

    pub fn ui_transform(&self) -> UiTransform {
        let og = self.layout.size().as_vec2();
        let size = self.bounds().size().as_vec2();
        // 90 degree
        // u = x - y / y * 0.5
        // v = -(x + y - 2) / x * 0.5

        let scale = og / size;

        // let t = match self.orientation {
        //     Orientation::DEG0 => Val2::ZERO,
        //     Orientation::DEG90 => Val2::new(
        //         Val::Percent((size.x - size.y) / size.y * 50.0),
        //         Val::Percent(-(size.x + size.y - 2.) / size.x * 50.),
        //     ),
        //     Orientation::DEG180 => Val2::new(
        //         Val::Percent((-size.x + 1.) / size.x * 100.),
        //         Val::Percent((-size.y + 1.) / size.y * 100.),
        //     ),
        //     Orientation::DEG270 =>Val2::new(
        //         Val::Percent(-(size.x + size.y - 2.) / size.y * 50.),
        //         Val::Percent((size.y - size.x) / size.x * 50.),
        //     ),
        // };
        UiTransform {
            translation: Val2::default(),
            rotation: self.rotation(),
            scale: scale,
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
            Layout::Rect { size } => {
                write!(f, "Rect {{ Width :{},\n Height :{} }}", size.x, size.y)
            }
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
