// SPDX-License-Identifier: LGPL-3.0-or-later
// MeteoritePlacer crater, fallout, and crater lake (AE2 19.2.18).
// FalloutCopy / FalloutSand / FalloutSnow and CraterType.
// Copyright (c) AlgorithmX2 / AE2 contributors; 2026 Horizon port contributors.

/// Saved `PlacedMeteoriteSettings` crater ordinal. Append only; the source says not to reorder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CraterType {
    None = 0,
    Normal = 1,
    Lava = 2,
    Obsidian = 3,
    Water = 4,
    Snow = 5,
    Ice = 6,
}

/// Saved fallout ordinal. `getFallout` uses `Fallout` for `NONE` and `DEFAULT`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FalloutMode {
    None = 0,
    Default = 1,
    Sand = 2,
    Terracotta = 3,
    IceSnow = 4,
}

/// Tags `FalloutMode.fromBiome` reads, in enum order. `NONE` and `DEFAULT` match nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BiomeTags {
    pub sandy: bool,
    pub beach: bool,
    pub badlands: bool,
    pub cold: bool,
}

/// Tags `FalloutCopy` reads at the meteor center. One block is chosen for the whole meteor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CopyBiome {
    pub badlands: bool,
    pub snowy: bool,
    pub beach: bool,
    pub sandy: bool,
    pub plains: bool,
    pub forest: bool,
}

/// Chunk slice passed into `MeteoritePlacer` (`StructurePiece.postProcess` bounds).
#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub radius: f64,
    pub min_x: i32,
    pub max_x: i32,
    pub min_z: i32,
    pub max_z: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockAt {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LakeWater {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub block: &'static str,
    pub schedule_water_tick: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlopeAt {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LakeScan {
    pub water: Vec<LakeWater>,
    pub slopes: Vec<SlopeAt>,
}

/// `placeEnclosingBlock`: air, or a dry block that can be replaced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnKind {
    Open,
    Stop,
}

pub const SKY_STONE: &str = "ae2:sky_stone_block";

const SPECIFIED_BLOCK_THRESHOLD: f32 = 0.9;
const AIR_BLOCK_THRESHOLD: f32 = 0.8;
const BLOCK_THRESHOLD_STEP: f32 = 0.1;
const GLASS_THRESHOLD: f32 = 0.66;
const SNOW_THRESHOLD: f32 = 0.7;
const ICE_THRESHOLD: f32 = 0.5;
/// `level.getRandom().nextFloat() < 0.075f` on a lava crater's enclosing column.
const MAGMA_CHANCE: f32 = 0.075;
const BOWL_SLOPE: f64 = 0.02;

pub fn should_place_crater(kind: CraterType) -> bool {
    kind != CraterType::None
}

pub fn filler(kind: CraterType) -> &'static str {
    match kind {
        CraterType::None | CraterType::Normal => "minecraft:air",
        CraterType::Lava => "minecraft:lava",
        CraterType::Obsidian => "minecraft:obsidian",
        CraterType::Water => "minecraft:water",
        CraterType::Snow => "minecraft:snow_block",
        CraterType::Ice => "minecraft:ice",
    }
}

pub fn fallout_mode(tags: BiomeTags) -> FalloutMode {
    if tags.sandy || tags.beach {
        FalloutMode::Sand
    } else if tags.badlands {
        FalloutMode::Terracotta
    } else if tags.cold {
        FalloutMode::IceSnow
    } else {
        FalloutMode::Default
    }
}

pub fn copy_block(biome: CopyBiome) -> &'static str {
    if biome.badlands {
        "minecraft:terracotta"
    } else if biome.snowy {
        "minecraft:snow_block"
    } else if biome.beach || biome.sandy {
        "minecraft:sand"
    } else if biome.plains || biome.forest {
        "minecraft:dirt"
    } else {
        "minecraft:cobblestone"
    }
}

