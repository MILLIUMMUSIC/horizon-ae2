// SPDX-License-Identifier: LGPL-3.0-or-later
// Original appeng.block.orientation.SpinMapping and FacingWithSpinStrategy
// placement rules for fully orientable AE2 blocks (Charger, Drive). The native
// blockstate tables expose the source `facing` and `spin` properties; this
// module only decides which orientation a placement context produces.

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
}
