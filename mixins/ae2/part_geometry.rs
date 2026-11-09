// SPDX-License-Identifier: LGPL-3.0-or-later
// BusCollisionHelper and AbstractReportingPart.getBoxes. API behavior retains
// the notice in LICENSES/AE2-API-MIT. Coordinates leave model pixels here.
use super::ae2_cable::{self, CableType};
use super::ae2_grid::Direction;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds(pub [f64; 6]);

// Java Math.min/max preserve signed zero and propagate NaN, unlike f64::min/max.
fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if a == 0.0 && b == 0.0 && b.is_sign_negative() {
        b
    } else if a <= b {
        a
    } else {
        b
    }
}
fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        a
    } else if a == 0.0 && b == 0.0 && a.is_sign_negative() {
        b
    } else if a >= b {
        a
    } else {
        b
    }
}
impl Bounds {
    pub fn new(v: [f64; 6]) -> Self {
        Self(std::array::from_fn(|i| {
            if i < 3 {
                min(v[i], v[i + 3])
            } else {
                max(v[i - 3], v[i])
            }
        }))
    }
    pub fn inflate(self, amount: f64) -> Self {
        Self::new(std::array::from_fn(|i| {
            self.0[i] + if i < 3 { -amount } else { amount }
        }))
    }
    pub fn contains(self, point: [f64; 3]) -> bool {
        (0..3).all(|i| self.0[i] <= point[i] && point[i] < self.0[i + 3])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionHelper {
    axes: [Direction; 3],
    visual: bool,
}
impl CollisionHelper {
    pub fn new(side: Option<Direction>, visual: bool) -> Self {
        use Direction::*;
        Self::with_axes(
            match side {
                None | Some(South) => [East, Up, South],
                Some(Down) => [East, North, Down],
                Some(Up) => [East, South, Up],
                Some(East) => [South, Up, East],
                Some(West) => [North, Up, West],
                Some(North) => [West, Up, North],
            },
            visual,
        )
    }
    pub fn with_axes(axes: [Direction; 3], visual: bool) -> Self {
        Self { axes, visual }
    }
    pub fn axes(self) -> [Direction; 3] {
        self.axes
    }
    pub fn is_bb_collision(self) -> bool {
        !self.visual
    }
    pub fn add_box(self, pixels: [f64; 6]) -> Bounds {
        let v = pixels.map(|n| n / 16.0);
        let [x, y, z] = self.axes.map(Direction::offset);
        let a: [f64; 3] = std::array::from_fn(|i| {
            v[0] * f64::from(x[i]) + v[1] * f64::from(y[i]) + v[2] * f64::from(z[i])
        });
        let b: [f64; 3] = std::array::from_fn(|i| {
            v[3] * f64::from(x[i]) + v[4] * f64::from(y[i]) + v[5] * f64::from(z[i])
        });
        Bounds::new(std::array::from_fn(|i| {
            let axis = i % 3;
            let value = if i < 3 { a[axis] } else { b[axis] };
            if x[axis] + y[axis] + z[axis] < 0 {
                value + 1.0
            } else {
                value
            }
        }))
    }
    pub fn reporting_boxes(self) -> [Bounds; 2] {
        [
            self.add_box([2., 2., 14., 14., 14., 16.]),
            self.add_box([4., 4., 13., 12., 12., 14.]),
        ]
    }
}

/// Source CableBus selection tries Down ... East, then the center. Facades are
/// a separate second pass and are deliberately not represented by this API.
pub fn select_primary(
    boxes: &[(Option<Direction>, Bounds)],
    point: [f64; 3],
) -> Option<Option<Direction>> {
    Direction::ALL
        .map(Some)
        .into_iter()
        .chain([None])
        .find(|side| {
            boxes
                .iter()
                .any(|(slot, bounds)| slot == side && bounds.inflate(0.002).contains(point))
        })
}

/// Shape collection for reporting parts (AEBasePart connection length 3).
/// Other part implementations and facades require their own source profiles.
pub fn reporting_host_boxes(
    cable: CableType,
    parts: u8,
    connections: u8,
    dense_neighbours: u8,
    visual: bool,
) -> Vec<(Option<Direction>, Bounds)> {
    let mut output = Vec::new();
    for side in Direction::ALL {
        if parts & (1 << side as u8) != 0 {
            output.extend(
                CollisionHelper::new(Some(side), visual)
                    .reporting_boxes()
                    .map(|b| (Some(side), b)),
            );
        }
    }
    let helper = CollisionHelper::new(None, visual);
    let lengths = std::array::from_fn(|i| (parts & (1 << i) != 0).then_some(3.0));
    output.extend(
        ae2_cable::boxes(
            cable,
            connections,
            dense_neighbours,
            helper.is_bb_collision(),
            127,
            lengths,
        )
        .into_iter()
        .map(|b| (None, helper.add_box(b.0))),
    );
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_bounds_preserve_nan_and_signed_zero() {
        for (a, b) in [(0.0, -0.0), (-0.0, 0.0)] {
            assert_eq!(min(a, b).to_bits(), (-0.0f64).to_bits());
            assert_eq!(max(a, b).to_bits(), 0.0f64.to_bits());
        }
        assert!(min(1.0, f64::NAN).is_nan());
        assert!(max(f64::NAN, 1.0).is_nan());
    }
    #[test]
    fn reporting_faces_have_source_depth_and_do_not_rotate_with_spin() {
        for side in Direction::ALL {
            let helper = CollisionHelper::new(Some(side), false);
            let [screen, back] = helper.reporting_boxes();
            let axis = side.offset().iter().position(|n| *n != 0).unwrap();
            let positive = side.offset()[axis] > 0;
            assert_eq!(screen.0[axis], if positive { 14. / 16. } else { 0. });
            assert_eq!(screen.0[axis + 3], if positive { 1. } else { 2. / 16. });
            assert_eq!(back.0[axis], if positive { 13. / 16. } else { 2. / 16. });
            assert!(helper.is_bb_collision());
        }
    }
    #[test]
    fn dense_outline_is_wider_than_physical_collision() {
        for kind in [CableType::DenseCovered, CableType::DenseSmart] {
            let outline = reporting_host_boxes(kind, 0, 0, 0, true);
            let collision = reporting_host_boxes(kind, 0, 0, 0, false);
            assert_eq!(
                outline[0].1.0,
                [
                    3. / 16.,
                    3. / 16.,
                    3. / 16.,
                    13. / 16.,
                    13. / 16.,
                    13. / 16.
                ]
            );
            assert_eq!(
                collision[0].1.0,
                [
                    4.9 / 16.,
                    4.9 / 16.,
                    4.9 / 16.,
                    11.1 / 16.,
                    11.1 / 16.,
                    11.1 / 16.
                ]
            );
        }
    }
    #[test]
    fn selection_uses_half_open_inflated_boxes_and_side_before_center() {
        let bounds = Bounds::new([0., 0., 0., 1., 1., 1.]);
        let slots = [
            (None, bounds),
            (Some(Direction::East), bounds),
            (Some(Direction::Down), bounds),
        ];
        assert_eq!(
            select_primary(&slots, [-0.002, 0.5, 0.5]),
            Some(Some(Direction::Down))
        );
        assert_eq!(select_primary(&slots, [1.002, 0.5, 0.5]), None);
        assert_eq!(select_primary(&slots, [f64::NAN, 0.5, 0.5]), None);
        assert_eq!(select_primary(&slots[..1], [0.5; 3]), Some(None));
    }
}
