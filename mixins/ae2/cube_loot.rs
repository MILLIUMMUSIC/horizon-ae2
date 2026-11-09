// SPDX-License-Identifier: LGPL-3.0-or-later AND MIT
// The source cube's TagEntry is one weighted choice emitting every member.
// Native block loot lacks TagEntry; retain its actual table, tool, conditions,
// modifiers and named random sequence. Never replace the tag with a fixed list.
use super::{ItemStack, LootContext};
use horizon_core::random::RandomSource;
use serde_json::Value;
use std::sync::Arc;

enum Output<'a> {
    Item(ItemStack, Value),
    Tag(&'a Value),
}
struct Choice<'a> {
    weight: i32,
    output: Output<'a>,
}
fn has_tag(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            map.get("type").and_then(Value::as_str) == Some("minecraft:tag")
                || map.values().any(has_tag)
        }
        Value::Array(values) => values.iter().any(has_tag),
        _ => false,
    }
}
fn members<'a>(ctx: &'a LootContext<'_>, entry: &'a Value) -> &'a [String] {
    entry["items"]
        .as_str()
        .and_then(|s| s.strip_prefix('#'))
        .and_then(|tag| ctx.items.item_tag_order.get(tag))
        .map_or(&[], Vec::as_slice)
}
fn stack(id: &str) -> ItemStack {
    ItemStack {
        item: Arc::from(id),
        count: 1,
        damage: 0,
        extra: None,
    }
}
fn expand<'v>(
    ctx: &LootContext<'_>,
    entry: &'v Value,
    random: &mut impl RandomSource,
    out: &mut Vec<Choice<'v>>,
) -> bool {
    let kind = entry["type"].as_str().unwrap_or("");
    if matches!(
        kind,
        "minecraft:alternatives" | "minecraft:group" | "minecraft:sequence" | "minecraft:tag"
    ) {
        if !ctx.conditions(entry, random) {
            return false;
        }
        let children = || entry["children"].as_array().into_iter().flatten();
        match kind {
            "minecraft:alternatives" => children().any(|c| expand(ctx, c, random, out)),
            "minecraft:group" => {
                for c in children() {
                    expand(ctx, c, random, out);
                }
                true
            }
            "minecraft:sequence" => {
                for c in children() {
                    if !expand(ctx, c, random, out) {
                        return false;
                    }
                }
                true
            }
            "minecraft:tag" => {
                let weight = entry["weight"].as_i64().unwrap_or(1) as i32;
                if entry["expand"].as_bool().unwrap_or(false) {
                    for id in members(ctx, entry) {
                        out.push(Choice {
                            weight,
                            output: Output::Item(stack(id), entry["modifier"].clone()),
                        });
                    }
                } else {
                    out.push(Choice {
                        weight,
                        output: Output::Tag(entry),
                    });
                }
                // Empty tags still expand successfully: alternatives stops.
                true
            }
            _ => unreachable!(),
        }
    } else {
        let mut native = Vec::new();
        let accepted = ctx.expand(entry, random, &mut native);
        out.extend(native.into_iter().map(|(weight, stack, modifier)| Choice {
            weight,
            output: Output::Item(stack, modifier),
        }));
        accepted
    }
}
fn choose(
    ctx: &LootContext<'_>,
    entries: &[Value],
    random: &mut impl RandomSource,
    produced: &mut Vec<ItemStack>,
) {
    let mut choices = Vec::new();
    for entry in entries {
        expand(ctx, entry, random, &mut choices);
    }
    choices.retain(|choice| choice.weight > 0);
    if choices.is_empty() {
        return;
    }
    let pick = if choices.len() == 1 {
        0
    } else {
        let total = choices
            .iter()
            .fold(0i32, |sum, c| sum.wrapping_add(c.weight));
        let mut index = random
            .next_int_bound(total)
            .expect("positive source pool weight");
        choices
            .iter()
            .position(|choice| {
                index -= choice.weight;
                index < 0
            })
            .expect("weighted choice")
    };
    let mut output = |mut stack, modifier: &Value| {
        ctx.modifiers(modifier, &mut stack, random);
        produced.push(stack);
    };
    match choices.swap_remove(pick).output {
        Output::Item(stack, modifier) => output(stack, &modifier),
        Output::Tag(entry) => {
            for id in members(ctx, entry) {
                output(stack(id), &entry["modifier"]);
            }
        }
    }
}
pub(super) fn evaluate(
    ctx: &LootContext<'_>,
    table: &Value,
    random: &mut impl RandomSource,
) -> Vec<ItemStack> {
    if ctx.block != "ae2:mysterious_cube" || !has_tag(table) {
        return ctx.evaluate(table, random);
    }
    // Native block loot has no luck parameter. The pinned cube has constant
    // weight 1/quality 0, bonus rolls 0 and no table modifier; those broader
    // loot-context contracts remain outside this narrow source-table binding.
    let mut out = Vec::new();
    for pool in table["pools"].as_array().into_iter().flatten() {
        if !ctx.conditions(pool, random) {
            continue;
        }
        let rolls = LootContext::int_number(&pool["rolls"], random).max(0) as u32;
        for _ in 0..rolls {
            let mut produced = Vec::new();
            let entries = pool["entries"].as_array().map_or(&[][..], Vec::as_slice);
            if has_tag(&pool["entries"]) {
                choose(ctx, entries, random, &mut produced);
            } else {
                ctx.choose(entries, random, &mut produced);
            }
            for mut item in produced {
                ctx.modifiers(&pool["modifier"], &mut item, random);
                ctx.modifiers(&table["modifier"], &mut item, random);
                if item.count > 0 {
                    out.push(item);
                }
            }
        }
    }
    out
}

