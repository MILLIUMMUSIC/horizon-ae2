// SPDX-License-Identifier: LGPL-3.0-or-later
// In-world fluix throw: TransformLogic.tryTransform for
// ae2:transform/fluix_crystals. ItemEntityMixin calls that only after the
// trigger has stayed in a matching fluid for more than 60 ticks.
// A missing circumstance field is fluid tag minecraft:water
// (TransformRecipe codec default). Matching uses that tag on the fluid at
// the trigger's bounding-box center, not a block-id comparison.
// Copyright (c) AlgorithmX2 / AE2 contributors; 2026 Horizon port contributors.

/// `FluidTags.WATER`. Codec default when `circumstance` is absent.
pub const WATER_TAG: &str = "minecraft:water";

pub const CHARGED_CERTUS: &str = "ae2:charged_certus_quartz_crystal";
pub const REDSTONE: &str = "minecraft:redstone";
pub const NETHER_QUARTZ: &str = "minecraft:quartz";
pub const FLUIX_CRYSTAL: &str = "ae2:fluix_crystal";

/// Server `ItemEntityMixin` calls the throw only after `ae2_transformTime > 60`.
pub const FLUID_TICKS_BEFORE_TRY: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Circumstance {
    /// Field omitted. Resolves to [`WATER_TAG`].
    Missing,
    Fluid(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub id: &'static str,
    pub ingredients: &'static [&'static str],
    pub result: &'static str,
    pub count: u32,
    pub circumstance: Circumstance,
}

/// Charged certus, redstone, and nether quartz. Circumstance is omitted, so
/// the fluid tag is minecraft:water. The result is two fluix crystals.
pub const FLUIX_CRYSTALS: Recipe = Recipe {
    id: "ae2:transform/fluix_crystals",
    ingredients: &[CHARGED_CERTUS, REDSTONE, NETHER_QUARTZ],
    result: FLUIX_CRYSTAL,
    count: 2,
    circumstance: Circumstance::Missing,
};

pub fn circumstance_tag(circumstance: Circumstance) -> &'static str {
    match circumstance {
        Circumstance::Missing => WATER_TAG,
        Circumstance::Fluid(tag) => tag,
    }
}

/// One item entity already inside the trigger's ±1 block region.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThrownItem<'a> {
    pub item: &'a str,
    pub count: u32,
    pub alive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Product {
    pub item: &'static str,
    pub count: u32,
}

/// Consume one of each fluix ingredient when `trigger` is the charged certus
/// entity and `fluid_tags` contains the recipe fluid tag. An empty tag list
/// is an empty fluid state. A successful throw discards entities left empty.
pub fn try_fluix_throw(
    items: &mut [ThrownItem<'_>],
    trigger: usize,
    fluid_tags: &[&str],
) -> Option<Product> {
    let recipe = &FLUIX_CRYSTALS;
    let tag = circumstance_tag(recipe.circumstance);
    if !fluid_tags.contains(&tag) {
        return None;
    }
    let trigger_item = items.get(trigger)?;
    if !trigger_item.alive || trigger_item.count == 0 || trigger_item.item != recipe.ingredients[0]
    {
        return None;
    }

    let mut missing = recipe.ingredients.to_vec();
    let mut claimed = vec![0u32; items.len()];
    for (index, stack) in items.iter().enumerate() {
        if !stack.alive || stack.count == 0 {
            continue;
        }
        let mut cursor = 0;
        while cursor < missing.len() {
            if stack.count > claimed[index] && missing[cursor] == stack.item {
                claimed[index] += 1;
                missing.remove(cursor);
            } else {
                cursor += 1;
            }
        }
    }
    if !missing.is_empty() {
        return None;
    }

    for (stack, taken) in items.iter_mut().zip(claimed) {
        if taken == 0 {
            continue;
        }
        stack.count -= taken;
        if stack.count == 0 {
            stack.alive = false;
        }
    }
    Some(Product {
        item: recipe.result,
        count: recipe.count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thrown(item: &'static str, count: u32) -> ThrownItem<'static> {
        ThrownItem {
            item,
            count,
            alive: true,
        }
    }

    fn fluix_inputs(charged: u32) -> [ThrownItem<'static>; 3] {
        [
            thrown(CHARGED_CERTUS, charged),
            thrown(REDSTONE, 1),
            thrown(NETHER_QUARTZ, 1),
        ]
    }

    #[test]
    fn water_consumes_one_of_each_and_yields_two_fluix() {
        assert_eq!(FLUIX_CRYSTALS.circumstance, Circumstance::Missing);
        assert_eq!(circumstance_tag(Circumstance::Missing), WATER_TAG);
        let mut items = [
            thrown(CHARGED_CERTUS, 2),
            thrown(REDSTONE, 1),
            thrown(NETHER_QUARTZ, 1),
            thrown("minecraft:cobblestone", 4),
        ];
        let product = try_fluix_throw(&mut items, 0, &[WATER_TAG]).expect("water match");
        assert_eq!(
            product,
            Product {
                item: FLUIX_CRYSTAL,
                count: 2,
            }
        );
        assert_eq!(items[0].count, 1);
        assert!(items[0].alive);
        assert_eq!(
            items[1],
            ThrownItem {
                item: REDSTONE,
                count: 0,
                alive: false,
            }
        );
        assert_eq!(items[2].count, 0);
        assert!(!items[2].alive);
        assert_eq!(items[3].count, 4);
        assert!(items[3].alive);

        let mut later = fluix_inputs(1);
        assert!(try_fluix_throw(&mut later, 1, &[WATER_TAG]).is_none());
        assert!(later.iter().all(|item| item.alive && item.count == 1));
    }

    #[test]
    fn out_of_water_rejects_without_consuming() {
        let original = fluix_inputs(1);
        for fluid in [&[][..], &["minecraft:lava"][..]] {
            let mut items = original.clone();
            assert!(try_fluix_throw(&mut items, 0, fluid).is_none());
            assert_eq!(items, original);
        }
    }
}
