// SPDX-License-Identifier: LGPL-3.0-or-later
// The native evaluator must retain the actual tool and named RNG stream.
#[cfg(test)]
mod tests {
    use crate::items::{ItemStack, Items};
    use crate::random_sequences::RandomSequences;
    use horizon_core::random::{LegacyRandom, RandomSource};
    use horizon_world::registry::BlockRegistry;
    use serde_json::json;

    fn fixture() -> (Items, BlockRegistry) {
        let registry = BlockRegistry::from_facts_json(
            json!({"states": [
                ["", false, false, "", "ae2:quartz_cluster"],
                ["", false, false, "", "minecraft:stone"]
            ], "defaults": {"ae2:quartz_cluster": 0, "minecraft:stone": 1}})
            .to_string()
            .as_bytes(),
        )
        .unwrap();
        let mut items = Items::for_tests(Default::default());
        // A native-format fixture of AE2's alternatives/set_count/apply_bonus
        // table. The world scenario separately uses the actual carried jar data.
        let table = json!({"random_sequence":"ae2:blocks/quartz_cluster", "pools":[{
            "rolls":1, "entries":[{"type":"minecraft:alternatives", "children":[
                {"type":"minecraft:item", "condition":"minecraft:tool/can_silk_touch",
                 "name":"ae2:quartz_cluster"},
                {"type":"minecraft:item", "name":"ae2:certus_quartz_crystal", "modifier":[
                    {"type":"minecraft:set_count", "count":4},
                    {"type":"minecraft:apply_bonus", "enchantment":"minecraft:fortune",
                     "formula":"minecraft:uniform_bonus_count", "parameters":{"bonusMultiplier":1}}
                ]}
            ]}]
        }]});
        std::sync::Arc::get_mut(&mut items.loot)
            .unwrap()
            .insert("ae2:blocks/quartz_cluster".into(), table.clone());
        std::sync::Arc::get_mut(&mut items.loot)
            .unwrap()
            .insert("minecraft:blocks/stone".into(), table);
        (items, registry)
    }

    #[test]
    fn missing_entity_refuses_before_rng_or_sequence_creation_and_keeps_vanilla() {
        let (items, registry) = fixture();
        let mut random = LegacyRandom::new(91);
        let mut reference = random.clone();
        let mut sequences = RandomSequences::default();
        let before = sequences.to_nbt();
        let cluster = registry.default_state("ae2:quartz_cluster").unwrap();
        let drops = items.block_drops_in(
            &registry,
            cluster,
            None,
            &mut random,
            Some((&mut sequences, 42)),
        );
        assert!(drops.is_empty());
        assert_eq!(sequences.to_nbt(), before);
        assert_eq!(random.next_long(), reference.next_long());
        let stone = registry.default_state("minecraft:stone").unwrap();
        assert_eq!(
            items.block_drops(&registry, stone, None, &mut random)[0].count,
            4
        );
    }

    #[test]
    fn supplied_tool_and_zero_fortune_keep_the_original_sequence_draw() {
        let (items, registry) = fixture();
        let cluster = registry.default_state("ae2:quartz_cluster").unwrap();
        let mut plain = ItemStack::new("minecraft:diamond_pickaxe", 1);
        let mut silk = plain.clone();
        silk.enchant("minecraft:silk_touch", 1);
        for fortune in [0, 3] {
            if fortune > 0 {
                plain.enchant("minecraft:fortune", fortune);
            }
            let mut sequences = RandomSequences::default();
            let mut reference = RandomSequences::default();
            let mut level_rng = LegacyRandom::new(91);
            let mut untouched = level_rng.clone();
            let expected = reference
                .get("ae2:blocks/quartz_cluster", 42)
                .next_int_bound(fortune as i32 + 1)
                .unwrap() as u32;
            let drops = items.block_drops_exploded(
                &registry,
                cluster,
                Some(&plain),
                &[],
                &mut level_rng,
                Some((&mut sequences, 42)),
                None,
                true,
            );
            assert_eq!(drops.len(), 1);
            assert_eq!(&*drops[0].item, "ae2:certus_quartz_crystal");
            assert_eq!(drops[0].count, 4 + expected);
            assert_eq!(sequences.to_nbt(), reference.to_nbt());
            assert_eq!(level_rng.next_long(), untouched.next_long());
            let before = sequences.to_nbt();
            let drops = items.block_drops_exploded(
                &registry,
                cluster,
                Some(&silk),
                &[],
                &mut level_rng,
                Some((&mut sequences, 42)),
                None,
                true,
            );
            assert_eq!(&*drops[0].item, "ae2:quartz_cluster");
            assert_eq!(drops[0].count, 1);
            assert_eq!(sequences.to_nbt(), before);
        }
    }
}
