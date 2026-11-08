// SPDX-License-Identifier: LGPL-3.0-or-later
// Horizon adapter for the first Charger/Crank vertical slice. See docs/PARITY.md
// for the explicit gaps; full-block network energy lives in ae2_grid_glue.

use super::Game;
use super::ae2_charger::{self as rules, Crank, CrankResult};
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::crafting::{Ingredient, recipe_result};
use horizon_client::items::ItemStack;
use horizon_core::random::RandomSource;
use horizon_world::nbt::{Compound, JavaString, Tag};

const CHARGER: &str = "ae2:charger";
const CRANK: &str = "ae2:crank";
type Pos = [i32; 3];

pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && (matches!(block, CHARGER | CRANK) || super::ae2_grid_glue::owns(block)) {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if matches!(block, CHARGER | CRANK | "ae2:inscriber") {
        *ticks = true;
    }
}

pub(super) fn container(block: &str, out: &mut Option<ModContainer>) {
    if block == "ae2:inscriber" {
        *out = Some(ModContainer {
            slots: 4,
            faces: std::array::from_fn(|_| (0..4).collect()),
            inputs: vec![0, 1, 2],
            outputs: vec![3],
            menu: None,
        });
    }
    if block == "ae2:drive" {
        // The wire menu carries 18 slots; the native screen and server hide
        // 10..18. Only the original ten slots are mounted as storage cells.
        *out = Some(ModContainer {
            slots: 18,
            faces: std::array::from_fn(|_| (0..10).collect()),
            inputs: (0..10).collect(),
            outputs: (0..10).collect(),
            menu: Some("minecraft:generic_9x2".to_owned()),
        });
    }
    if block == CHARGER {
        // AEBaseInvBlockEntity exposes the same inventory on every side.
        // Dynamic ChargerInvFilter rules are enforced by the machines hooks.
        *out = Some(ModContainer {
            slots: 1,
            faces: std::array::from_fn(|_| vec![0]),
            inputs: vec![0],
            outputs: vec![0],
            menu: None,
        });
    }
}

fn energy(nbt: &Compound) -> f64 {
    match nbt.get(&JavaString::from("internalCurrentPower")) {
        Some(Tag::Double(v)) if f64::from_bits(*v).is_finite() => {
            f64::from_bits(*v).clamp(0.0, rules::POWER_MAXIMUM)
        }
        _ => 0.0,
    }
}

fn set_energy(nbt: &mut Compound, value: f64) {
    nbt.insert(
        JavaString::from("internalCurrentPower"),
        Tag::Double(value.to_bits()),
    );
}

// Upstream leaves rotation/charge/hits transient. Horizon's `data` slots are
// transient too, unlike its NBT compound. This avoids resuming turns after load.
fn read_crank(be: &ModBlockEntity) -> Crank {
    Crank {
        rotation: be.data.first().copied().unwrap_or(0),
        charge: be.data.get(1).copied().unwrap_or(0),
        hits: be.data.get(2).copied().unwrap_or(0).max(0) as u32,
    }
}
fn write_crank(be: &mut ModBlockEntity, crank: Crank) {
    be.data = vec![crank.rotation, crank.charge, crank.hits as i32];
}

fn direction(name: &str) -> Pos {
    match name {
        "down" => [0, -1, 0],
        "up" => [0, 1, 0],
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        "east" => [1, 0, 0],
        _ => [0, 0, -1],
    }
}

fn facing(game: &Game<'_>, pos: Pos) -> Pos {
    direction(
        game.native_block_property(pos, "facing")
            .as_deref()
            .unwrap_or_default(),
    )
}

/// The crank's attached crankable on `facing`: only the Charger is ported, and
/// the source requires it to point back at the crank. Shared with wrench turns.
pub(super) fn crankable_attached(game: &Game<'_>, pos: Pos, facing: &str) -> Option<Pos> {
    let dir = direction(facing);
    let target = std::array::from_fn(|i| pos[i] - dir[i]);
    (matches!(
        game.native_block(target).as_str(),
        CHARGER | "ae2:inscriber"
    ) && game.native_block_property(target, "facing").as_deref() != Some(facing))
    .then_some(target)
}

fn crank_target(game: &Game<'_>, pos: Pos) -> Option<Pos> {
    let face = game
        .native_block_property(pos, "facing")
        .unwrap_or_default();
    crankable_attached(game, pos, &face)
}

pub(super) fn ingredient_json(value: &serde_json::Value) -> Option<serde_json::Value> {
    use serde_json::Value;
    match value {
        Value::String(_) => Some(value.clone()),
        Value::Object(o) => o
            .get("item")
            .and_then(Value::as_str)
            .map(|s| Value::String(s.to_owned()))
            .or_else(|| {
                o.get("tag")
                    .and_then(Value::as_str)
                    .map(|s| Value::String(format!("#{s}")))
            }),
        Value::Array(a) => Some(Value::Array(a.iter().filter_map(ingredient_json).collect())),
        _ => None,
    }
}

fn recipe(game: &Game<'_>, stack: &ItemStack) -> Option<ItemStack> {
    game.recipes()
        .custom
        .get("ae2:charger")?
        .iter()
        .find_map(|r| {
            // GuideME results contain components that are not ported. Refuse them
            // rather than silently manufacturing an invalid book or tool.
            if r.json["result"].get("components").is_some() {
                return None;
            }
            let value = ingredient_json(&r.json["ingredient"])?;
            let matcher = Ingredient::parse(&value, game.items())?;
            matcher
                .test(stack)
                .then(|| recipe_result(&r.json["result"]))?
        })
}

