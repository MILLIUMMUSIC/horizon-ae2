// SPDX-License-Identifier: MIT
// StorageHelper.poweredExtraction / poweredInsert from AE2 19.2.18.
// Copyright (c) 2013 AlgorithmX2; see LICENSES/AE2-API-MIT.
// The binding owns the actual key/action source, CONFIG power multiplier and
// player statistics. No adapter may replace a full key with its registry ID.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Simulate,
    Modulate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Statistic {
    Extracted,
    Inserted,
}
/// Callbacks use signed Java longs and preserve one action source. A native
/// binding must apply PowerMultiplier.CONFIG on both power calls and record a
/// statistic only for a source with an actual player. Partial/racing callbacks
/// must return what really happened, without refunding or inventing transfers.
pub trait Context {
    type Key;
    fn extract(&mut self, key: &Self::Key, amount: i64, action: Action) -> i64;
    fn insert(&mut self, key: &Self::Key, amount: i64, action: Action) -> i64;
    fn power(&mut self, amount: f64, action: Action) -> f64;
    fn amount_per_operation(&self, key: &Self::Key) -> i32;
    fn is_item(&self, key: &Self::Key) -> bool;
    fn statistic(&mut self, statistic: Statistic, amount: i32);
}

pub fn powered_extraction<C: Context>(
    context: &mut C,
    key: &C::Key,
    amount: i64,
    mode: Action,
) -> i64 {
    let retrieved = context.extract(key, amount, Action::Simulate);
    let factor = f64::from(context.amount_per_operation(key)).max(1.0);
    let available_power = context.power(retrieved as f64 / factor, Action::Simulate);
    // Rust's saturating float-to-long cast matches Java d2l, including NaN,
    // infinities and truncation. Do not replace the original +0.9 tolerance.
    let to_extract = ((available_power * factor + 0.9) as i64).min(retrieved);
    if to_extract > 0 {
        if mode == Action::Modulate {
            // Source deliberately requests power for the simulated retrieved
            // amount, rather than the smaller power-limited transfer.
            context.power(retrieved as f64 / factor, Action::Modulate);
            let extracted = context.extract(key, to_extract, Action::Modulate);
            if extracted != 0 && context.is_item(key) {
                context.statistic(Statistic::Extracted, saturated_int(extracted));
            }
            return extracted;
        }
        return to_extract;
    }
    0
}

pub fn powered_insert<C: Context>(context: &mut C, key: &C::Key, amount: i64, mode: Action) -> i64 {
    let amount = context.insert(key, amount, Action::Simulate);
    if amount <= 0 {
        return 0;
    }
    let factor = f64::from(context.amount_per_operation(key)).max(1.0);
    let available_power = context.power(amount as f64 / factor, Action::Simulate);
    let amount = ((available_power * factor + 0.9) as i64).min(amount);
    if amount <= 0 {
        return 0;
    }
    if mode == Action::Modulate {
        // Insertion, unlike extraction, charges the reduced amount.
        context.power(amount as f64 / factor, Action::Modulate);
        let inserted = context.insert(key, amount, Action::Modulate);
        if context.is_item(key) {
            context.statistic(Statistic::Inserted, saturated_int(inserted));
        }
        return inserted;
    }
    amount
}
fn saturated_int(amount: i64) -> i32 {
    amount.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Calls {
        events: Vec<String>,
        simulated: i64,
        committed: i64,
        energy: f64,
    }
    impl Context for Calls {
        type Key = i32;
        fn extract(&mut self, _: &i32, amount: i64, mode: Action) -> i64 {
            self.events.push(format!("extract:{amount}:{mode:?}"));
            if mode == Action::Simulate {
                self.simulated
            } else {
                self.committed
            }
        }
        fn insert(&mut self, _: &i32, amount: i64, mode: Action) -> i64 {
            self.events.push(format!("insert:{amount}:{mode:?}"));
            if mode == Action::Simulate {
                self.simulated
            } else {
                self.committed
            }
        }
        fn power(&mut self, amount: f64, mode: Action) -> f64 {
            self.events.push(format!("power:{amount}:{mode:?}"));
            self.energy
        }
        fn amount_per_operation(&self, key: &i32) -> i32 {
            *key
        }
        fn is_item(&self, _: &i32) -> bool {
            true
        }
        fn statistic(&mut self, statistic: Statistic, amount: i32) {
            self.events.push(format!("stat:{statistic:?}:{amount}"));
        }
    }
    #[test]
    fn extraction_charges_original_simulation_while_insert_charges_limited_amount() {
        let mut calls = Calls {
            events: vec![],
            simulated: 100,
            committed: 2,
            energy: 2.1,
        };
        assert_eq!(powered_extraction(&mut calls, &1, 500, Action::Modulate), 2);
        assert_eq!(
            calls.events,
            [
                "extract:500:Simulate",
                "power:100:Simulate",
                "power:100:Modulate",
                "extract:3:Modulate",
                "stat:Extracted:2"
            ]
        );
        calls.events.clear();
        assert_eq!(powered_insert(&mut calls, &1, 500, Action::Modulate), 2);
        assert_eq!(
            calls.events,
            [
                "insert:500:Simulate",
                "power:100:Simulate",
                "power:3:Modulate",
                "insert:3:Modulate",
                "stat:Inserted:2"
            ]
        );
    }
    #[test]
    fn zero_and_nan_callbacks_keep_source_early_returns_and_simulation_has_no_commit() {
        let mut calls = Calls {
            events: vec![],
            simulated: 0,
            committed: 7,
            energy: f64::NAN,
        };
        assert_eq!(powered_insert(&mut calls, &125, 40, Action::Modulate), 0);
        assert_eq!(calls.events, ["insert:40:Simulate"]);
        calls.events.clear();
        assert_eq!(
            powered_extraction(&mut calls, &125, 40, Action::Modulate),
            0
        );
        assert_eq!(calls.events, ["extract:40:Simulate", "power:0:Simulate"]);
        calls.events.clear();
        calls.simulated = 40;
        calls.energy = 1.0;
        assert_eq!(
            powered_extraction(&mut calls, &125, 100, Action::Simulate),
            40
        );
        assert_eq!(calls.events.len(), 2);
    }
}
