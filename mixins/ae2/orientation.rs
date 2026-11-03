// SPDX-License-Identifier: LGPL-3.0-or-later
// Original appeng.block.orientation.SpinMapping, FacingWithSpinStrategy and
// BlockOrientation rotation rules. The native blockstate tables expose the
// source `facing` and `spin` properties; this module decides which orientation
// a placement context produces and how the Certus Quartz Wrench turns one.

/// Direction order used by Minecraft's `Direction` enum.
pub const DIRECTIONS: [&str; 6] = ["down", "up", "north", "south", "west", "east"];

/// `SpinMapping.SPIN_DIRECTIONS`: for each facing, the up direction of each
/// spin value in clockwise order.
const SPIN_DIRECTIONS: [[&str; 4]; 6] = [
    ["north", "west", "south", "east"],
    ["north", "east", "south", "west"],
    ["up", "west", "down", "east"],
    ["up", "east", "down", "west"],
    ["up", "south", "down", "north"],
    ["up", "north", "down", "south"],
];

pub fn ordinal(direction: &str) -> Option<usize> {
    DIRECTIONS.iter().position(|d| *d == direction)
}

pub fn opposite(direction: &str) -> &'static str {
    match direction {
        "down" => "up",
        "up" => "down",
        "north" => "south",
        "south" => "north",
        "west" => "east",
        _ => "west",
    }
}

/// `Direction.fromYRot` for a horizontal direction.
pub fn from_yaw(yaw: f32) -> &'static str {
    let yaw = yaw.rem_euclid(360.0);
    if yaw < 45.0 || yaw >= 315.0 {
        "south"
    } else if yaw < 135.0 {
        "west"
    } else if yaw < 225.0 {
        "north"
    } else {
        "east"
    }
}

/// `SpinMapping.getSpinFromUp`: a degenerate up direction falls back to 0.
pub fn spin_from_up(facing: &str, up: &str) -> u8 {
    let Some(index) = ordinal(facing) else {
        return 0;
    };
    SPIN_DIRECTIONS[index]
        .iter()
        .position(|d| *d == up)
        .map_or(0, |i| i as u8)
}

/// `SpinMapping.getUpFromSpin`.
pub fn up_from_spin(facing: &str, spin: u8) -> &'static str {
    let Some(index) = ordinal(facing) else {
        return "up";
    };
    SPIN_DIRECTIONS[index][usize::from(spin.min(3))]
}

/// `FacingWithSpinStrategy.getStateForPlacement`: the horizontal direction is
/// the source `forward`; looking beyond 65 degrees up or down makes the facing
/// vertical and takes the spin reference from the horizontal look.
pub fn placement(yaw: f32, pitch: f32) -> (&'static str, u8) {
    let horizontal = opposite(from_yaw(yaw));
    let (facing, up) = if pitch > 65.0 {
        ("up", opposite(horizontal))
    } else if pitch < -65.0 {
        ("down", opposite(horizontal))
    } else {
        (horizontal, "up")
    };
    (facing, spin_from_up(facing, up))
}

fn step(direction: &str) -> Option<[i32; 3]> {
    Some(match direction {
        "down" => [0, -1, 0],
        "up" => [0, 1, 0],
        "north" => [0, 0, -1],
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => return None,
    })
}

fn from_step(step: [i32; 3]) -> Option<&'static str> {
    Some(match step {
        [0, -1, 0] => "down",
        [0, 1, 0] => "up",
        [0, 0, -1] => "north",
        [0, 0, 1] => "south",
        [-1, 0, 0] => "west",
        [1, 0, 0] => "east",
        _ => return None,
    })
}

/// One clockwise quarter turn around the clicked face's axis, as
/// `Direction.getClockWise(Axis)`:
/// X: `(x, z, -y)`, Y: `(-z, y, x)`, Z: `(y, -x, z)`.
fn clockwise(step: [i32; 3], axis: usize) -> [i32; 3] {
    let [x, y, z] = step;
    match axis {
        0 => [x, z, -y],
        1 => [-z, y, x],
        _ => [y, -x, z],
    }
}

/// `BlockOrientation.rotateClockwiseAround(clickedFace)`: the clicked face's
/// axis direction selects clockwise or counter-clockwise.
pub fn rotated(direction: &str, clicked: &str) -> Option<&'static str> {
    let (axis, positive) = match clicked {
        "east" => (0, true),
        "west" => (0, false),
        "up" => (1, true),
        "down" => (1, false),
        "south" => (2, true),
        "north" => (2, false),
        _ => return None,
    };
    let mut step = step(direction)?;
    let turns = if positive { 1 } else { 3 };
    for _ in 0..turns {
        step = clockwise(step, axis);
    }
    from_step(step)
}