/// `FalloutSand` and `FalloutSnow` return 2. `Fallout` and `FalloutCopy` return 0.
pub fn adjust_crater(mode: FalloutMode) -> i32 {
    match mode {
        FalloutMode::Sand | FalloutMode::IceSnow => 2,
        FalloutMode::None | FalloutMode::Default | FalloutMode::Terracotta => 0,
    }
}

/// Block `getRandomFall` would put. `None` means that roll does not call `put`.
pub fn fall_block(mode: FalloutMode, specified: &'static str, roll: f32) -> Option<&'static str> {
    match mode {
        FalloutMode::None | FalloutMode::Default => Some(default_fall(roll)),
        FalloutMode::Sand | FalloutMode::Terracotta | FalloutMode::IceSnow => {
            if roll > SPECIFIED_BLOCK_THRESHOLD {
                Some(specified)
            } else {
                get_other(mode, roll)
            }
        }
    }
}

/// Block `getRandomInset` would put. Default insets always put, including air.
pub fn inset_block(mode: FalloutMode, specified: &'static str, roll: f32) -> Option<&'static str> {
    match mode {
        FalloutMode::None | FalloutMode::Default => Some(default_inset(roll)),
        FalloutMode::Sand | FalloutMode::Terracotta | FalloutMode::IceSnow => {
            if roll > SPECIFIED_BLOCK_THRESHOLD {
                Some(specified)
            } else if roll > AIR_BLOCK_THRESHOLD {
                Some("minecraft:air")
            } else {
                get_other(mode, roll - BLOCK_THRESHOLD_STEP)
            }
        }
    }
}

/// Lava craters roll magma on `Level.getRandom` before the meteorite fallout roll.
pub fn enclosing_block(
    kind: CraterType,
    magma_roll: f32,
    mode: FalloutMode,
    specified: &'static str,
    fall_roll: f32,
) -> Option<&'static str> {
    if kind == CraterType::Lava && magma_roll < MAGMA_CHANCE {
        Some("minecraft:magma_block")
    } else {
        fall_block(mode, specified, fall_roll)
    }
}

fn default_fall(roll: f32) -> &'static str {
    if roll > 0.9 {
        "minecraft:stone"
    } else if roll > 0.8 {
        "minecraft:cobblestone"
    } else if roll > 0.7 {
        "minecraft:dirt"
    } else {
        "minecraft:gravel"
    }
}

fn default_inset(roll: f32) -> &'static str {
    if roll > 0.9 {
        "minecraft:cobblestone"
    } else if roll > 0.8 {
        "minecraft:stone"
    } else if roll > 0.7 {
        "minecraft:grass_block"
    } else if roll > 0.6 {
        SKY_STONE
    } else if roll > 0.5 {
        "minecraft:gravel"
    } else {
        "minecraft:air"
    }
}

fn get_other(mode: FalloutMode, roll: f32) -> Option<&'static str> {
    match mode {
        FalloutMode::Sand => {
            if roll > GLASS_THRESHOLD {
                Some("minecraft:glass")
            } else {
                None
            }
        }
        FalloutMode::IceSnow => {
            if roll > SNOW_THRESHOLD {
                Some("minecraft:snow")
            } else if roll > ICE_THRESHOLD {
                Some("minecraft:ice")
            } else {
                None
            }
        }
        FalloutMode::None | FalloutMode::Default | FalloutMode::Terracotta => None,
    }
}

fn bowl_height(origin_y: i32, radius: f64, adjust: i32) -> f64 {
    f64::from(origin_y) - radius + 1.0 + f64::from(adjust)
}

fn horizontal_distance(x: i32, z: i32, origin_x: i32, origin_z: i32) -> f64 {
    let dx = f64::from(x.wrapping_sub(origin_x));
    let dz = f64::from(z.wrapping_sub(origin_z));
    dx * dx + dz * dz
}

pub fn in_crater_bowl(site: Site, x: i32, y: i32, z: i32, adjust: i32) -> bool {
    let distance = horizontal_distance(x, z, site.x, site.z);
    let height = bowl_height(site.y, site.radius, adjust);
    f64::from(y) > height + distance * BOWL_SLOPE
}

