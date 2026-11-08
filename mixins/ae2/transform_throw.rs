// SPDX-License-Identifier: LGPL-3.0-or-later
// In-world fluix throw: TransformLogic.tryTransform for
// ae2:transform/fluix_crystals. ItemEntityMixin calls that only after the
// trigger has stayed in a matching fluid for more than 60 ticks.
// A missing circumstance field is fluid tag minecraft:water
// (TransformRecipe codec default). Matching uses that tag on the fluid at
// the trigger's bounding-box center, not a block-id comparison.
// Copyright (c) AlgorithmX2 / AE2 contributors; 2026 Horizon port contributors.

/// Server `ItemEntityMixin` calls the throw only after `ae2_transformTime > 60`.
pub const FLUID_TICKS_BEFORE_TRY: i32 = 60;

/// The mixin's transient Java int is attached to the entity, never its stack.
/// Ineligible stacks leave it unchanged; only a failed attempt resets it.
pub fn fluid_tick(
    counter: &mut i32,
    removed: bool,
    eligible: bool,
    valid_fluid: bool,
    attempt: impl FnOnce() -> bool,
) {
    if removed || !eligible {
        return;
    }
    if valid_fluid {
        *counter = counter.wrapping_add(1);
        if *counter > FLUID_TICKS_BEFORE_TRY && !attempt() {
            *counter = 0;
        }
    } else {
        *counter = 0;
    }
}

/// Source ordered greedy matching. Rows are ingredients, columns are entities
/// in Level.getEntities order. A repeated ingredient may claim the same entity
/// several times, but cannot spend more than its current count. No mutation
/// occurs until the entire recipe has matched.
pub fn claims(counts: &[u32], matches: &[Vec<bool>], trigger: usize) -> Option<Vec<u32>> {
    if matches.is_empty()
        || counts.get(trigger).is_none_or(|n| *n == 0)
        || !matches[0].get(trigger).copied().unwrap_or(false)
        || matches.iter().any(|row| row.len() != counts.len())
    {
        return None;
    }
    let mut missing = vec![true; matches.len()];
    let mut claimed = vec![0; counts.len()];
    for (entity, count) in counts.iter().enumerate() {
        for (ingredient, row) in matches.iter().enumerate() {
            if missing[ingredient] && row[entity] && *count > claimed[entity] {
                claimed[entity] += 1;
                missing[ingredient] = false;
            }
        }
    }
    (!missing.contains(&true)).then_some(claimed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_threshold_failure_reset_success_retention_and_java_wrap() {
        let mut counter = 59;
        fluid_tick(&mut counter, false, true, true, || panic!("early attempt"));
        assert_eq!(counter, 60);
        fluid_tick(&mut counter, false, true, true, || true);
        assert_eq!(counter, 61);
        fluid_tick(&mut counter, false, true, true, || false);
        assert_eq!(counter, 0);
        counter = i32::MAX;
        fluid_tick(&mut counter, false, true, true, || {
            panic!("overflow attempt")
        });
        assert_eq!(counter, i32::MIN);
    }
    #[test]
    fn removed_or_ineligible_preserve_counter_while_invalid_fluid_resets() {
        let mut counter = 61;
        fluid_tick(&mut counter, true, true, true, || panic!("removed entity"));
        fluid_tick(&mut counter, false, false, false, || {
            panic!("ineligible entity")
        });
        assert_eq!(counter, 61);
        fluid_tick(&mut counter, false, true, false, || panic!("dry entity"));
        assert_eq!(counter, 0);
    }
    #[test]
    fn repeated_ingredients_respect_greedy_entity_order_and_available_count() {
        let matches = vec![
            vec![true, true, false],
            vec![true, true, false],
            vec![false, false, true],
        ];
        assert_eq!(claims(&[1, 1, 1], &matches, 0), Some(vec![1, 1, 1]));
        assert_eq!(claims(&[2, 1, 1], &matches, 1), Some(vec![2, 0, 1]));
        assert_eq!(claims(&[1, 0, 1], &matches, 0), None);
        assert_eq!(claims(&[2, 1, 1], &matches, 2), None);
        assert_eq!(claims(&[2, 1, 1], &[], 0), None);
    }
}
