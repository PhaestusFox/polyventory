use super::*;

#[test]
fn ocupied_1x1() {
    let shape = Shape {
        cells: Cells::Rect {
            size: UVec2::new(1, 1),
        },
        position: IVec2::new(1, 1),
        orientation: Orientation::Identity,
    };
    for orientation in OrientationIter::all() {
        let shape = Shape {
            orientation,
            ..shape.clone()
        };
        for x in 0..5 {
            for y in 0..5 {
                let cell = IVec2::new(x, y);
                if cell == IVec2::new(1, 1) {
                    assert!(shape.ocupies(cell), "Cell {:?} should be ocupied", cell);
                } else {
                    assert!(
                        !shape.ocupies(cell),
                        "Cell {:?} should not be ocupied",
                        cell
                    );
                }
            }
        }
    }
}

#[test]
fn ocupied_1x2() {
    let keys = [
        [(1, 1), (1, 2)],
        [(1, 1), (0, 1)],
        [(1, 1), (1, 0)],
        [(1, 1), (2, 1)],
    ];
    let shape = Shape {
        cells: Cells::Rect {
            size: UVec2::new(1, 2),
        },
        position: IVec2::new(1, 1),
        orientation: Orientation::Identity,
    };

    for (orientation, key) in OrientationIter::all().zip(keys) {
        let shape = Shape {
            orientation,
            ..shape.clone()
        };
        for x in 0..5 {
            for y in 0..5 {
                let cell = IVec2::new(x, y);
                if key.contains(&(x, y)) {
                    assert!(shape.ocupies(cell), "Cell {:?} should be ocupied", cell);
                } else {
                    assert!(
                        !shape.ocupies(cell),
                        "Cell {:?} should not be ocupied",
                        cell
                    );
                }
            }
        }
    }
}

#[test]
fn ocupied_2x2() {
    let keys = [
        [(1, 2), (2, 1), (2, 2)],
        [(1, 2), (0, 2), (0, 1)],
        [(0, 1), (0, 0), (1, 0)],
        [(1, 0), (2, 0), (2, 1)],
    ];
    let shape = Shape {
        cells: Cells::Rect {
            size: UVec2::new(2, 2),
        },
        position: IVec2::new(1, 1),
        orientation: Orientation::Identity,
    };
    for (orientation, key) in OrientationIter::all().zip(keys) {
        let shape = Shape {
            orientation,
            ..shape.clone()
        };
        for x in 0..5 {
            for y in 0..5 {
                let cell = IVec2::new(x, y);
                if key.contains(&(x, y)) || cell == IVec2::new(1, 1) {
                    assert!(shape.ocupies(cell), "Cell {:?} should be ocupied", cell);
                } else {
                    assert!(
                        !shape.ocupies(cell),
                        "Cell {:?} should not be ocupied",
                        cell
                    );
                }
            }
        }
    }
}

#[test]
fn ocupied_1x4() {
    let keys = [
        [(5, 5), (5, 6), (5, 7), (5, 8)],
        [(5, 5), (4, 5), (3, 5), (2, 5)],
        [(5, 5), (5, 4), (5, 3), (5, 2)],
        [(5, 5), (6, 5), (7, 5), (8, 5)],
    ];
    let shape = Shape {
        cells: Cells::Rect {
            size: UVec2::new(1, 4),
        },
        position: IVec2::new(5, 5),
        orientation: Orientation::Identity,
    };
    for (orientation, key) in OrientationIter::all().zip(keys) {
        let shape = Shape {
            orientation,
            ..shape.clone()
        };
        for x in 0..10 {
            for y in 0..10 {
                let cell = IVec2::new(x, y);
                if key.contains(&(x, y)) {
                    assert!(shape.ocupies(cell), "Cell {:?} should be ocupied", cell);
                } else {
                    assert!(
                        !shape.ocupies(cell),
                        "Cell {:?} should not be ocupied",
                        cell
                    );
                }
            }
        }
    }
}

#[test]
fn bounds_1x4() {
    let keys = [
        (IVec2::new(5, 5), IVec2::new(6, 9)),
        (IVec2::new(2, 5), IVec2::new(6, 6)),
        (IVec2::new(5, 2), IVec2::new(6, 6)),
        (IVec2::new(5, 5), IVec2::new(9, 6)),
    ];

    let shape = Shape {
        cells: Cells::Rect {
            size: UVec2::new(1, 4),
        },
        position: IVec2::new(5, 5),
        orientation: Orientation::Identity,
    };

    for (orientation, (min, max)) in OrientationIter::all().zip(keys) {
        let shape = Shape {
            orientation,
            ..shape.clone()
        };
        let bounds = shape.bounds();
        assert_eq!(bounds.min.as_ivec2(), min);
        assert_eq!(bounds.max.as_ivec2(), max);
    }
}
