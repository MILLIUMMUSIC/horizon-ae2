// SPDX-License-Identifier: LGPL-3.0-or-later
// CertusQuartzClusterBlock.getDrops and original bud/cluster loot tables.
// Copyright (c) AlgorithmX2 / AE2 contributors; 2026 Horizon port contributors.

pub const CRYSTAL: &str = "ae2:certus_quartz_crystal";
pub const DUST: &str = "ae2:certus_quartz_dust";
pub const CLUSTER_DROP: u32 = 4;

pub fn is_cluster(block: &str) -> bool {
    matches!(
        block,
        "ae2:quartz_cluster"
            | "ae2:small_quartz_bud"
            | "ae2:medium_quartz_bud"
            | "ae2:large_quartz_bud"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drop {
    pub item: &'static str,
    pub count: u32,
}

/// `CertusQuartzClusterBlock.getDrops` (AE2 19.2.18).
///
/// No entity (`LootContextParams.THIS_ENTITY` absent) drops nothing, so a
/// bud or cluster whose support breaks leaves no items. A player break uses
/// the block loot table: Silk Touch keeps the block. A cluster otherwise
/// drops four `ae2:certus_quartz_crystal` plus Fortune's
/// `uniform_bonus_count` roll (`nextInt(fortune_level + 1)`,
/// `bonusMultiplier` 1). `fortune_roll` is that `nextInt` result. Each
/// smaller bud otherwise drops one `ae2:certus_quartz_dust`.
pub fn drops(block: &str, has_entity: bool, silk_touch: bool, fortune_roll: u32) -> Vec<Drop> {
    if !has_entity {
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
    fn missing_entity_drops_nothing() {
        assert!(drops("ae2:quartz_cluster", false, false, 0).is_empty());
        assert!(drops("ae2:small_quartz_bud", false, true, 0).is_empty());
        assert!(drops("minecraft:stone", true, false, 0).is_empty());
    }
}
