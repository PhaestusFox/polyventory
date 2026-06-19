use super::*;
use bevy::prelude::*;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct AabbBox {
    pub min: IVec2,
    pub max: IVec2,
}

impl AabbBox {
    pub fn size(&self) -> UVec2 {
        (self.max - self.min).as_uvec2() + UVec2::ONE
    }
}

impl core::ops::Add<IVec2> for AabbBox {
    type Output = Self;
    fn add(self, rhs: IVec2) -> Self::Output {
        AabbBox {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl core::ops::AddAssign<IVec2> for AabbBox {
    fn add_assign(&mut self, rhs: IVec2) {
        self.min += rhs;
        self.max += rhs;
    }
}

impl core::ops::Mul<Orientation> for AabbBox {
    type Output = Self;
    fn mul(self, rhs: Orientation) -> Self::Output {
        let a = rhs.apply(self.min);
        let b = rhs.apply(self.max);
        AabbBox {
            min: IVec2::new(a.x.min(b.x), a.y.min(b.y)),
            max: IVec2::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }
}

impl core::ops::MulAssign<Orientation> for AabbBox {
    fn mul_assign(&mut self, rhs: Orientation) {
        let a = rhs.apply(self.min);
        let b = rhs.apply(self.max);
        self.min = IVec2::new(a.x.min(b.x), a.y.min(b.y));
        self.max = IVec2::new(a.x.max(b.x), a.y.max(b.y));
    }
}

impl core::cmp::PartialOrd for AabbBox {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        // Self fits within other
        if self == other {
            Some(Ordering::Equal)
        } else if self.min.x >= other.min.x
            && self.max.x <= other.max.x
            && self.min.y >= other.min.y
            && self.max.y <= other.max.y
        {
            Some(Ordering::Less)
        }
        // Other fits within self
        else if other.min.x >= self.min.x
            && other.max.x <= self.max.x
            && other.min.y >= self.min.y
            && other.max.y <= self.max.y
        {
            Some(Ordering::Greater)
        }
        // they don't overlap or don't fit
        else {
            None
        }
    }

    // self is completely outside other
    fn gt(&self, other: &Self) -> bool {
        self.min.y > other.max.y
            || self.max.y < other.min.y
            || self.min.x > other.max.x
            || self.max.x < other.min.x
    }
    // self is completely inside other
    fn lt(&self, other: &Self) -> bool {
        self.min.y >= other.min.y
            && self.max.y <= other.max.y
            && self.min.x >= other.min.x
            && self.max.x <= other.max.x
    }
    // self is within or colliding with other
    fn le(&self, other: &Self) -> bool {
        other < self
            || (
                // selfs bottom edge is within other height
                self.min.y >= other.min.y && self.min.y <= other.max.y
        &&
        // self's bottom edge is within or wider than other's width
        self.min.x <= other.max.x && self.max.x >= other.min.x
        ||
        // self's left edge is within other width
        self.min.x <= other.max.x && self.min.x >= other.min.x
        &&
        // self's left edge is within or taller than other's height
        self.min.y <= other.max.y && self.max.y >= other.min.y
        ||
        // self's top edge is within other height
        self.max.y >= other.min.y && self.max.y <= other.max.y
        &&
        // self's top edge is within or wider than other's width
        self.min.x <= other.max.x && self.max.x >= other.min.x
        ||
        // self's right edge is within other width
        self.max.x >= other.min.x && self.max.x <= other.max.x
        &&
        // self's right edge is within or taller than other's height
        self.min.y <= other.max.y && self.max.y >= other.min.y
            )
    }
    // self is colliding with other but not completely inside or outside
    fn ge(&self, other: &Self) -> bool {
        self.le(other) && !(self < other)
    }
}

// impl core::cmp::Ord for AabbBox {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.partial_cmp(other).unwrap()
//     }
// }

macro_rules! assert_fl {
    ($arg:expr $(,)?) => {
        assert!(!($arg))
    };
    ($arg:expr, $($arg2:tt)+) => {
        assert!(!($arg), $($arg2)+)
    };
}

#[test]
fn rotation_test() {
    let mut box1 = AabbBox {
        min: IVec2::new(-1, -1),
        max: IVec2::new(1, 1),
    };
    box1 *= Orientation::DEG90;
    assert_eq!(box1.min, IVec2::new(-1, -1));
    assert_eq!(box1.max, IVec2::new(1, 1));
    box1 += IVec2::new(1, 1);
    assert_eq!(box1.min, IVec2::new(0, 0));
    assert_eq!(box1.max, IVec2::new(2, 2));
    box1 *= Orientation::DEG90;
    assert_eq!(box1.min, IVec2::new(0, 0));
    assert_eq!(box1.max, IVec2::new(2, 2));
    box1 *= Orientation::DEG90;
    assert!(box1.min == IVec2::new(0, 0));
    assert!(box1.max == IVec2::new(2, 2));
    box1 *= Orientation::DEG90;
    assert!(box1.min == IVec2::new(0, 0));
    assert!(box1.max == IVec2::new(2, 2));

    let mut box2 = AabbBox {
        min: IVec2::new(1, 1),
        max: IVec2::new(1, 1),
    };
    box2 *= Orientation::DEG90;
    assert_eq!(box2.min, IVec2::new(1, 1));
    assert_eq!(box2.max, IVec2::new(1, 1));
    box2 *= Orientation::DEG90;
    assert_eq!(box2.min, IVec2::new(1, 1));
    assert_eq!(box2.max, IVec2::new(1, 1));
    box2 *= Orientation::DEG90;
    assert_eq!(box2.min, IVec2::new(1, 1));
    assert_eq!(box2.max, IVec2::new(1, 1));
    box2 *= Orientation::DEG90;
    assert_eq!(box2.min, IVec2::new(1, 1));
    assert_eq!(box2.max, IVec2::new(1, 1));

    let mut box3 = AabbBox {
        min: IVec2::new(0, 0),
        max: IVec2::new(3, 1),
    };
    box3 *= Orientation::DEG90;
    assert_eq!(box3.min, IVec2::new(0, 0));
    assert_eq!(box3.max, IVec2::new(1, 3));
    box3 *= Orientation::DEG90;
    assert_eq!(box3.min, IVec2::new(0, 0));
    assert_eq!(box3.max, IVec2::new(3, 1));
    box3 *= Orientation::DEG90;
    assert_eq!(box3.min, IVec2::new(0, 0));
    assert_eq!(box3.max, IVec2::new(1, 3));
    box3 *= Orientation::DEG90;
    assert_eq!(box3.min, IVec2::new(0, 0));
    assert_eq!(box3.max, IVec2::new(3, 1));
    box3 *= Orientation::DEG180;
    assert_eq!(box3.min, IVec2::new(0, 0));
    assert_eq!(box3.max, IVec2::new(3, 1));
    box3 *= Orientation::DEG270;
    assert_eq!(box3.min, IVec2::new(0, 0));
    assert_eq!(box3.max, IVec2::new(1, 3));
}

#[test]
fn collision_test() {
    let box0 = AabbBox {
        min: IVec2::new(-1, -1),
        max: IVec2::new(-1, -1),
    };
    let box1 = AabbBox {
        min: IVec2::new(0, 0),
        max: IVec2::new(2, 2),
    };
    let box2 = AabbBox {
        min: IVec2::new(1, 1),
        max: IVec2::new(3, 3),
    };
    let box3 = AabbBox {
        min: IVec2::new(1, 1),
        max: IVec2::new(1, 1),
    };
    let box4 = AabbBox {
        min: IVec2::new(1, 1),
        max: IVec2::new(1, 1),
    };
    let bottle = AabbBox {
        min: IVec2::new(0, 0),
        max: IVec2::new(0, 3),
    };
    let bag = AabbBox {
        min: IVec2::new(0, 0),
        max: IVec2::new(29, 49),
    };
    // box1 is not inside box2
    assert_fl!(box1 < box2);
    // box1 is not outside box2
    assert_fl!(box1 > box2);
    // box1 and box2 collide
    assert!(box1 <= box2);
    // box1 and box2 collide
    assert!(box1 >= box2);
    // box1 and box2 are not equal
    assert_ne!(box1, box2);
    // box3 is inside box1
    assert!(box3 < box1);
    // box3 is inside box2
    assert!(box3 < box2);
    // box1 is not inside box3
    assert_fl!(box1 < box3);
    // box2 is not inside box3
    assert_fl!(box2 < box3);
    // box3 is inside box4
    assert!(box3 < box4);
    // box3 is not outside box4
    assert_fl!(box3 > box4);
    // box3 is equal to box4
    assert_eq!(box3, box4);
    // box3 is within or colliding with box4
    assert!(
        box3 <= box4,
        "{:?} should not collide with {:?}",
        box3,
        box4
    );
    // box3 is not colliding with box4
    assert_fl!(box3 >= box4);
    // box3 is the exact same as box4
    assert!(box3 == box4);
    // bottle is inside bag
    assert!(bottle < bag);
    let fbottle = bottle * Orientation::DEG180;
    // bottle is not inside bag when rotated 180 degrees
    assert_fl!(fbottle < bag, "{:?} should not fit in {:?}", fbottle, bag);
    // bottle is not outside bag when rotated 180 degrees
    assert_fl!(fbottle > bag, "{:?} should not fit in {:?}", fbottle, bag);
    // bottle is colliding with bag when rotated 180 degrees
    assert!(fbottle >= bag, "{:?} should not fit in {:?}", fbottle, bag);
    assert!(fbottle <= bag, "{:?} should not fit in {:?}", fbottle, bag);

    assert!(box0 > box1);
    assert!(box0 > box2);
    assert!(box0 > box3);
    assert!(box0 > box4);
    assert_fl!(box0 < box1);
    assert_fl!(box0 < box2);
    assert_fl!(box0 < box3);
    assert_fl!(box0 < box4);
    assert_fl!(box0 <= box1);
    assert_fl!(box0 <= box2);
    assert_fl!(box0 <= box3);
    assert_fl!(box0 <= box4);
}

#[test]
// this was a legit situation that passed and should fail
fn water_then_backpack() {
    let water = AabbBox {
        min: IVec2::new(0, 0),
        max: IVec2::new(0, 3),
    } * Orientation::DEG90
        + IVec2::new(3, 9);
    let backpack = AabbBox {
        min: IVec2::new(0, 0),
        max: IVec2::new(12, 14),
    } + IVec2::new(2, 0);

    assert!(backpack <= water)
}
