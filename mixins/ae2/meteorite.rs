// SPDX-License-Identifier: LGPL-3.0-or-later
// MeteoritePlacer (AE2 19.2.18): sky-stone ellipsoid, mysterious cube,
// crater bowl, fallout blocks, and crater-lake water.
// ae2meteorite writes all of those. Press drops stay on the cube.
// Copyright (c) AlgorithmX2 / AE2 contributors; 2026 Horizon port contributors.

use super::ae2_meteorite_fallout as meteorite_fallout;

pub use meteorite_fallout::{
    ColumnKind, CopyBiome, CraterType, FalloutMode, Site, column_kind, copy_block, crater_squared,
    enclosing_block, fall_block, fall_budget, fall_height, filler, inset_block, inset_reaches,
};

pub const SKY_STONE: &str = "ae2:sky_stone_block";
pub const MYSTERIOUS_CUBE: &str = "ae2:mysterious_cube";
pub const PRESSES_TAG: &str = "ae2:inscriber_presses";
pub const GUIDE: &str = "ae2:guide";
/// `spawnPressesInMeteorites` default.
pub const SPAWN_PRESSES: bool = true;
/// `spawnFlawlessOnly` default.
pub const FLAWLESS_ONLY: bool = false;

const QUARTZ: &[&str] = &[
    "ae2:quartz_block",
    "ae2:damaged_budding_quartz",
    "ae2:chipped_budding_quartz",
    "ae2:flawed_budding_quartz",
    "ae2:flawless_budding_quartz",
];
const BUDS: &[&str] = &[
    "ae2:small_quartz_bud",
    "ae2:medium_quartz_bud",
    "ae2:large_quartz_bud",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: &'static str,
    /// Buds use `AmethystClusterBlock.FACING = UP`.
    pub facing_up: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LakePlaced {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: &'static str,
    /// `scheduleTick(pos, Fluids.WATER, 0)` when this cell is at sea level - 1.
    pub schedule_water_tick: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlopeSite {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LakePlacement {
    pub water: Vec<LakePlaced>,
    pub slopes: Vec<SlopeSite>,
}

pub trait Rng {
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

/// Sky-stone ellipsoid plus the center chest. `radius` is `PlacedMeteoriteSettings`
/// meteorite radius (`nextFloat * 6 + 2` in `MeteoriteStructure`).
pub fn place_sky_stone(x: i32, y: i32, z: i32, radius: f64, rng: &mut impl Rng) -> Vec<Placed> {
    let quartz = if FLAWLESS_ONLY { &QUARTZ[4..] } else { QUARTZ };
    let squared = radius * radius;
    let mut out = Vec::new();
    let x0 = x - 8;
    let x1 = x + 8;
    let z0 = z - 8;
    let z1 = z + 8;
    for i in x0..=x1 {
        for j in (y - 8)..(y + 8) {
            for k in z0..=z1 {
                let dx = i - x;
                let dy = j - y;
                let dz = k - z;
                let dy_scale = if j > y { 1.4 } else { 0.8 };
                let distance = f64::from(dx * dx) * 0.7
                    + f64::from(dy * dy) * dy_scale
                    + f64::from(dz * dz) * 0.7;
                if distance >= squared {
                    continue;
                }
                if dx.abs() <= 1 && dy.abs() <= 1 && dz.abs() <= 1 {
                    if dy == -1 {
                        let index = rng.next_int(quartz.len() as i32) as usize;
                        out.push(Placed {
                            x: i,
                            y: j,
                            z: k,
                            block: quartz[index],
                            facing_up: false,
                        });
                        if index != 0 && (dx != 0 || dz != 0) && rng.next_float() <= 0.7 {
                            let bud = BUDS[rng.next_int(BUDS.len() as i32) as usize];
                            out.push(Placed {
                                x: i,
                                y: j + 1,
                                z: k,
                                block: bud,
                                facing_up: true,
                            });
                        }
                    }
                } else {
                    out.push(Placed {
                        x: i,
                        y: j,
                        z: k,
                        block: SKY_STONE,
                        facing_up: false,
                    });
                }
            }
        }
    }
    if SPAWN_PRESSES {
        out.push(Placed {
            x,
            y,
            z,
            block: MYSTERIOUS_CUBE,
            facing_up: false,
        });
    }
    out
}

/// One draw for [`place_sky_stone_rolls`]. The slot field matching the ask is written.
#[derive(Clone, Copy, Debug)]
pub enum SkyRoll {
    Int(i32),
    Float,
}

#[derive(Clone, Copy, Debug)]
pub struct RollSlot {
    pub int: i32,
    pub float: f32,
}

/// `place_sky_stone` fed by one closure, so the command can share `RandomSource`.
pub fn place_sky_stone_rolls(
    x: i32,
    y: i32,
    z: i32,
    radius: f64,
    roll: impl FnMut(SkyRoll, &mut RollSlot),
) -> Vec<Placed> {
    struct Rolls<F>(F);
    impl<F: FnMut(SkyRoll, &mut RollSlot)> Rng for Rolls<F> {
        fn next_int(&mut self, bound: i32) -> i32 {
            let mut slot = RollSlot { int: 0, float: 0.0 };
            (self.0)(SkyRoll::Int(bound), &mut slot);
            slot.int
        }
        fn next_float(&mut self) -> f32 {
            let mut slot = RollSlot { int: 0, float: 0.0 };
            (self.0)(SkyRoll::Float, &mut slot);
            slot.float
        }
    }
    place_sky_stone(x, y, z, radius, &mut Rolls(roll))
}

/// `placeCrater`. Sand and snow fallout raise the bowl by `adjustCrater`.
pub fn place_crater(
    site: Site,
    mode: FalloutMode,
    kind: CraterType,
    max_build_height: i32,
    solid: impl FnMut(i32, i32, i32) -> bool,
) -> Vec<Placed> {
    meteorite_fallout::crater_blocks(site, mode, kind, max_build_height, solid)
        .into_iter()
        .map(as_placed)
        .collect()
}

/// `placeCraterLake`. Slope columns still need `place_pillar` to write fallout.
pub fn place_lake(
    site: Site,
    mode: FalloutMode,
    sea_level: i32,
    is_air: impl FnMut(i32, i32, i32) -> bool,
) -> LakePlacement {
    let scan = meteorite_fallout::scan_lake(site, mode, sea_level, is_air);
    LakePlacement {
        water: scan
            .water
            .into_iter()
            .map(|cell| LakePlaced {
                x: cell.x,
                y: cell.y,
                z: cell.z,
                block: cell.block,
                schedule_water_tick: cell.schedule_water_tick,
            })
            .collect(),
        slopes: scan
            .slopes
            .into_iter()
            .map(|cell| SlopeSite {
                x: cell.x,
                y: cell.y,
                z: cell.z,
            })
            .collect(),
    }
}

/// One `pillarDownSlopeBlocks` walk. The block closure is `enclosing_block`.
pub fn place_pillar(
    x: i32,
    y: i32,
    z: i32,
    open: impl FnMut(i32, i32, i32) -> ColumnKind,
    block: impl FnMut(i32, i32, i32) -> Option<&'static str>,
) -> Vec<Placed> {
    meteorite_fallout::pillar_blocks(x, y, z, open, block)
        .into_iter()
        .map(as_placed)
        .collect()
}

fn as_placed(block: meteorite_fallout::BlockAt) -> Placed {
    Placed {
        x: block.x,
        y: block.y,
        z: block.z,
        block: block.block,
        facing_up: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Zero;
    impl Rng for Zero {
        fn next_int(&mut self, bound: i32) -> i32 {
            assert!(bound > 0);
            0
        }
        fn next_float(&mut self) -> f32 {
            0.0
        }
    }

    #[test]
    fn ellipsoid_places_sky_stone_and_the_center_cube() {
        let placed = place_sky_stone(0, 0, 0, 4.0, &mut Zero);
        let at = |x, y, z| {
            placed
                .iter()
                .rev()
                .find(|block| block.x == x && block.y == y && block.z == z)
                .map(|block| block.block)
        };
        assert_eq!(at(0, 0, 0), Some(MYSTERIOUS_CUBE));
        assert_eq!(at(2, 0, 0), Some(SKY_STONE));
        assert_eq!(at(10, 0, 0), None);
        assert_eq!(at(0, -1, 0), Some("ae2:quartz_block"));
        assert!(placed.iter().any(|block| block.block == SKY_STONE));
    }

    #[test]
    fn placer_crater_and_lake_follow_the_fallout_mode() {
        let site = Site {
            x: 0,
            y: 10,
            z: 0,
            radius: 4.0,
            min_x: 0,
            max_x: 0,
            min_z: 0,
            max_z: 0,
        };
        let low = place_crater(
            site,
            FalloutMode::Default,
            CraterType::Normal,
            12,
            |_, _, _| false,
        );
        let high = place_crater(
            site,
            FalloutMode::Sand,
            CraterType::Normal,
            12,
            |_, _, _| false,
        );
        assert!(
            low.iter()
                .any(|block| block.y == 8 && block.block == "minecraft:air")
        );
        assert!(!high.iter().any(|block| block.y == 8));
        let lake = place_lake(site, FalloutMode::Default, 12, |_, y, _| y == 11);
        assert!(lake.water.iter().any(|block| {
            block.y == 11 && block.block == "minecraft:water" && block.schedule_water_tick
        }));
        assert!(lake.water.iter().all(|block| block.y != 8));
        let pillar = place_pillar(
            0,
            1,
            0,
            |_, y, _| {
                if y == 1 {
                    ColumnKind::Open
                } else {
                    ColumnKind::Stop
                }
            },
            |_, _, _| Some("minecraft:gravel"),
        );
        assert_eq!(pillar.len(), 1);
        assert_eq!(pillar[0].block, "minecraft:gravel");
        assert!(!pillar[0].facing_up);
    }
}
