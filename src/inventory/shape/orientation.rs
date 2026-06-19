use super::*;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Orientation: u8 {
        const DEG0 = 0b00;
        const DEG90 = 0b01;
        const DEG180 = 0b10;
        const DEG270 = Self::DEG90.bits() | Self::DEG180.bits();
    }
}

impl core::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Orientation");
        s.field(
            "Rotation",
            match self.bits() & 0b11 {
                0b01 => &"90",
                0b10 => &"180",
                0b11 => &"270",
                _ => &"0",
            },
        );
        s.finish()
    }
}

pub enum Operations {
    RotateClockWise,
    RotateCounterClockWise,
}

impl Orientation {
    pub fn apply(&self, cell: IVec2) -> IVec2 {
        if !(self.bits() & 0b11).is_multiple_of(2) {
            cell.yx()
        } else {
            cell
        }
    }

    pub fn operation(self, opp: Operations) -> Self {
        match opp {
            Operations::RotateClockWise => self.cw(),
            Operations::RotateCounterClockWise => self.ccw(),
        }
    }

    fn cw(mut self) -> Self {
        let s = self.bits().wrapping_add(1) & 0b11;
        self.remove(Orientation::DEG270);
        self.insert(Orientation::from_bits_retain(s));
        self
    }

    fn ccw(mut self) -> Self {
        let s = self.bits().wrapping_sub(1) & 0b11;
        self.remove(Orientation::DEG270);
        self.insert(Orientation::from_bits_retain(s));
        self
    }

    pub const fn degrees(self) -> f32 {
        self.intersection(Orientation::DEG270).bits() as f32 * 90.
    }

    pub fn iter_orientations() -> impl Iterator<Item = Orientation> {
        OrientationIter(0)
    }
}

pub struct OrientationIter(u8);

impl Iterator for OrientationIter {
    type Item = Orientation;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 > Orientation::all().bits() {
            return None;
        }
        let o = Orientation::from_bits_truncate(self.0);
        self.0 += 1;
        Some(o)
    }
}