/// `placeCrater`. `max_build_height` is inclusive, matching `j <= level.getMaxBuildHeight()`.
/// The y loop starts at `originY - 5`. `NONE` places nothing.
pub fn crater_blocks(
    site: Site,
    mode: FalloutMode,
    kind: CraterType,
    max_build_height: i32,
    mut solid: impl FnMut(i32, i32, i32) -> bool,
) -> Vec<BlockAt> {
    if !should_place_crater(kind) || site.min_x > site.max_x || site.min_z > site.max_z {
        return Vec::new();
    }
    let start_y = site.y.wrapping_sub(5);
    if start_y > max_build_height {
        return Vec::new();
    }
    let adjust = adjust_crater(mode);
    let mut out = Vec::new();
    for y in start_y..=max_build_height {
        for x in site.min_x..=site.max_x {
            for z in site.min_z..=site.max_z {
                if !in_crater_bowl(site, x, y, z, adjust) {
                    continue;
                }
                let block = if kind != CraterType::Normal && y < site.y && solid(x, y, z) {
                    filler(kind)
                } else {
                    "minecraft:air"
                };
                out.push(BlockAt { x, y, z, block });
            }
        }
    }
    out
}

/// `placeCraterLake` water, plus columns that start `pillarDownSlopeBlocks`.
/// Water replaces `Blocks.AIR` only. The top water cell schedules a fluid tick of delay 0.
pub fn scan_lake(
    site: Site,
    mode: FalloutMode,
    sea_level: i32,
    mut is_air: impl FnMut(i32, i32, i32) -> bool,
) -> LakeScan {
    let mut water = Vec::new();
    let mut slopes = Vec::new();
    if site.min_x > site.max_x || site.min_z > site.max_z {
        return LakeScan { water, slopes };
    }
    let start_y = site.y.wrapping_sub(5);
    let max_y = sea_level.wrapping_sub(1);
    if start_y > max_y {
        return LakeScan { water, slopes };
    }
    let adjust = adjust_crater(mode);
    let height = bowl_height(site.y, site.radius, adjust);
    for x in site.min_x..=site.max_x {
        for z in site.min_z..=site.max_z {
            for y in start_y..=max_y {
                let distance = horizontal_distance(x, z, site.x, site.z);
                if in_crater_bowl(site, x, y, z, adjust) {
                    if is_air(x, y, z) {
                        water.push(LakeWater {
                            x,
                            y,
                            z,
                            block: "minecraft:water",
                            schedule_water_tick: y == max_y,
                        });
                    }
                } else if slope_trigger(y, sea_level, height, distance) {
                    slopes.push(SlopeAt { x, y, z });
                }
            }
        }
    }
    LakeScan { water, slopes }
}

fn slope_trigger(y: i32, sea_level: i32, height: f64, distance: f64) -> bool {
    let max_y = sea_level.wrapping_sub(1);
    let left = max_y
        .wrapping_add(max_y.wrapping_sub(y).wrapping_mul(2))
        .wrapping_add(2);
    f64::from(left) > height + distance * BOWL_SLOPE
}

pub fn column_kind(
    is_air: bool,
    fluid_empty: bool,
    can_be_replaced: bool,
    replaceable_tag: bool,
) -> ColumnKind {
    if is_air || (fluid_empty && (can_be_replaced || replaceable_tag)) {
        ColumnKind::Open
    } else {
        ColumnKind::Stop
    }
}

/// Walks down at most 20 blocks, as `pillarDownSlopeBlocks` does.
/// `None` from `block` still consumes the step: `getOther` may put nothing.
pub fn pillar_blocks(
    x: i32,
    mut y: i32,
    z: i32,
    mut open: impl FnMut(i32, i32, i32) -> ColumnKind,
    mut block: impl FnMut(i32, i32, i32) -> Option<&'static str>,
) -> Vec<BlockAt> {
    let mut out = Vec::new();
    for _ in 0..20 {
        if open(x, y, z) == ColumnKind::Stop {
            break;
        }
        if let Some(block) = block(x, y, z) {
            out.push(BlockAt { x, y, z, block });
        }
        y = y.wrapping_sub(1);
    }
    out
}