fn charger_work(game: &mut Game<'_>, pos: Pos, be: &mut ModBlockEntity, power: &mut f64) -> bool {
    if !super::ae2_charger_inventory::usable(be) {
        return false;
    }
    let Some(stack) = be.items.first().and_then(Option::as_ref) else {
        return false;
    };
    match super::ae2_item_energy_glue::load(stack) {
        Err(_) => return false, // Strict codec refusal preserves the original stack.
        Ok(Some(mut item)) if item.is_chargeable() => {
            let working = rules::charge_item(
                power,
                &mut item.energy,
                item.charge_rate,
                rules::CHARGER_INTERVAL,
                1.0,
                1.0,
                |amount| super::ae2_grid_glue::extract(game.dim.id(), pos, amount, 1.0, true),
            );
            if working {
                super::ae2_item_energy_glue::persist(be.items[0].as_mut().unwrap(), &item);
            }
            return working;
        }
        _ => {}
    }
    let result = recipe(game, stack);
    // Consume world RNG only when the non-chargeable recipe branch would roll.
    let random = if *power >= rules::POWER_THRESHOLD && result.is_some() {
        game.world.random.next_float()
    } else {
        0.0
    };
    let (working, done) = rules::recipe_work(power, result.is_some(), random, 1.0);
    if done {
        be.items[0] = result;
    }
    working
}

pub(super) fn tick(game: &mut Game<'_>, pos: Pos, be: &mut ModBlockEntity) {
    match be.id.as_str() {
        "ae2:inscriber" => super::ae2_inscriber_glue::tick(game, pos, be),
        CRANK => {
            let mut crank = read_crank(be);
            if crank.tick()
                && let Some(target) = crank_target(game, pos)
            {
                game.native_with_mod_block_entity(target, |target| {
                    if matches!(target.id.as_str(), CHARGER | "ae2:inscriber") {
                        let value =
                            (energy(&target.nbt) + rules::POWER_PER_TURN).min(rules::POWER_MAXIMUM);
                        set_energy(&mut target.nbt, value);
                    }
                });
            }
            write_crank(be, crank);
        }
        CHARGER => {
            be.data.resize(2, 0);
            be.data[0] += 1;
            if be.data[0] < rules::CHARGER_INTERVAL as i32 {
                return;
            }
            be.data[0] = 0;
            let mut power = energy(&be.nbt);
            let working = charger_work(game, pos, be, &mut power);
            // ChargerBlockEntity.doWork refills AFTER processing, from the ME
            // grid, at most 800 AE per call. The local crank buffer is private.
            if power < rules::POWER_THRESHOLD {
                power += super::ae2_grid_glue::extract(
                    game.dim.id(),
                    pos,
                    (rules::POWER_MAXIMUM - power).min(800.0),
                    1.0,
                    true,
                );
            }
            // Upstream working is transient and synchronized, not saved.
            be.data[1] = i32::from(working);
            be.nbt.remove(&JavaString::from("HorizonWorking"));
            set_energy(&mut be.nbt, power);
            super::ae2_charger_inventory::persist(be);
        }
        _ => {}
    }
}

pub(super) fn right_click(game: &mut Game<'_>, pos: Pos, canceled: &mut bool) {
    if *canceled {
        return;
    }
    match game.native_block(pos).as_str() {
        CHARGER => {
            *canceled = true;
            super::ae2_grid_glue::sync_chargers(game);
            if let Some(hand) = game.held().cloned() {
                if !super::ae2_item_energy_glue::chargeable(&hand) && recipe(game, &hand).is_none()
                {
                    return;
                }
                let inserted = game
                    .native_with_mod_block_entity(pos, |be| {
                        if be.id != CHARGER || !super::ae2_charger_inventory::usable(be) {
                            return false;
                        }
                        be.items.resize(1, None);
                        if be.items[0].is_some() {
                            return false;
                        }
                        let mut one = hand.clone();
                        one.count = 1;
                        be.items[0] = Some(one);
                        super::ae2_charger_inventory::persist(be);
                        true
                    })
                    .unwrap_or(false);
                if inserted && let Some(hand) = game.held_mut().as_mut() {
                    hand.count -= 1;
                    if hand.count == 0 {
                        *game.held_mut() = None;
                    }
                }
            } else {
                let taken = game
                    .native_with_mod_block_entity(pos, |be| {
                        if be.id != CHARGER || !super::ae2_charger_inventory::usable(be) {
                            return None;
                        }
                        let taken = be.items.first_mut().and_then(Option::take);
                        super::ae2_charger_inventory::persist(be);
                        taken
                    })
                    .flatten();
                if let Some(stack) = taken {
                    let dir = facing(game, pos);
                    let at = std::array::from_fn(|i| f64::from(pos[i] + dir[i]) + 0.5);
                    game.spawn_item(stack, at, [0.0, 0.0, 0.0], 10);
                }
            }
        }
        CRANK => {
            *canceled = true;
            let can_turn = crank_target(game, pos).and_then(|target| {
                game.native_with_mod_block_entity(target, |be| rules::can_turn(energy(&be.nbt)))
            });
            let result = game.native_with_mod_block_entity(pos, |be| {
                let mut crank = read_crank(be);
                let result = crank.power(can_turn);
                write_crank(be, crank);
                result
            });
            if result == Some(CrankResult::Break) {
                let air = game.pack.assets.registry.default_state("minecraft:air");
                if let Some(air) = air {
                    game.native_set_block(game.dim.id(), pos, air, 3);
                }
            }
        }
        "ae2:drive" => super::ae2_grid_glue::flush_drives(game),
        _ if super::ae2_grid_glue::owns(&game.native_block(pos)) => {
            // Native energy cells have no vanilla template menu.
            *canceled = true;
        }
        _ => {}
    }
}
