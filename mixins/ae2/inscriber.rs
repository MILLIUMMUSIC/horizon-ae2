// SPDX-License-Identifier: LGPL-3.0-or-later
// InscriberBlockEntity.tickingRequest/hasCraftWork at upstream.lock.json.
// Copyright (c) AlgorithmX2 / AE2 contributors; 2026 Horizon port contributors.

pub const BLOCK: &str = "ae2:inscriber";
pub const CAPACITY: f64 = 1600.0;
pub const MAX_PROCESSING_STEPS: i32 = 200;
pub const SLOTS: usize = 4;
pub const TOP: usize = 0;
pub const BOTTOM: usize = 1;
pub const MIDDLE: usize = 2;
pub const OUTPUT: usize = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Machine {
    pub processing: i32,
    pub smash: bool,
    pub final_step: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modulation {
    Urgent,
    Slower,
    Sleep,
}

/// Bindings supply the real inventories, recipe cache, energy services and
/// notifications. Completion inserts first and consumes ingredients only when
/// the entire result fits, including a partially accepted result's side effect.
pub trait Host {
    fn has_task(&mut self) -> bool;
    fn output_fits(&mut self) -> bool;
    fn insert_result(&mut self) -> bool;
    fn consume_inputs(&mut self, machine: &Machine);
    fn grid_ready(&self) -> bool;
    fn speed_cards(&self) -> u32;
    fn local_power(&mut self, amount: f64, commit: bool) -> f64;
    fn grid_power(&mut self, amount: f64, commit: bool) -> f64;
    fn push_result(&mut self) -> bool;
    fn auto_export_work(&self) -> bool;
    fn mark_for_update(&mut self, machine: &Machine);
    fn save_changes(&mut self, machine: &Machine);
}

pub fn speed_factor(cards: u32) -> i32 {
    match cards {
        1 => 3,
        2 => 5,
        3 => 10,
        4 => 50,
        _ => 2,
    }
}

impl Machine {
    /// getTickingRequest short-circuits the craft query during auto-export.
    pub fn sleeping_request(&mut self, host: &mut impl Host) -> bool {
        !host.auto_export_work() && !self.has_craft_work(host)
    }

    fn has_craft_work(&mut self, host: &mut impl Host) -> bool {
        if host.has_task() {
            return host.output_fits();
        }
        self.processing = 0;
        self.smash
    }

    /// One IGridTickable callback; not one arbitrary elapsed world tick.
    /// Source ignores ticksSinceLastCall for processing and smash advancement.
    pub fn tick(&mut self, host: &mut impl Host) -> Modulation {
        if self.smash {
            self.final_step = self.final_step.wrapping_add(1);
            if self.final_step == 8 {
                if host.has_task() && host.insert_result() {
                    self.processing = 0;
                    host.consume_inputs(self);
                }
                host.save_changes(self);
            } else if self.final_step == 16 {
                self.final_step = 0;
                self.smash = false;
                host.mark_for_update(self);
            }
        } else if self.has_craft_work(host) {
            if host.grid_ready() {
                let speed = speed_factor(host.speed_cards());
                let consumption = f64::from(10 * speed);
                let threshold = consumption - 0.01;
                let mut from_local = true;
                let mut available = host.local_power(consumption, false);
                if available <= threshold {
                    from_local = false;
                    available = host.grid_power(consumption, false);
                }
                if available > threshold {
                    if from_local {
                        host.local_power(consumption, true);
                    } else {
                        host.grid_power(consumption, true);
                    }
                    self.processing = self.processing.wrapping_add(speed);
                }
            }
            if self.processing > MAX_PROCESSING_STEPS {
                self.processing = MAX_PROCESSING_STEPS;
                if host.has_task() && host.output_fits() {
                    self.smash = true;
                    self.final_step = 0;
                    host.mark_for_update(self);
                }
            }
        }
        if host.push_result() || self.has_craft_work(host) {
            Modulation::Urgent
        } else if host.auto_export_work() {
            Modulation::Slower
        } else {
            Modulation::Sleep
        }
    }
}
