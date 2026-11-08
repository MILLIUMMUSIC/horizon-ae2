// SPDX-License-Identifier: LGPL-3.0-or-later
// BuddingCertusQuartzBlock.randomTick (AE2 19.2.18).
// Copyright (c) AlgorithmX2 / AE2 contributors; 2026 Horizon port contributors.

pub const GROWTH_CHANCE: i32 = 5;
pub const DECAY_CHANCE: i32 = 12;
pub const CRYSTAL: &str = "ae2:certus_quartz_crystal";
pub const DUST: &str = "ae2:certus_quartz_dust";
/// `quartz_cluster.json` `set_count` before Fortune.
pub const CLUSTER_DROP: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Host {
    Flawless,
    Flawed,
    Chipped,
    Damaged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Empty,
    Small,
    Medium,
    Large,
    Occupied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Growth {
    pub cluster: Option<&'static str>,
    pub host: Option<&'static str>,
}

pub fn host(block: &str) -> Option<Host> {
    match block {
        "ae2:flawless_budding_quartz" => Some(Host::Flawless),
        "ae2:flawed_budding_quartz" => Some(Host::Flawed),
        "ae2:chipped_budding_quartz" => Some(Host::Chipped),
        "ae2:damaged_budding_quartz" => Some(Host::Damaged),
        _ => None,
    }
}

pub fn target(block: &str, facing: Option<&str>, direction: &str, water_source: bool) -> Target {
    if block == "minecraft:air" || (block == "minecraft:water" && water_source) {
        return Target::Empty;
    }
    let stage = match block {
        "ae2:small_quartz_bud" => Target::Small,
        "ae2:medium_quartz_bud" => Target::Medium,
        "ae2:large_quartz_bud" => Target::Large,
        _ => return Target::Occupied,
    };
    if facing == Some(direction) {
        stage
    } else {
        Target::Occupied
    }
}

/// One successful growth roll. `decay_zero` is `nextInt(DECAY_CHANCE) == 0`.
pub fn grow(host: Host, target: Target, decay_zero: bool) -> Growth {
    let cluster = match target {
        Target::Empty => Some("ae2:small_quartz_bud"),
        Target::Small => Some("ae2:medium_quartz_bud"),
        Target::Medium => Some("ae2:large_quartz_bud"),
        Target::Large => Some("ae2:quartz_cluster"),
        Target::Occupied => None,
    };
    let host = match (cluster, host, decay_zero) {
        (Some(_), Host::Flawless, _) | (None, _, _) | (Some(_), _, false) => None,
        (Some(_), Host::Flawed, true) => Some("ae2:chipped_budding_quartz"),
        (Some(_), Host::Chipped, true) => Some("ae2:damaged_budding_quartz"),
        (Some(_), Host::Damaged, true) => Some("ae2:quartz_block"),
    };
    Growth { cluster, host }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drop {
    pub item: &'static str,
    pub count: u32,
}

/// `CertusQuartzClusterBlock.getDrops` (AE2 19.2.18).
///
/// No player (`LootContextParams.THIS_ENTITY` absent) drops nothing, so a
/// bud or cluster whose support breaks leaves no items. A player break uses
/// the block loot table: Silk Touch keeps the block. A cluster otherwise
/// drops four `ae2:certus_quartz_crystal` plus Fortune's
/// `uniform_bonus_count` roll (`nextInt(fortune_level + 1)`,
/// `bonusMultiplier` 1). `fortune_roll` is that `nextInt` result. Each
/// smaller bud otherwise drops one `ae2:certus_quartz_dust`.
pub fn drops(block: &str, player: bool, silk_touch: bool, fortune_roll: u32) -> Vec<Drop> {
    if !player {
        return Vec::new();
    }
    let keep = |item| Drop { item, count: 1 };
    let one = match block {
        "ae2:quartz_cluster" if silk_touch => Some(keep("ae2:quartz_cluster")),
        "ae2:quartz_cluster" => Some(Drop {
            item: CRYSTAL,
            count: CLUSTER_DROP + fortune_roll,
        }),
        "ae2:small_quartz_bud" if silk_touch => Some(keep("ae2:small_quartz_bud")),
        "ae2:medium_quartz_bud" if silk_touch => Some(keep("ae2:medium_quartz_bud")),
        "ae2:large_quartz_bud" if silk_touch => Some(keep("ae2:large_quartz_bud")),
        "ae2:small_quartz_bud" | "ae2:medium_quartz_bud" | "ae2:large_quartz_bud" => Some(Drop {
            item: DUST,
            count: 1,
        }),
        _ => None,
    };
    one.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flawless_host_grows_a_cluster_and_does_not_decay() {
        let mut target = Target::Empty;
        for expected in [
            "ae2:small_quartz_bud",
            "ae2:medium_quartz_bud",
            "ae2:large_quartz_bud",
            "ae2:quartz_cluster",
        ] {
            let step = grow(Host::Flawless, target, true);
            assert_eq!(step.cluster, Some(expected));
            assert_eq!(step.host, None);
            target = match expected {
                "ae2:small_quartz_bud" => Target::Small,
                "ae2:medium_quartz_bud" => Target::Medium,
                "ae2:large_quartz_bud" => Target::Large,
                _ => Target::Occupied,
            };
        }
    }

    #[test]
    fn damaged_host_decays_to_quartz_block_only_on_a_zero_roll() {
        let grew = grow(Host::Damaged, Target::Empty, true);
        assert_eq!(grew.cluster, Some("ae2:small_quartz_bud"));
        assert_eq!(grew.host, Some("ae2:quartz_block"));
        let stayed = grow(Host::Damaged, Target::Empty, false);
        assert_eq!(stayed.host, None);
        assert_eq!(grow(Host::Flawless, Target::Occupied, true).cluster, None);
    }

    #[test]
    fn cluster_drops_four_certus_crystals() {
        assert_eq!(
            drops("ae2:quartz_cluster", true, false, 0),
            vec![Drop {
                item: CRYSTAL,
                count: 4,
            }]
        );
    }

    #[test]
    fn silk_touch_keeps_the_cluster() {
        assert_eq!(
            drops("ae2:quartz_cluster", true, true, 3),
            vec![Drop {
                item: "ae2:quartz_cluster",
                count: 1,
            }]
        );
    }

    #[test]
    fn buds_drop_one_certus_dust_and_silk_touch_keeps_the_bud() {
        for block in [
            "ae2:small_quartz_bud",
            "ae2:medium_quartz_bud",
            "ae2:large_quartz_bud",
        ] {
            assert_eq!(
                drops(block, true, false, 2),
                vec![Drop {
                    item: DUST,
                    count: 1,
                }]
            );
            assert_eq!(
                drops(block, true, true, 0),
                vec![Drop {
                    item: block,
                    count: 1,
                }]
            );
        }
    }

    #[test]
    fn fortune_uniform_bonus_adds_the_rolled_extra_crystals() {
        // nextInt(fortuneLevel + 1) with bonusMultiplier 1.
        assert_eq!(drops("ae2:quartz_cluster", true, false, 2)[0].count, 6);
    }

    #[test]
    fn no_player_drops_nothing() {
        assert!(drops("ae2:quartz_cluster", false, false, 0).is_empty());
        assert!(drops("ae2:small_quartz_bud", false, true, 0).is_empty());
        assert!(drops("minecraft:stone", true, false, 0).is_empty());
    }
}
