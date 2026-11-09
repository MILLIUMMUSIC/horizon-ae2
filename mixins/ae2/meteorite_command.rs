// SPDX-License-Identifier: LGPL-3.0-or-later
// Explicit inputs for an operator-only development probe, not worldgen admission.
use super::ae2_meteorite_fallout::{CraterType, FalloutMode};
use super::ae2_meteorite_placer::{Bounds, Settings};
pub const USAGE: &str = "ae2meteorite <settings:text>";
pub const ARGUMENTS: &str =
    "x y z radius crater fallout pure lake flawless presses minX minY minZ maxX maxY maxZ seaLevel";
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub settings: Settings,
    pub bounds: Bounds,
    pub sea_level: i32,
}
pub fn parse(words: &[&str]) -> Result<Request, &'static str> {
    if words.len() != 17 {
        return Err(ARGUMENTS);
    }
    let integer = |i: usize| words[i].parse::<i32>().map_err(|_| "Invalid integer");
    let boolean = |i: usize| {
        words[i]
            .parse::<bool>()
            .map_err(|_| "Expected true or false")
    };
    let radius: f32 = words[3].parse().map_err(|_| "Invalid radius")?;
    if !radius.is_finite() || !(2.0..=8.0).contains(&radius) {
        return Err("Development probe radius must be finite and between 2 and 8");
    }
    let crater = match words[4] {
        "none" => CraterType::None,
        "normal" => CraterType::Normal,
        "lava" => CraterType::Lava,
        "obsidian" => CraterType::Obsidian,
        "water" => CraterType::Water,
        "snow" => CraterType::Snow,
        "ice" => CraterType::Ice,
        _ => return Err("Unknown crater: none normal lava obsidian water snow ice"),
    };
    let fallout = match words[5] {
        "none" => FalloutMode::None,
        "default" => FalloutMode::Default,
        "sand" => FalloutMode::Sand,
        "terracotta" => FalloutMode::Terracotta,
        "ice_snow" => FalloutMode::IceSnow,
        _ => return Err("Unknown fallout: none default sand terracotta ice_snow"),
    };
    let pos = [integer(0)?, integer(1)?, integer(2)?];
    let bounds = Bounds {
        min: [integer(10)?, integer(11)?, integer(12)?],
        max: [integer(13)?, integer(14)?, integer(15)?],
    };
    let sea_level = integer(16)?;
    // Diagnostic admission limits must not enter the original placer program.
    if pos
        .into_iter()
        .chain(bounds.min)
        .chain(bounds.max)
        .chain([sea_level])
        .any(|n| !(-29_999_900..=29_999_900).contains(&n))
        || (0..3).any(|i| bounds.min[i] > bounds.max[i])
        || [0, 2]
            .into_iter()
            .any(|i| i64::from(bounds.max[i]) - i64::from(bounds.min[i]) >= 256)
    {
        return Err("Invalid bounds or slice wider than 256 blocks");
    }
    Ok(Request {
        settings: Settings {
            pos,
            radius,
            crater,
            fallout,
            pure: boolean(6)?,
            lake: boolean(7)?,
            flawless_only: boolean(8)?,
            spawn_presses: boolean(9)?,
        },
        bounds,
        sea_level,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn probe_preserves_settings_and_disjoint_slice() {
        let args = "-8 40 7 6.75 ice ice_snow false true true false 0 -64 0 15 319 15 -63";
        let r = parse(&args.split_whitespace().collect::<Vec<_>>()).unwrap();
        assert_eq!(r.settings.pos, [-8, 40, 7]);
        assert_eq!(r.settings.crater, CraterType::Ice);
        assert!(r.settings.flawless_only && r.settings.lake);
        assert!(!r.settings.spawn_presses && !r.settings.pure);
        assert_eq!(r.bounds.center(), [8, 128, 8]);
        assert_eq!(r.sea_level, -63);
    }
    #[test]
    fn invalid_probe_fails_before_world_access() {
        let words: Vec<_> = "0 40 0 4 lava default true false false true -8 -64 -8 8 319 8 63"
            .split_whitespace()
            .collect();
        for (i, values) in [
            (3, vec!["NaN", "inf", "-1", "1.99", "8.01"]),
            (4, vec!["unknown"]),
            (5, vec!["unknown"]),
            (6, vec!["1"]),
            (10, vec!["9", "-2147483648"]),
            (13, vec!["248"]),
            (16, vec!["2147483647"]),
        ] {
            for value in values {
                let mut invalid = words.clone();
                invalid[i] = value;
                assert!(parse(&invalid).is_err(), "{i}: {value}");
            }
        }
        assert!(parse(&words[..16]).is_err());
    }
}
