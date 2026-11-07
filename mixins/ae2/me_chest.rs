// SPDX-License-Identifier: LGPL-3.0-or-later
// MEChestBlockEntity local cell, input slot and powered insert/extract.
// Copyright AlgorithmX2 / TeamAppliedEnergistics; 2026 Horizon port contributors.
// Grid nodes, the terminal menu, fluids and lights stay outside this rules module.

use super::ae2_backend::CellBackend;
use super::ae2_energy::StoredEnergy;
use super::ae2_keys::KeyType;
use super::ae2_storage::CellKey;

pub const BLOCK: &str = "ae2:chest";
/// Combined inventory: input, cell, then one withdraw slot. The menu pads to nine.
pub const SLOTS: usize = 9;
pub const INPUT: usize = 0;
pub const CELL_SLOT: usize = 1;
pub const WITHDRAW: usize = 2;
/// `setInternalMaxPower(PowerMultiplier.CONFIG.multiply(500))` at the default multiplier.
pub const LOCAL_CAPACITY: f64 = 500.0;
/// Chest idle before the installed cell's own drain. Source sets this in `updateHandler`.
pub const BASE_IDLE: f64 = 1.0;

/// One ME Chest. The cell is the only stored inventory. Power is the local buffer
/// (`internalCurrentPower`); a grid node is not attached here.
#[derive(Clone, Debug)]
pub struct Chest<K: CellKey> {
    cell: Option<CellBackend<K>>,
    input: Option<(K, u64)>,
    energy: StoredEnergy,
    idle: f64,
    priority: i32,
}

#[derive(Clone, Debug)]
pub struct Saved<K: CellKey> {
    pub cell: Option<CellBackend<K>>,
    pub input: Option<(K, u64)>,
    pub power: f64,
    pub idle: f64,
    pub priority: i32,
}

impl<K: CellKey> Default for Chest<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: CellKey> Chest<K> {
    pub fn new() -> Self {
        Self {
            cell: None,
            input: None,
            energy: StoredEnergy::new(0.0, LOCAL_CAPACITY),
            idle: 0.0,
            priority: 0,
        }
    }

    pub fn power(&self) -> f64 {
        self.energy.amount()
    }

    pub fn set_power(&mut self, power: f64) {
        let power = if power.is_finite() {
            power.max(0.0)
        } else {
            0.0
        };
        self.energy.set_stored(power.min(LOCAL_CAPACITY));
    }

    pub fn idle(&self) -> f64 {
        self.idle
    }

    pub fn set_idle(&mut self, idle: f64) {
        self.idle = if idle.is_finite() { idle.max(0.0) } else { 0.0 };
    }

    pub fn priority(&self) -> i32 {
        self.priority
    }

    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }

    pub fn set_cell(&mut self, cell: Option<CellBackend<K>>) {
        self.cell = cell;
    }

    pub fn cell(&self) -> Option<&CellBackend<K>> {
        self.cell.as_ref()
    }

    pub fn cell_mut(&mut self) -> Option<&mut CellBackend<K>> {
        self.cell.as_mut()
    }

    pub fn set_input(&mut self, key: K, amount: u64) {
        self.input = (amount > 0).then_some((key, amount));
    }

    pub fn input(&self) -> Option<&(K, u64)> {
        self.input.as_ref()
    }

    pub fn clear_input(&mut self) {
        self.input = None;
    }

    /// `MEChestBlockEntity.isPowered` local half: `getAECurrentPower() > 1`.
    /// The grid-node half is not available without a registered chest node.
    pub fn is_powered(&self) -> bool {
        self.energy.amount() > 1.0
    }

    /// `updateHandler`: a readable cell sets `idlePowerUsage = 1 + drain`.
    /// Removing the cell does not clear that field.
    pub fn update_handler(&mut self) {
        if let Some(cell) = &self.cell {
            self.idle = BASE_IDLE + cell.idle_drain();
        }
    }

    pub fn stored(&self, key: &K) -> u64 {
        self.cell
            .as_ref()
            .and_then(|cell| cell.stacks().get(key).copied())
            .unwrap_or(0)
    }

    pub fn first_item(&self) -> Option<K> {
        self.cell.as_ref().and_then(|cell| {
            cell.stacks().iter().find_map(|(key, amount)| {
                (*amount > 0 && key.key_type() == KeyType::Item).then(|| key.clone())
            })
        })
    }

    /// `InputInventoryFilter.allowInsert`: powered, readable cell, simulate insert > 0.
    pub fn allow_input(&mut self, key: &K, amount: u64) -> bool {
        if !self.is_powered() || amount == 0 {
            return false;
        }
        self.update_handler();
        self.cell
            .as_mut()
            .is_some_and(|cell| cell.insert(key.clone(), amount, false) > 0)
    }

    /// `StorageHelper.poweredInsert` against the local buffer. Multiplier is the default 1.
    pub fn powered_insert(&mut self, key: &K, amount: u64, commit: bool) -> u64 {
        self.transfer(key, amount, commit, true)
    }

    /// `StorageHelper.poweredExtraction` against the local buffer.
    pub fn powered_extract(&mut self, key: &K, amount: u64, commit: bool) -> u64 {
        self.transfer(key, amount, commit, false)
    }

    /// `tryToStoreContents`: refresh the handler, then powered-insert the input stack.
    /// Returns the count still in the input slot.
    pub fn store_input(&mut self) -> u64 {
        let Some((key, amount)) = self.input.clone() else {
            return 0;
        };
        self.update_handler();
        let inserted = self.powered_insert(&key, amount, true);
        let left = amount.saturating_sub(inserted);
        self.input = (left > 0).then_some((key, left));
        left
    }

    /// `serverTick` without a powered grid: drain idle, then store any pending input.
    pub fn server_tick(&mut self) {
        let _ = self.energy.extract(self.idle, true);
        if self.input.is_some() {
            self.store_input();
        }
    }

    pub fn save(&self) -> Saved<K> {
        Saved {
            cell: self.cell.clone(),
            input: self.input.clone(),
            power: self.power(),
            idle: self.idle,
            priority: self.priority,
        }
    }

    pub fn load(saved: Saved<K>) -> Self {
        let mut chest = Self::new();
        chest.cell = saved.cell;
        chest.input = saved.input;
        chest.set_power(saved.power);
        chest.set_idle(saved.idle);
        chest.priority = saved.priority;
        chest
    }

    fn transfer(&mut self, key: &K, amount: u64, commit: bool, insert: bool) -> u64 {
        let simulated = match self.cell.as_mut() {
            Some(cell) if insert => cell.insert(key.clone(), amount, false),
            Some(cell) => cell.extract(key, amount, false),
            None => return 0,
        };
        let moved = affordable(&mut self.energy, simulated, energy_factor(key), commit);
        if !commit || moved == 0 {
            return moved;
        }
        match self.cell.as_mut() {
            Some(cell) if insert => cell.insert(key.clone(), moved, true),
            Some(cell) => cell.extract(key, moved, true),
            None => 0,
        }
    }
}

