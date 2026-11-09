// SPDX-License-Identifier: LGPL-3.0-or-later
// AbstractReportingPart state and AEBasePart's client power/channel flags.
use super::ae2_grid::Direction;

pub const BOXES: [[f64; 6]; 2] = [[2., 2., 14., 14., 14., 16.], [4., 4., 13., 12., 12., 14.]];
pub fn idle_power(require_channel: bool) -> f64 {
    if require_channel { 0.5 } else { 1. / 16. }
}
pub fn floor(value: f64) -> i32 {
    let truncated = value as i32;
    if value < f64::from(truncated) {
        truncated.wrapping_sub(1)
    } else {
        truncated
    }
}
fn floor_float(value: f32) -> i32 {
    let truncated = value as i32;
    if value < truncated as f32 {
        truncated.wrapping_sub(1)
    } else {
        truncated
    }
}

/// NumericTag.getAsByte, including Float/DoubleTag's Mth.floor conversion.
#[derive(Clone, Copy, Debug)]
pub enum Number {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}
impl Number {
    pub fn byte(self) -> i8 {
        match self {
            Self::Byte(v) => v,
            Self::Short(v) => v as i8,
            Self::Int(v) => v as i8,
            Self::Long(v) => v as i8,
            Self::Float(v) => floor_float(v) as i8,
            Self::Double(v) => floor(v) as i8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub powered: bool,
    pub missing_channel: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    Off,
    On,
    HasChannel,
}
impl Flags {
    pub fn byte(self) -> u8 {
        u8::from(self.powered) | (u8::from(self.missing_channel) << 1)
    }
    pub fn read(&mut self, value: u8, send_power: bool, send_missing: bool) -> bool {
        let old = *self;
        self.powered = value & 1 != 0;
        self.missing_channel = value & 2 != 0;
        send_power && self.powered != old.powered
            || send_missing && self.missing_channel != old.missing_channel
    }
    pub fn active(self) -> bool {
        self.powered && !self.missing_channel
    }
    pub fn model(self) -> Model {
        if self.active() {
            Model::HasChannel
        } else if self.powered {
            Model::On
        } else {
            Model::Off
        }
    }
}

#[derive(Clone, Debug)]
pub struct Reporting {
    spin: i8,
    opacity: i32,
    pub flags: Flags,
}
impl Default for Reporting {
    fn default() -> Self {
        Self::new(0)
    }
}
impl Reporting {
    pub fn new(spin: i8) -> Self {
        Self {
            spin,
            opacity: -1,
            flags: Flags::default(),
        }
    }
    pub fn spin(&self) -> i8 {
        self.spin
    }
    pub fn read_spin(&mut self, number: Option<Number>) {
        self.spin = number.map_or(0, Number::byte);
    }
    /// Called after the inherited placement callback. Horizontal sides keep
    /// their previous spin; preserve float32 multiplication/division order.
    pub fn on_placement(&mut self, side: Direction, yaw: f32) {
        let rotation = floor(f64::from(yaw * 4.0 / 360.0) + 2.5) & 3;
        if matches!(side, Direction::Up | Direction::Down) {
            self.spin = rotation as i8;
        }
    }
    /// The caller checks canWrenchRotate first. True requests both markForUpdate
    /// and markForSave; a client consumes the use without changing state.
    pub fn wrench_turn(&mut self, client: bool) -> bool {
        if client {
            return false;
        }
        self.spin = ((i32::from(self.spin) + 1) % 4) as i8;
        true
    }
    pub fn neighbor_changed(&mut self, pos: [i32; 3], side: Direction, neighbor: [i32; 3]) -> bool {
        let offset = side.offset();
        let adjacent = std::array::from_fn(|i| pos[i].wrapping_add(offset[i]));
        if adjacent == neighbor {
            self.opacity = -1;
            true
        } else {
            false
        }
    }
    pub fn light(
        &mut self,
        powered: bool,
        light_source: bool,
        query_opacity: impl FnOnce() -> i32,
    ) -> i32 {
        if self.opacity == -1 {
            self.opacity = query_opacity();
        }
        let emit: i32 = if powered {
            if light_source { 15 } else { 9 }
        } else {
            0
        };
        emit.wrapping_sub(self.opacity).max(0)
    }
    pub fn write_stream(&mut self, flags: Flags) -> [u8; 2] {
        self.flags = flags;
        [flags.byte(), self.spin as u8]
    }
    pub fn read_stream(&mut self, bytes: [u8; 2], send_power: bool, send_missing: bool) -> bool {
        let changed = self.flags.read(bytes[0], send_power, send_missing);
        let old = self.spin;
        self.spin = bytes[1] as i8;
        changed || old != self.spin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loaded_spin_is_signed_and_wrench_uses_java_remainder() {
        let mut part = Reporting::new(-128);
        assert!(!part.wrench_turn(true));
        assert_eq!(part.spin(), -128);
        assert!(part.wrench_turn(false));
        assert_eq!(part.spin(), -3);
        part.wrench_turn(false);
        assert_eq!(part.spin(), -2);
        part.on_placement(Direction::North, 0.0);
        assert_eq!(part.spin(), -2);
        part.on_placement(Direction::Up, 0.0);
        assert_eq!(part.spin(), 2);
    }
    #[test]
    fn saved_floats_floor_before_narrowing_and_missing_spin_defaults_to_zero() {
        let mut part = Reporting::default();
        part.read_spin(Some(Number::Double(-0.1)));
        assert_eq!(part.spin(), -1);
        part.read_spin(Some(Number::Float(f32::NEG_INFINITY)));
        assert_eq!(part.spin(), -1);
        part.read_spin(Some(Number::Double(f64::NAN)));
        assert_eq!(part.spin(), 0);
        part.read_spin(None);
        assert_eq!(part.spin(), 0);
    }
    #[test]
    fn opacity_is_lazy_even_when_off_and_only_facing_neighbor_invalidates_it() {
        let mut part = Reporting::default();
        let mut queries = 0;
        assert_eq!(
            part.light(false, false, || {
                queries += 1;
                4
            }),
            0
        );
        assert_eq!(part.light(true, false, || panic!("cached")), 5);
        assert!(!part.neighbor_changed([0; 3], Direction::North, [0, 0, 1]));
        assert_eq!(part.light(true, true, || panic!("cached")), 11);
        assert!(part.neighbor_changed([0; 3], Direction::North, [0, 0, -1]));
        assert_eq!(
            part.light(true, false, || {
                queries += 1;
                8
            }),
            1
        );
        assert_eq!(queries, 2);
    }
    #[test]
    fn stream_updates_suppressed_flags_but_still_reports_spin_changes() {
        let mut part = Reporting::default();
        assert!(!part.read_stream([3, 0], false, false));
        assert!(part.flags.powered);
        assert!(part.flags.missing_channel);
        assert!(part.read_stream([3, 255], false, false));
        assert_eq!(part.spin(), -1);
        let flags = Flags {
            powered: true,
            missing_channel: false,
        };
        assert_eq!(part.write_stream(flags), [1, 255]);
        assert_eq!(part.flags.model(), Model::HasChannel);
    }
}