/// `BlockOrientation.rotateClockwiseAround` for a full orientation: rotate the
/// facing and its up reference, then derive the spin. A degenerate up
/// reference falls back to spin 0, as `BlockOrientation.get(facing, top)` does.
pub fn rotate_full(facing: &str, spin: u8, clicked: &str) -> Option<(&'static str, u8)> {
    ordinal(facing)?;
    let up = up_from_spin(facing, spin);
    let new_facing = rotated(facing, clicked)?;
    let new_up = rotated(up, clicked)?;
    Some((new_facing, spin_from_up(new_facing, new_up)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_look_keeps_the_horizontal_facing_and_up_reference() {
        assert_eq!(placement(0.0, 0.0), ("north", 0));
        assert_eq!(placement(90.0, 0.0), ("east", 0));
        assert_eq!(placement(180.0, 0.0), ("south", 0));
        assert_eq!(placement(270.0, 0.0), ("west", 0));
        assert_eq!(placement(-90.0, 0.0), ("west", 0));
    }

    #[test]
    fn steep_pitch_makes_the_facing_vertical_with_the_horizontal_reference() {
        assert_eq!(placement(0.0, 66.0), ("up", 2));
        assert_eq!(placement(90.0, 66.0), ("up", 3));
        assert_eq!(placement(180.0, 90.0), ("up", 0));
        assert_eq!(placement(270.0, 66.0), ("up", 1));
        assert_eq!(placement(0.0, -66.0), ("down", 2));
        assert_eq!(placement(90.0, -66.0), ("down", 1));
        assert_eq!(placement(180.0, -90.0), ("down", 0));
        assert_eq!(placement(270.0, -66.0), ("down", 3));
    }

    #[test]
    fn boundaries_are_exclusive_above_and_below_sixty_five() {
        assert_eq!(placement(0.0, 65.0), ("north", 0));
        assert_eq!(placement(0.0, -65.0), ("north", 0));
    }

    #[test]
    fn spin_round_trips_through_the_original_up_table() {
        for facing in DIRECTIONS {
            for spin in 0..4u8 {
                assert_eq!(spin_from_up(facing, up_from_spin(facing, spin)), spin);
            }
        }
    }

    #[test]
    fn degenerate_up_direction_falls_back_to_no_spin() {
        assert_eq!(spin_from_up("north", "north"), 0);
        assert_eq!(spin_from_up("unknown", "up"), 0);
    }

    #[test]
    fn clockwise_axis_rotations_match_minecraft_direction() {
        // Direction.getClockWise(Axis) from the pinned 1.21.1 client.
        assert_eq!(rotated("north", "up"), Some("east"));
        assert_eq!(rotated("east", "up"), Some("south"));
        assert_eq!(rotated("south", "up"), Some("west"));
        assert_eq!(rotated("west", "up"), Some("north"));
        assert_eq!(rotated("up", "up"), Some("up"));
        assert_eq!(rotated("down", "up"), Some("down"));
        assert_eq!(rotated("down", "east"), Some("south"));
        assert_eq!(rotated("up", "east"), Some("north"));
        assert_eq!(rotated("north", "east"), Some("down"));
        assert_eq!(rotated("south", "east"), Some("up"));
        assert_eq!(rotated("down", "south"), Some("west"));
        assert_eq!(rotated("up", "south"), Some("east"));
        assert_eq!(rotated("west", "south"), Some("up"));
        assert_eq!(rotated("east", "south"), Some("down"));
        // The negative axis directions are the inverse quarter turns.
        assert_eq!(rotated("north", "down"), Some("west"));
        assert_eq!(rotated("down", "down"), Some("down"));
        assert_eq!(rotated("down", "up"), Some("down"));
        assert_eq!(rotated("down", "west"), Some("north"));
        assert_eq!(rotated("north", "north"), Some("north"));
    }

    #[test]
    fn four_clicks_around_an_axis_return_the_orientation() {
        for direction in DIRECTIONS {
            for clicked in DIRECTIONS {
                let mut current = direction;
                for _ in 0..4 {
                    current = rotated(current, clicked).unwrap();
                }
                assert_eq!(current, direction, "{direction} around {clicked}");
            }
        }
        for clicked in DIRECTIONS {
            let mut orientation = ("north", 0);
            for _ in 0..4 {
                orientation = rotate_full(orientation.0, orientation.1, clicked).unwrap();
            }
            assert_eq!(
                orientation,
                ("north", 0),
                "full orientation around {clicked}"
            );
        }
    }

    #[test]
    fn full_orientation_rotation_derives_the_source_spin() {
        assert_eq!(rotate_full("north", 0, "up"), Some(("east", 0)));
        assert_eq!(rotate_full("north", 0, "east"), Some(("down", 0)));
        assert_eq!(rotate_full("north", 0, "south"), Some(("north", 3)));
        assert_eq!(rotate_full("north", 0, "west"), Some(("up", 2)));
        // Rotating around X turns a down-facing block south while rotating
        // its up reference onto the vertical axis.
        assert_eq!(rotate_full("down", 0, "east"), Some(("south", 2)));
        assert_eq!(rotate_full("down", 1, "east"), Some(("south", 3)));
    }

    #[test]
    fn unknown_directions_and_faces_are_refused() {
        assert_eq!(rotated("north", "sideways"), None);
        assert_eq!(rotated("sideways", "up"), None);
        assert_eq!(rotate_full("sideways", 0, "up"), None);
    }
}