fn energy_factor<K: CellKey>(key: &K) -> f64 {
    if key.key_type() == KeyType::Fluid {
        125.0
    } else {
        1.0
    }
}

/// `StorageHelper` simulate-then-afford formula. `+ 0.9` is the source long cast.
fn affordable(energy: &mut StoredEnergy, simulated: u64, factor: f64, commit: bool) -> u64 {
    if simulated == 0 {
        return 0;
    }
    let factor = factor.max(1.0);
    let (available, _) = energy.extract(simulated as f64 / factor, false);
    let moved = ((available * factor) + 0.9) as u64;
    let moved = moved.min(simulated);
    if moved == 0 {
        return 0;
    }
    if commit {
        let _ = energy.extract(moved as f64 / factor, true);
    }
    moved
}

#[cfg(test)]
mod tests {
    use super::super::{
        ae2_keys::{Key, KeyProperties, KeyType},
        ae2_storage::{Cell, Tier},
    };
    use super::*;

    fn diamond() -> Key {
        Key::new(
            KeyType::Item,
            "minecraft:diamond".to_owned(),
            Vec::new(),
            KeyProperties {
                max_stack: 64,
                ..KeyProperties::default()
            },
        )
        .unwrap()
    }

    fn cell() -> CellBackend<Key> {
        Cell::new(Tier::K1, KeyType::Item).into()
    }

    fn charged(power: f64) -> Chest<Key> {
        let mut chest = Chest::new();
        chest.set_power(power);
        chest.set_cell(Some(cell()));
        chest
    }

    #[test]
    fn insert_extract_and_save_load_roundtrip_the_cell() {
        let key = diamond();
        let mut chest = charged(500.0);
        assert!(chest.allow_input(&key, 7));
        assert_eq!(chest.idle(), 1.5);
        chest.set_input(key.clone(), 7);
        chest.server_tick();
        assert!(chest.input().is_none());
        assert_eq!(chest.stored(&key), 7);
        // The filter's updateHandler sets idle before this tick drains it, then the insert costs 7.
        assert_eq!(chest.power(), 491.5);

        let loaded = Chest::load(chest.save());
        assert_eq!(loaded.stored(&key), 7);
        assert_eq!(loaded.power(), 491.5);
        assert_eq!(loaded.idle(), 1.5);
        let mut loaded = loaded;
        assert_eq!(loaded.powered_extract(&key, 7, true), 7);
        assert_eq!(loaded.stored(&key), 0);
        assert_eq!(loaded.power(), 484.5);
        assert_eq!(loaded.powered_extract(&key, 1, true), 0);
    }

    #[test]
    fn local_power_above_one_gates_the_input_filter() {
        let key = diamond();
        let mut chest = charged(1.0);
        assert!(!chest.is_powered());
        assert!(!chest.allow_input(&key, 1));
        chest.set_power(1.1);
        assert!(chest.is_powered());
        assert!(chest.allow_input(&key, 1));

        let mut poor = charged(0.2);
        assert!(!poor.is_powered());
        assert_eq!(poor.powered_insert(&key, 1, true), 1);
        assert_eq!(poor.stored(&key), 1);
        assert_eq!(poor.power(), 0.0);
    }

    #[test]
    fn idle_is_set_by_the_handler_and_not_cleared_when_the_cell_leaves() {
        let mut chest = charged(100.0);
        chest.set_input(diamond(), 1);
        chest.server_tick();
        assert_eq!(chest.idle(), 1.5);
        assert_eq!(chest.power(), 99.0);
        chest.set_cell(None);
        chest.update_handler();
        assert_eq!(chest.idle(), 1.5);
        chest.server_tick();
        assert_eq!(chest.power(), 97.5);
    }

    #[test]
    fn input_without_a_readable_cell_is_not_consumed() {
        let mut chest = Chest::new();
        chest.set_power(500.0);
        let key = diamond();
        assert!(!chest.allow_input(&key, 4));
        chest.set_input(key.clone(), 4);
        chest.server_tick();
        assert_eq!(chest.input().map(|(_, n)| *n), Some(4));
        assert_eq!(chest.stored(&key), 0);
        assert_eq!(chest.power(), 500.0);
    }
}