// The game's player/Level callers select their actual named sequence before
// these hooks. The received RNG is used directly; no table/level RNG swap.
pub(super) fn drops(
    items: &super::Items,
    registry: &horizon_world::registry::BlockRegistry,
    state: horizon_world::registry::BlockStateId,
    held: Option<&ItemStack>,
    dynamic: &[ItemStack],
    random: &mut impl RandomSource,
    breaker: bool,
) -> Option<Vec<ItemStack>> {
    let block = registry.state(state)?;
    if block.name != "ae2:mysterious_cube" {
        return None;
    }
    let table = items.loot.get("ae2:blocks/mysterious_cube")?;
    if !has_tag(table) {
        return None;
    }
    let ctx = LootContext {
        items,
        block: &block.name,
        properties: &block.properties,
        held,
        killed_by_player: false,
        explosion_radius: None,
        this_entity: breaker,
    };
    Some(
        evaluate(&ctx, table, random)
            .into_iter()
            .flat_map(|item| {
                if item.item.starts_with(super::DYNAMIC) {
                    dynamic.to_vec()
                } else {
                    vec![item]
                }
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::random::LegacyRandom;
    use serde_json::json;
    use std::collections::{BTreeMap, HashMap};
    #[test]
    fn ae2_cube_tag_is_one_choice_emits_ordered_members_and_keeps_rng() {
        let mut items = super::super::Items::for_tests(HashMap::new());
        items.item_tag_order.insert(
            "ae2:test".into(),
            vec!["test:second".into(), "test:first".into()],
        );
        let props = BTreeMap::new();
        let ctx = LootContext {
            items: &items,
            block: "ae2:mysterious_cube",
            properties: &props,
            held: None,
            killed_by_player: false,
            explosion_radius: None,
            this_entity: true,
        };
        let table = json!({"pools":[{"rolls":1,"entries":[{"type":"minecraft:tag","items":"#ae2:test","expand":false,"modifier":{"type":"minecraft:set_count","count":2}}]}]});
        let mut random = LegacyRandom::new(47);
        let mut reference = LegacyRandom::new(47);
        let result = evaluate(&ctx, &table, &mut random);
        assert_eq!(
            result
                .iter()
                .map(|s| (&*s.item, s.count))
                .collect::<Vec<_>>(),
            vec![("test:second", 2), ("test:first", 2)]
        );
        assert_eq!(random.next_long(), reference.next_long());
    }
    #[test]
    fn ae2_empty_tag_stops_alternatives_and_other_blocks_keep_native_evaluator() {
        let items = super::super::Items::for_tests(HashMap::new());
        let props = BTreeMap::new();
        let mut ctx = LootContext {
            items: &items,
            block: "ae2:mysterious_cube",
            properties: &props,
            held: None,
            killed_by_player: false,
            explosion_radius: None,
            this_entity: true,
        };
        for expanded in [false, true] {
            let table = json!({"pools":[{"rolls":1,"entries":[{"type":"minecraft:alternatives","children":[{"type":"minecraft:tag","items":"#ae2:empty","expand":expanded},{"type":"minecraft:item","name":"test:fallback"}]}]}]});
            assert!(evaluate(&ctx, &table, &mut LegacyRandom::new(1)).is_empty());
            ctx.block = "minecraft:stone";
            // The original native evaluator does not support tags and reaches
            // the fallback. The mod must not alter that other block's result.
            assert_eq!(
                &*evaluate(&ctx, &table, &mut LegacyRandom::new(1))[0].item,
                "test:fallback"
            );
            ctx.block = "ae2:mysterious_cube";
        }
    }
}