/// `(meteoriteRadius * 2 + 5) ^ 2`, the decay crater term.
pub fn crater_squared(radius: f64) -> f64 {
    let real = radius * 2.0 + 5.0;
    real * real
}

pub fn fall_budget(radius: f64) -> f64 {
    100.0 * crater_squared(radius)
}

pub fn fall_height(crater_sq: f64, extra_range: f64, dist: f64) -> f64 {
    crater_sq * (extra_range + 0.2) - (dist - crater_sq * 1.7).abs()
}

/// Decay inset reaches this cell. The center 3x3x3 room is left alone.
pub fn inset_reaches(dx: i32, dy: i32, dz: i32, crater_sq: f64) -> bool {
    if dx.abs() <= 1 && dy.abs() <= 1 && dz.abs() <= 1 {
        return false;
    }
    let dx = f64::from(dx);
    let dy = f64::from(dy);
    let dz = f64::from(dz);
    let distance = dx * dx + dy * dy + dz * dz;
    distance < crater_sq * 1.6
}

/// Pure craters do not decay their own filler. The rest of decay reads the level.
pub fn decay_skips(pure_crater: bool, block: &str, filler_block: &str) -> bool {
    pure_crater && block == filler_block
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(cells: &[BlockAt], x: i32, y: i32, z: i32) -> Option<&'static str> {
        cells
            .iter()
            .find(|cell| cell.x == x && cell.y == y && cell.z == z)
            .map(|cell| cell.block)
    }

    fn column(y: i32, radius: f64, max_x: i32, max_z: i32) -> Site {
        Site {
            x: 0,
            y,
            z: 0,
            radius,
            min_x: 0,
            max_x,
            min_z: 0,
            max_z,
        }
    }

    #[test]
    fn crater_and_fallout_ordinals_match_saved_settings() {
        assert_eq!(CraterType::None as u8, 0);
        assert_eq!(CraterType::Ice as u8, 6);
        assert_eq!(FalloutMode::None as u8, 0);
        assert_eq!(FalloutMode::IceSnow as u8, 4);
        assert!(should_place_crater(CraterType::Normal));
        assert!(!should_place_crater(CraterType::None));
    }

    #[test]
    fn fallout_mode_follows_enum_tag_order() {
        assert_eq!(fallout_mode(BiomeTags::default()), FalloutMode::Default);
        assert_eq!(
            fallout_mode(BiomeTags {
                cold: true,
                ..BiomeTags::default()
            }),
            FalloutMode::IceSnow
        );
        assert_eq!(
            fallout_mode(BiomeTags {
                badlands: true,
                ..BiomeTags::default()
            }),
            FalloutMode::Terracotta
        );
        assert_eq!(
            fallout_mode(BiomeTags {
                sandy: true,
                badlands: true,
                cold: true,
                ..BiomeTags::default()
            }),
            FalloutMode::Sand
        );
        assert_eq!(
            fallout_mode(BiomeTags {
                beach: true,
                ..BiomeTags::default()
            }),
            FalloutMode::Sand
        );
    }

    #[test]
    fn copy_block_uses_fallout_copy_priority() {
        assert_eq!(copy_block(CopyBiome::default()), "minecraft:cobblestone");
        assert_eq!(
            copy_block(CopyBiome {
                sandy: true,
                badlands: true,
                ..CopyBiome::default()
            }),
            "minecraft:terracotta"
        );
        assert_eq!(
            copy_block(CopyBiome {
                snowy: true,
                ..CopyBiome::default()
            }),
            "minecraft:snow_block"
        );
        assert_eq!(
            copy_block(CopyBiome {
                beach: true,
                ..CopyBiome::default()
            }),
            "minecraft:sand"
        );
        assert_eq!(
            copy_block(CopyBiome {
                plains: true,
                ..CopyBiome::default()
            }),
            "minecraft:dirt"
        );
        assert_eq!(
            copy_block(CopyBiome {
                forest: true,
                ..CopyBiome::default()
            }),
            "minecraft:dirt"
        );
        assert_eq!(adjust_crater(FalloutMode::Sand), 2);
        assert_eq!(adjust_crater(FalloutMode::IceSnow), 2);
        assert_eq!(adjust_crater(FalloutMode::Terracotta), 0);
        assert_eq!(adjust_crater(FalloutMode::Default), 0);
    }

    #[test]
    fn default_fallout_block_list() {
        let mode = FalloutMode::Default;
        let specified = "minecraft:sand";
        assert_eq!(fall_block(mode, specified, 0.95), Some("minecraft:stone"));
        assert_eq!(
            fall_block(mode, specified, 0.9),
            Some("minecraft:cobblestone")
        );
        assert_eq!(fall_block(mode, specified, 0.8), Some("minecraft:dirt"));
        assert_eq!(fall_block(mode, specified, 0.7), Some("minecraft:gravel"));
        assert_eq!(
            fall_block(FalloutMode::None, specified, 0.95),
            Some("minecraft:stone")
        );
        assert_eq!(
            inset_block(mode, specified, 0.95),
            Some("minecraft:cobblestone")
        );
        assert_eq!(inset_block(mode, specified, 0.9), Some("minecraft:stone"));
        assert_eq!(
            inset_block(mode, specified, 0.8),
            Some("minecraft:grass_block")
        );
        assert_eq!(inset_block(mode, specified, 0.7), Some(SKY_STONE));
        assert_eq!(inset_block(mode, specified, 0.6), Some("minecraft:gravel"));
        assert_eq!(inset_block(mode, specified, 0.5), Some("minecraft:air"));
    }

    #[test]
    fn sand_fallout_block_list() {
        let mode = FalloutMode::Sand;
        let specified = copy_block(CopyBiome {
            sandy: true,
            ..CopyBiome::default()
        });
        assert_eq!(specified, "minecraft:sand");
        assert_eq!(fall_block(mode, specified, 0.95), Some("minecraft:sand"));
        assert_eq!(fall_block(mode, specified, 0.9), Some("minecraft:glass"));
        assert_eq!(fall_block(mode, specified, 0.66), None);
        assert_eq!(inset_block(mode, specified, 0.95), Some("minecraft:sand"));
        assert_eq!(inset_block(mode, specified, 0.85), Some("minecraft:air"));
        assert_eq!(inset_block(mode, specified, 0.79), Some("minecraft:glass"));
        assert_eq!(inset_block(mode, specified, 0.75), None);
    }

    #[test]
    fn terracotta_fallout_places_only_its_biome_block_or_inset_air() {
        let mode = FalloutMode::Terracotta;
        let specified = "minecraft:terracotta";
        assert_eq!(fall_block(mode, specified, 0.95), Some(specified));
        assert_eq!(fall_block(mode, specified, 0.9), None);
        assert_eq!(inset_block(mode, specified, 0.85), Some("minecraft:air"));
        assert_eq!(inset_block(mode, specified, 0.5), None);
    }

    #[test]
    fn ice_snow_fallout_block_list() {
        let mode = FalloutMode::IceSnow;
        let snowy = copy_block(CopyBiome {
            snowy: true,
            ..CopyBiome::default()
        });
        assert_eq!(snowy, "minecraft:snow_block");
        assert_eq!(fall_block(mode, snowy, 0.95), Some("minecraft:snow_block"));
        assert_eq!(fall_block(mode, snowy, 0.75), Some("minecraft:snow"));
        assert_eq!(fall_block(mode, snowy, 0.7), Some("minecraft:ice"));
        assert_eq!(fall_block(mode, snowy, 0.5), None);
        let cold_only = copy_block(CopyBiome::default());
        assert_eq!(cold_only, "minecraft:cobblestone");
        assert_eq!(
            fall_block(mode, cold_only, 0.95),
            Some("minecraft:cobblestone")
        );
        assert_eq!(inset_block(mode, snowy, 0.85), Some("minecraft:air"));
        assert_eq!(inset_block(mode, snowy, 0.75), Some("minecraft:ice"));
        assert_eq!(inset_block(mode, snowy, 0.6), None);
    }

    #[test]
    fn normal_crater_cuts_the_paraboloid() {
        let site = column(10, 4.0, 10, 0);
        let cells = crater_blocks(
            site,
            FalloutMode::Default,
            CraterType::Normal,
            12,
            |_, _, _| panic!("normal craters do not read solidity"),
        );
        assert_eq!(at(&cells, 0, 8, 0), Some("minecraft:air"));
        assert_eq!(at(&cells, 0, 7, 0), None);
        assert_eq!(at(&cells, 7, 8, 0), Some("minecraft:air"));
        assert_eq!(at(&cells, 8, 8, 0), None);
        assert_eq!(at(&cells, 10, 9, 0), None);
        assert_eq!(at(&cells, 10, 10, 0), Some("minecraft:air"));
        assert_eq!(at(&cells, 0, 12, 0), Some("minecraft:air"));
        let across = crater_blocks(
            column(10, 4.0, 0, 8),
            FalloutMode::Default,
            CraterType::Normal,
            12,
            |_, _, _| false,
        );
        assert_eq!(at(&across, 0, 8, 7), Some("minecraft:air"));
        assert_eq!(at(&across, 0, 8, 8), None);
    }

    #[test]
    fn crater_loop_starts_five_below_the_origin_and_includes_max_height() {
        let site = column(10, 20.0, 0, 0);
        let cells = crater_blocks(
            site,
            FalloutMode::Default,
            CraterType::Normal,
            6,
            |_, _, _| false,
        );
        assert!(cells.iter().any(|cell| cell.y == 5));
        assert!(cells.iter().all(|cell| (5..=6).contains(&cell.y)));
        assert!(
            crater_blocks(
                site,
                FalloutMode::Default,
                CraterType::None,
                6,
                |_, _, _| true
            )
            .is_empty()
        );
        let inverted = Site {
            min_x: 3,
            max_x: 1,
            ..site
        };
        assert!(
            crater_blocks(
                inverted,
                FalloutMode::Default,
                CraterType::Normal,
                6,
                |_, _, _| true
            )
            .is_empty()
        );
    }

    #[test]
    fn special_craters_fill_solid_blocks_below_the_center() {
        let site = column(10, 4.0, 0, 0);
        let solid = |_, y: i32, _| y == 9;
        for (kind, block) in [
            (CraterType::Lava, "minecraft:lava"),
            (CraterType::Obsidian, "minecraft:obsidian"),
            (CraterType::Water, "minecraft:water"),
            (CraterType::Snow, "minecraft:snow_block"),
            (CraterType::Ice, "minecraft:ice"),
        ] {
            let cells = crater_blocks(site, FalloutMode::Default, kind, 10, solid);
            assert_eq!(at(&cells, 0, 9, 0), Some(block));
            assert_eq!(at(&cells, 0, 10, 0), Some("minecraft:air"));
        }
        let airy = crater_blocks(
            site,
            FalloutMode::Default,
            CraterType::Lava,
            10,
            |_, _, _| false,
        );
        assert_eq!(at(&airy, 0, 9, 0), Some("minecraft:air"));
    }

    #[test]
    fn sand_fallout_raises_the_crater_floor() {
        let site = column(10, 4.0, 0, 0);
        let cells = crater_blocks(
            site,
            FalloutMode::Sand,
            CraterType::Normal,
            12,
            |_, _, _| false,
        );
        assert_eq!(at(&cells, 0, 8, 0), None);
        assert_eq!(at(&cells, 0, 10, 0), Some("minecraft:air"));
    }

    #[test]
    fn crater_lake_fills_air_below_sea_level() {
        let site = column(10, 4.0, 0, 0);
        let scan = scan_lake(site, FalloutMode::Default, 12, |_, _, _| true);
        let water = |y| {
            scan.water
                .iter()
                .find(|cell| cell.y == y)
                .map(|cell| (cell.block, cell.schedule_water_tick))
        };
        assert_eq!(water(11), Some(("minecraft:water", true)));
        assert_eq!(water(8), Some(("minecraft:water", false)));
        assert_eq!(water(7), None);
        assert!(scan.slopes.iter().any(|cell| cell.y == 7));
        let stone = scan_lake(site, FalloutMode::Default, 12, |_, y, _| y != 8);
        assert!(stone.water.iter().all(|cell| cell.y != 8));
        let raised = scan_lake(site, FalloutMode::Sand, 12, |_, _, _| true);
        assert!(raised.water.iter().all(|cell| cell.y != 8));
        let far = Site {
            max_x: 40,
            min_x: 40,
            ..site
        };
        let outside = scan_lake(far, FalloutMode::Default, 12, |_, _, _| {
            panic!("outside the bowl")
        });
        assert!(outside.water.is_empty());
        assert!(outside.slopes.is_empty());
        assert!(
            scan_lake(site, FalloutMode::Default, 4, |_, _, _| true)
                .water
                .is_empty()
        );
    }

    #[test]
    fn enclosing_column_uses_magma_then_fallout_and_stops_after_twenty() {
        assert_eq!(
            enclosing_block(
                CraterType::Lava,
                0.0,
                FalloutMode::Default,
                "minecraft:sand",
                0.95
            ),
            Some("minecraft:magma_block")
        );
        assert_eq!(
            enclosing_block(
                CraterType::Lava,
                0.075,
                FalloutMode::Default,
                "minecraft:sand",
                0.95
            ),
            Some("minecraft:stone")
        );
        assert_eq!(
            enclosing_block(
                CraterType::Normal,
                0.0,
                FalloutMode::Terracotta,
                "minecraft:terracotta",
                0.5
            ),
            None
        );
        assert_eq!(column_kind(true, false, false, false), ColumnKind::Open);
        assert_eq!(column_kind(false, false, true, true), ColumnKind::Stop);
        assert_eq!(column_kind(false, true, true, false), ColumnKind::Open);
        assert_eq!(column_kind(false, true, false, true), ColumnKind::Open);
        assert_eq!(column_kind(false, true, false, false), ColumnKind::Stop);
        let mut calls = 0;
        let empty = pillar_blocks(
            0,
            5,
            0,
            |_, _, _| {
                calls += 1;
                ColumnKind::Open
            },
            |_, _, _| None,
        );
        assert_eq!(calls, 20);
        assert!(empty.is_empty());
        let placed = pillar_blocks(
            0,
            4,
            0,
            |_, y, _| {
                if y == 2 {
                    ColumnKind::Stop
                } else {
                    ColumnKind::Open
                }
            },
            |_, _, _| Some("minecraft:gravel"),
        );
        assert_eq!(placed.len(), 2);
        assert_eq!(placed[0].y, 4);
        assert_eq!(placed[1].block, "minecraft:gravel");
    }

    #[test]
    fn decay_gates_match_the_crater_term() {
        let crater = crater_squared(4.0);
        assert_eq!(crater, 169.0);
        assert_eq!(fall_budget(4.0), 16900.0);
        assert!(fall_height(crater, 0.0, crater * 1.7) > 0.0);
        assert!(fall_height(crater, 0.0, 0.0) < 0.0);
        assert!(!inset_reaches(0, 0, 0, crater));
        assert!(!inset_reaches(1, -1, 1, crater));
        assert!(inset_reaches(2, 0, 0, crater));
        assert!(!inset_reaches(30, 0, 0, crater));
        assert!(decay_skips(true, "minecraft:lava", "minecraft:lava"));
        assert!(!decay_skips(true, "minecraft:stone", "minecraft:lava"));
        assert!(!decay_skips(false, "minecraft:lava", "minecraft:lava"));
    }
}
