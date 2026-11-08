// SPDX-License-Identifier: LGPL-3.0-or-later
// Inscriber native inventories, recipe work and private/grid energy bindings.
// Source menu/render packets, name-press components and complete managed-node
// lifecycle are separate work; no replacement recipes or fake output slots.
use super::ae2_recipe_ingredient::normalize as ingredient;
use super::{Game, ae2_inscriber as rules};
use horizon_client::{
    containers::{BlockEntity, ModBlockEntity},
    crafting::{Ingredient, recipe_result},
    items::ItemStack,
};
use horizon_core::geometry::Direction;
use horizon_world::nbt::{JavaString, Tag};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
type Pos = [i32; 3];

fn same(a: Option<&ItemStack>, b: Option<&ItemStack>) -> bool {
    let a = a.filter(|s| s.count > 0 && &*s.item != "minecraft:air");
    let b = b.filter(|s| s.count > 0 && &*s.item != "minecraft:air");
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a.same_item_same_components(b),
        _ => false,
    }
}
pub(super) fn usable(be: &ModBlockEntity) -> bool {
    be.nbt
        .get(&JavaString::from("inv"))
        .is_none_or(|tag| super::ae2_inventory_codec::read_slots(tag, rules::SLOTS).is_ok())
}
pub(super) fn persist(be: &mut ModBlockEntity) {
    if usable(be) {
        be.nbt
            .entry("internalCurrentPower".into())
            .or_insert(Tag::Double(0.0_f64.to_bits()));
        if setting(be, "inscriber_buffer_size").as_deref() == Some("NO") {
            be.nbt.insert(
                "inscriber_input_capacity".into(),
                Tag::String("FOUR".into()),
            );
            be.nbt.remove(&JavaString::from("inscriber_buffer_size"));
        }
        for (key, value) in [
            ("inscriber_input_capacity", "SIXTY_FOUR"),
            ("inscriber_separate_sides", "NO"),
            ("auto_export", "NO"),
        ] {
            be.nbt
                .entry(key.into())
                .or_insert_with(|| Tag::String(value.into()));
        }
        let inv = super::ae2_inventory_codec::write_slots(
            (0..rules::SLOTS).map(|i| be.items.get(i).and_then(Option::as_ref)),
            be.nbt.get(&JavaString::from("inv")),
        );
        be.nbt.insert("inv".into(), inv);
    }
}
#[derive(Default)]
struct Echo {
    observed: bool,
    blocked: bool,
    inv: Option<Tag>,
    items: Vec<Option<ItemStack>>,
    auto_export: bool,
}
impl Echo {
    fn note(&mut self, be: &ModBlockEntity) {
        self.observed = true;
        self.blocked = false;
        self.inv = be.nbt.get(&JavaString::from("inv")).cloned();
        self.items = be.items.iter().take(rules::SLOTS).cloned().collect();
        self.items.resize(rules::SLOTS, None);
        self.auto_export = setting(be, "auto_export").as_deref() == Some("YES");
    }
    fn sync(&mut self, be: &mut ModBlockEntity) {
        let inv = be.nbt.get(&JavaString::from("inv")).cloned();
        let export = setting(be, "auto_export").as_deref() == Some("YES");
        if self.observed && export != self.auto_export && be.data.len() >= 7 {
            be.data[6] = 0;
        }
        self.auto_export = export;
        let Ok(mut items) = super::ae2_inventory_codec::choose_slots(
            inv.as_ref(),
            self.inv.as_ref(),
            &be.items,
            self.observed,
            self.blocked,
            rules::SLOTS,
        ) else {
            self.blocked = true;
            self.inv = inv;
            self.observed = true;
            return;
        };
        for item in &mut items {
            if item
                .as_ref()
                .is_some_and(|s| s.count == 0 || &*s.item == "minecraft:air")
            {
                *item = None;
            }
        }
        if self.observed {
            let changed_input = (0..3).any(|i| {
                !same(
                    self.items.get(i).and_then(Option::as_ref),
                    items[i].as_ref(),
                )
            });
            let changed_output = self.items.get(3) != items.get(3);
            if changed_input || self.blocked {
                if let Some(processing) = be.data.first_mut() {
                    *processing = 0;
                }
            }
            // Source ignores count-only input edits, but output edits and
            // input component changes wake a sleeping tracker.
            if (changed_input || changed_output || self.blocked) && be.data.len() >= 7 {
                be.data[6] = 0;
            }
        }
        be.items.resize(be.items.len().max(rules::SLOTS), None);
        be.items[..rules::SLOTS].clone_from_slice(&items);
        persist(be);
        self.note(be);
    }
}
#[derive(Default)]
pub(super) struct Inventories(BTreeMap<Pos, Echo>);
impl Inventories {
    pub(super) fn sync(&mut self, game: &mut Game<'_>) {
        let positions: BTreeSet<_> = game
            .world
            .block_entities
            .iter()
            .filter_map(|(&(x, y, z), be)| {
                matches!(be, BlockEntity::Mod(be) if be.id == rules::BLOCK)
                    .then_some([x, y, z])
                    .filter(|pos| {
                        game.world.loaded(pos[0], pos[2]) && game.native_block(*pos) == rules::BLOCK
                    })
            })
            .collect();
        self.0.retain(|pos, _| positions.contains(pos));
        for pos in positions {
            if let Some(BlockEntity::Mod(be)) =
                game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
            {
                let before = (be.nbt.clone(), be.items.clone());
                self.0.entry(pos).or_default().sync(be);
                if before != (be.nbt.clone(), be.items.clone()) {
                    game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
                }
            }
        }
    }
    pub(super) fn note(&mut self, pos: Pos, be: &ModBlockEntity) {
        self.0.entry(pos).or_default().note(be);
    }
    pub(super) fn forget(&mut self, pos: Pos) {
        self.0.remove(&pos);
    }
}

#[derive(Clone)]
struct Task {
    result: ItemStack,
    press: bool,
}

fn matches(game: &Game<'_>, value: Option<&Value>, stack: Option<&ItemStack>) -> bool {
    let Some(value) = value else {
        return stack.is_none_or(|s| s.count == 0);
    };
    if value.as_array().is_some_and(Vec::is_empty) {
        return stack.is_none_or(|s| s.count == 0);
    }
    let Some(stack) = stack.filter(|s| s.count > 0) else {
        return false;
    };
    ingredient(value)
        .and_then(|value| Ingredient::parse(&value, game.items()))
        .is_some_and(|matcher| matcher.test(stack))
}
fn task(game: &Game<'_>, be: &ModBlockEntity) -> Option<Task> {
    let at = |i: usize| {
        be.items
            .get(i)
            .and_then(Option::as_ref)
            .filter(|s| s.count > 0)
    };
    let input = at(rules::MIDDLE)?;
    // Source name-press recipes need the original Component codec/append
    // semantics. Do not fall through to a different regular recipe for them.
    let name_a = at(rules::TOP).is_some_and(|s| &*s.item == "ae2:name_press");
    let name_b = at(rules::BOTTOM).is_some_and(|s| &*s.item == "ae2:name_press");
    if (name_a && (name_b || at(rules::BOTTOM).is_none())) || (at(rules::TOP).is_none() && name_b) {
        return None;
    }
    game.recipes()
        .custom
        .get("ae2:inscriber")?
        .iter()
        .find(|r| {
            let Some(ingredients) = r.json.get("ingredients").and_then(Value::as_object) else {
                return false;
            };
            let Some(middle) = ingredients.get("middle") else {
                return false;
            };
            let a = ingredients.get("top");
            let b = ingredients.get("bottom");
            matches(game, Some(middle), Some(input))
                && ((matches(game, a, at(rules::TOP)) && matches(game, b, at(rules::BOTTOM)))
                    || (matches(game, a, at(rules::BOTTOM)) && matches(game, b, at(rules::TOP))))
        })
        .and_then(|r| {
            let mode = r.json.get("mode")?.as_str()?;
            let result = r.json.get("result")?;
            if result.get("components").is_some() {
                return None;
            }
            let result = recipe_result(result)?;
            if result.count == 0 || game.items().get(&result.item).is_none() {
                return None;
            }
            Some(Task {
                result,
                press: mode == "press",
            })
        })
}
fn setting(be: &ModBlockEntity, key: &str) -> Option<String> {
    match be.nbt.get(&JavaString::from(key)) {
        Some(Tag::String(s)) => Some(s.to_string_lossy()),
        _ => None,
    }
}
pub(super) fn separate(be: &ModBlockEntity) -> bool {
    setting(be, "inscriber_separate_sides").as_deref() == Some("YES")
}
pub(super) fn input_capacity(be: &ModBlockEntity) -> u32 {
    if setting(be, "inscriber_buffer_size").as_deref() == Some("NO") {
        return 4;
    }
    match setting(be, "inscriber_input_capacity").as_deref() {
        Some("ONE") => 1,
        Some("FOUR") => 4,
        _ => 64,
    }
}
pub(super) fn container(
    be: &ModBlockEntity,
    facing: &str,
    spin: u8,
) -> horizon_client::containers::ModContainer {
    let mut out = horizon_client::containers::ModContainer {
        slots: 4,
        faces: std::array::from_fn(|_| (0..4).collect()),
        inputs: vec![0, 1, 2],
        outputs: vec![3],
        menu: None,
    };
    if !usable(be) {
        out.inputs.clear();
        out.outputs.clear();
        return out;
    }
    if machine(be).smash {
        out.inputs.clear();
    }
    if separate(be) {
        let up = super::ae2_orientation::up_from_spin(facing, spin);
        for (i, name) in super::ae2_orientation::DIRECTIONS.into_iter().enumerate() {
            out.faces[i] = if name == up {
                vec![0]
            } else if name == super::ae2_orientation::opposite(up) {
                vec![1]
            } else {
                vec![2, 3]
            };
        }
        if !machine(be).smash {
            out.outputs = vec![0, 1, 3];
        }
    }
    out
}
pub(super) fn allow_take(be: &ModBlockEntity, slot: usize) -> bool {
    usable(be) && (slot == rules::OUTPUT || (slot < 2 && separate(be) && !machine(be).smash))
}
pub(super) fn allow_insert(
    game: &Game<'_>,
    be: &ModBlockEntity,
    slot: usize,
    stack: &ItemStack,
) -> bool {
    if slot >= 3 || !usable(be) || machine(be).smash || stack.count == 0 {
        return false;
    }
    if slot < 2 && &*stack.item == "ae2:name_press" {
        return true;
    }
    let mut inputs = (0..3)
        .map(|i| {
            be.items
                .get(i)
                .and_then(Option::as_ref)
                .filter(|s| s.count > 0 && &*s.item != "minecraft:air")
        })
        .collect::<Vec<_>>();
    if slot == rules::MIDDLE
        && inputs[..2]
            .iter()
            .flatten()
            .any(|s| &*s.item == "ae2:name_press")
    {
        return true;
    }
    inputs[slot] = Some(stack);
    game.recipes()
        .custom
        .get("ae2:inscriber")
        .is_some_and(|recipes| {
            recipes.iter().any(|r| {
                let Some(ingredients) = r.json.get("ingredients").and_then(Value::as_object) else {
                    return false;
                };
                let Some(middle) = ingredients.get("middle") else {
                    return false;
                };
                if inputs[2].is_some() && !matches(game, Some(middle), inputs[2]) {
                    return false;
                }
                let top = ingredients.get("top");
                let bottom = ingredients.get("bottom");
                match (inputs[0], inputs[1]) {
                    (None, None) => true,
                    (Some(a), None) | (None, Some(a)) => {
                        matches(game, top, Some(a)) || matches(game, bottom, Some(a))
                    }
                    (Some(a), Some(b)) => {
                        (matches(game, top, Some(a)) && matches(game, bottom, Some(b)))
                            || (matches(game, top, Some(b)) && matches(game, bottom, Some(a)))
                    }
                }
            })
        })
}
pub(super) fn insert_stack(
    target: &mut Option<ItemStack>,
    mut stack: ItemStack,
    limit: u32,
) -> (Option<ItemStack>, bool) {
    if target
        .as_ref()
        .is_some_and(|s| !s.same_item_same_components(&stack))
    {
        return (Some(stack), false);
    }
    let count = target.as_ref().map_or(0, |s| s.count);
    let accepted = stack.count.min(limit.saturating_sub(count));
    if accepted == 0 {
        return (Some(stack), false);
    }
    let mut placed = target.as_ref().unwrap_or(&stack).clone();
    placed.count = count + accepted;
    *target = Some(placed);
    stack.count -= accepted;
    ((stack.count > 0).then_some(stack), true)
}
fn speed_cards(be: &ModBlockEntity) -> u32 {
    let mut slots = [false; 4];
    if let Some(Tag::List(items)) = be.nbt.get(&JavaString::from("upgrades")) {
        for tag in items {
            let Tag::Compound(map) = tag else {
                continue;
            };
            let slot = match map.get(&JavaString::from("Slot")) {
                Some(Tag::Int(i)) => *i,
                Some(Tag::Byte(i)) => i32::from(*i),
                Some(Tag::Short(i)) => i32::from(*i),
                _ => 0,
            };
            if let Ok(slot) = usize::try_from(slot)
                && slot < 4
            {
                slots[slot] = super::ae2_inventory_codec::read_stack(tag)
                    .ok()
                    .flatten()
                    .is_some_and(|s| &*s.item == "ae2:speed_card");
            }
        }
    }
    slots.into_iter().filter(|s| *s).count() as u32
}
fn machine(be: &ModBlockEntity) -> rules::Machine {
    rules::Machine {
        processing: be.data.first().copied().unwrap_or(0),
        smash: be.data.get(1).is_some_and(|s| *s != 0),
        final_step: be.data.get(2).copied().unwrap_or(0),
    }
}
fn store(be: &mut ModBlockEntity, m: &rules::Machine) {
    be.data.resize(be.data.len().max(7), 0);
    be.data[..3].copy_from_slice(&[m.processing, i32::from(m.smash), m.final_step]);
}
fn time(be: &ModBlockEntity) -> u64 {
    (be.data[3] as u32 as u64) | ((be.data[4] as u32 as u64) << 32)
}
fn set_time(be: &mut ModBlockEntity, tick: u64) {
    be.data[3] = tick as u32 as i32;
    be.data[4] = (tick >> 32) as u32 as i32;
}

struct Host<'a, 'b> {
    game: &'a mut Game<'b>,
    be: &'a mut ModBlockEntity,
    pos: Pos,
    task: Option<Task>,
}
impl Host<'_, '_> {
    fn limit(&self, stack: &ItemStack, slot: usize) -> u32 {
        let host = if slot == rules::OUTPUT {
            64
        } else {
            input_capacity(self.be)
        };
        host.min(
            self.game
                .items()
                .get(&stack.item)
                .map_or(64, |i| i.max_stack)
                .max(1),
        )
    }
    fn insert_output(&mut self, stack: &ItemStack, commit: bool) -> u32 {
        let existing = self.be.items[rules::OUTPUT].as_ref();
        if existing.is_some_and(|s| !s.same_item_same_components(stack)) {
            return stack.count;
        }
        let count = existing.map_or(0, |s| s.count);
        let accepted = stack
            .count
            .min(self.limit(stack, rules::OUTPUT).saturating_sub(count));
        if commit && accepted > 0 {
            let mut next = existing.unwrap_or(stack).clone();
            next.count = count + accepted;
            self.be.items[rules::OUTPUT] = Some(next);
        }
        stack.count - accepted
    }
    fn changed(&mut self, m: &rules::Machine) {
        store(self.be, m);
        persist(self.be);
        self.game
            .world
            .modified
            .insert((self.pos[0] >> 4, self.pos[2] >> 4));
        if let Some(log) = &mut self.game.world.block_log {
            log.push((self.pos[0], self.pos[1], self.pos[2]));
        }
    }
}
impl rules::Host for Host<'_, '_> {
    fn has_task(&mut self) -> bool {
        self.task = task(self.game, self.be);
        self.task.is_some()
    }
    fn output_fits(&mut self) -> bool {
        self.task
            .clone()
            .is_some_and(|t| self.insert_output(&t.result, false) == 0)
    }
    fn insert_result(&mut self) -> bool {
        self.task
            .clone()
            .is_some_and(|t| self.insert_output(&t.result, true) == 0)
    }
    fn consume_inputs(&mut self, m: &rules::Machine) {
        store(self.be, m);
        let press = self.task.as_ref().is_some_and(|t| t.press);
        for i in 0..3 {
            if (i == rules::MIDDLE || press)
                && let Some(stack) = &mut self.be.items[i]
            {
                stack.count = stack.count.saturating_sub(1);
                if stack.count == 0 {
                    self.be.items[i] = None;
                }
            }
        }
        self.task = None;
        persist(self.be);
    }
    fn grid_ready(&self) -> bool {
        super::ae2_grid_glue::inscriber_ready(self.game.dim.id(), self.pos)
    }
    fn speed_cards(&self) -> u32 {
        speed_cards(self.be)
    }
    fn local_power(&mut self, amount: f64, commit: bool) -> f64 {
        let saved = match self.be.nbt.get(&JavaString::from("internalCurrentPower")) {
            Some(Tag::Double(v)) if f64::from_bits(*v).is_finite() => f64::from_bits(*v),
            _ => 0.0,
        };
        let mut energy = super::ae2_energy::StoredEnergy::new(saved, rules::CAPACITY);
        let result = energy.extract(amount, commit).0;
        if commit {
            self.be.nbt.insert(
                "internalCurrentPower".into(),
                Tag::Double(energy.amount().to_bits()),
            );
        }
        result
    }
    fn grid_power(&mut self, amount: f64, commit: bool) -> f64 {
        super::ae2_grid_glue::extract(self.game.dim.id(), self.pos, amount, 1.0, commit)
    }
    fn push_result(&mut self) -> bool {
        if !<Self as rules::Host>::auto_export_work(self) {
            return false;
        }
        let facing = self
            .game
            .native_block_property(self.pos, "facing")
            .unwrap_or_else(|| "north".into());
        let spin = self
            .game
            .native_block_property(self.pos, "spin")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let up = super::ae2_orientation::up_from_spin(&facing, spin);
        let faces = [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ];
        let offsets = [
            [0, -1, 0],
            [0, 1, 0],
            [0, 0, -1],
            [0, 0, 1],
            [-1, 0, 0],
            [1, 0, 0],
        ];
        for (i, face) in faces.into_iter().enumerate() {
            let name = super::ae2_orientation::DIRECTIONS[i];
            if separate(self.be) && (name == up || name == super::ae2_orientation::opposite(up)) {
                continue;
            }
            let Some(mut extracted) = self.be.items[rules::OUTPUT].take() else {
                return false;
            };
            let before = extracted.count;
            let taken = before.min(64);
            if before > taken {
                let mut rest = extracted.clone();
                rest.count -= taken;
                self.be.items[rules::OUTPUT] = Some(rest);
            }
            extracted.count = taken;
            let target =
                std::array::from_fn::<_, 3, _>(|k| self.pos[k].wrapping_add(offsets[i][k]));
            let rest = self.game.insert_into_container(
                (target[0], target[1], target[2]),
                extracted,
                face.opposite(),
            );
            if let Some(rest) = rest {
                self.insert_output(&rest, true);
            }
            if self.be.items[rules::OUTPUT].as_ref().map_or(0, |s| s.count) != before {
                persist(self.be);
                return true;
            }
        }
        false
    }
    fn auto_export_work(&self) -> bool {
        self.be
            .items
            .get(rules::OUTPUT)
            .and_then(Option::as_ref)
            .is_some_and(|s| s.count > 0)
            && setting(self.be, "auto_export").as_deref() == Some("YES")
    }
    fn mark_for_update(&mut self, m: &rules::Machine) {
        self.changed(m);
    }
    fn save_changes(&mut self, m: &rules::Machine) {
        store(self.be, m);
        persist(self.be);
        self.game
            .world
            .modified
            .insert((self.pos[0] >> 4, self.pos[2] >> 4));
    }
}

pub(super) fn tick(game: &mut Game<'_>, pos: Pos, be: &mut ModBlockEntity) {
    if !usable(be) {
        return;
    }
    be.items.resize(be.items.len().max(rules::SLOTS), None);
    let mut m = machine(be);
    if be.data.len() < 7 {
        store(be, &m);
        set_time(be, game.ticks);
        be.data[5] = 10;
        let mut host = Host {
            game,
            be,
            pos,
            task: None,
        };
        let sleeping = m.sleeping_request(&mut host);
        store(host.be, &m);
        host.be.data[6] = i32::from(sleeping);
    }
    if be.data[6] != 0 || game.ticks.wrapping_sub(time(be)) < be.data[5] as u64 {
        return;
    }
    let mut host = Host {
        game,
        be,
        pos,
        task: None,
    };
    let modulation = m.tick(&mut host);
    store(host.be, &m);
    set_time(host.be, host.game.ticks);
    host.be.data[5] = match modulation {
        rules::Modulation::Urgent => 1,
        rules::Modulation::Slower => (host.be.data[5] + 1).clamp(1, 20),
        rules::Modulation::Sleep => 20,
    };
    host.be.data[6] = i32::from(modulation == rules::Modulation::Sleep);
    persist(host.be);
    super::ae2_grid_glue::note_inscriber(host.game.dim.id(), pos, host.be);
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_world::nbt::Compound;
    fn entity() -> ModBlockEntity {
        ModBlockEntity {
            id: rules::BLOCK.into(),
            nbt: Compound::new(),
            items: vec![None; 4],
            data: vec![188, 0, 0, 0, 0, 20, 1],
        }
    }
    #[test]
    fn inventory_echo_keeps_native_edits_and_blocks_malformed_source_until_repair() {
        let mut be = entity();
        be.nbt.insert(
            "inv".into(),
            super::super::ae2_inventory_codec::write_slots(
                [
                    Some(&ItemStack::new("ae2:calculation_processor_press", 1)),
                    None,
                    Some(&ItemStack::new("minecraft:iron_block", 2)),
                    None,
                ],
                None,
            ),
        );
        let mut echo = Echo::default();
        echo.sync(&mut be);
        assert_eq!(be.items[2].as_ref().unwrap().count, 2);
        be.items[2].as_mut().unwrap().count = 1;
        echo.sync(&mut be);
        assert_eq!(be.items[2].as_ref().unwrap().count, 1);
        be.nbt.insert("inv".into(), Tag::Int(9));
        echo.sync(&mut be);
        assert!(echo.blocked);
        be.items[2] = None;
        echo.sync(&mut be);
        assert_eq!(be.nbt.get(&JavaString::from("inv")), Some(&Tag::Int(9)));
        be.nbt.insert(
            "inv".into(),
            super::super::ae2_inventory_codec::write_slots(
                [
                    None,
                    None,
                    Some(&ItemStack::new("minecraft:diamond", 1)),
                    None,
                ],
                None,
            ),
        );
        echo.sync(&mut be);
        assert!(!echo.blocked);
        assert_eq!(&*be.items[2].as_ref().unwrap().item, "minecraft:diamond");
        assert_eq!(be.data[0], 0);
        assert_eq!(be.data[6], 0);
        be.data[6] = 1;
        echo.note(&be);
        be.nbt
            .insert("auto_export".into(), Tag::String("YES".into()));
        echo.sync(&mut be);
        assert_eq!(
            be.data[6], 0,
            "AUTO_EXPORT changes wake an unchanged inventory"
        );
    }
    #[test]
    fn speed_counts_occupied_slots_not_counts_and_last_duplicate_slot_wins() {
        let mut be = entity();
        let row = |slot, id, count| {
            Tag::compound([
                ("Slot", Tag::Int(slot)),
                ("id", Tag::String(id)),
                ("count", Tag::Int(count)),
            ])
        };
        be.nbt.insert(
            "upgrades".into(),
            Tag::List(vec![
                row(0, "ae2:speed_card".into(), 99),
                row(2, "ae2:speed_card".into(), 1),
                row(0, "minecraft:stone".into(), 1),
                row(7, "ae2:speed_card".into(), 1),
            ]),
        );
        assert_eq!(speed_cards(&be), 1);
        be.nbt
            .insert("inscriber_input_capacity".into(), Tag::String("ONE".into()));
        assert_eq!(input_capacity(&be), 1);
        be.nbt
            .insert("inscriber_buffer_size".into(), Tag::String("NO".into()));
        assert_eq!(input_capacity(&be), 4);
        persist(&mut be);
        assert!(
            !be.nbt
                .contains_key(&JavaString::from("inscriber_buffer_size"))
        );
        be.nbt
            .insert("inscriber_input_capacity".into(), Tag::String("ONE".into()));
        assert_eq!(input_capacity(&be), 1);
    }
    #[test]
    fn separate_faces_follow_spin_and_smash_locks_inputs_but_not_output() {
        let mut be = entity();
        be.nbt
            .insert("inscriber_separate_sides".into(), Tag::String("YES".into()));
        for facing in super::super::ae2_orientation::DIRECTIONS {
            for spin in 0..4 {
                let top = super::super::ae2_orientation::up_from_spin(facing, spin);
                let bottom = super::super::ae2_orientation::opposite(top);
                for smash in [false, true] {
                    be.data[1] = i32::from(smash);
                    let c = container(&be, facing, spin);
                    assert_eq!(c.slots, 4);
                    for (i, face) in super::super::ae2_orientation::DIRECTIONS
                        .into_iter()
                        .enumerate()
                    {
                        assert_eq!(
                            c.faces[i],
                            if face == top {
                                vec![0]
                            } else if face == bottom {
                                vec![1]
                            } else {
                                vec![2, 3]
                            }
                        );
                    }
                    assert_eq!(c.inputs, if smash { vec![] } else { vec![0, 1, 2] });
                    assert_eq!(c.outputs, if smash { vec![3] } else { vec![0, 1, 3] });
                }
            }
        }
    }
    #[test]
    fn insertion_preserves_component_identity_limits_remainder_and_existing_overstacks() {
        let mut incoming = ItemStack::new("minecraft:iron_block", 4);
        incoming
            .data_mut()
            .set_other("example:marker", Some(Tag::Long(19)));
        let mut previous = incoming.clone();
        previous.count = 63;
        let mut slot = Some(previous);
        let (rest, changed) = insert_stack(&mut slot, incoming.clone(), 64);
        assert!(changed);
        assert_eq!(slot.as_ref().unwrap().count, 64);
        assert_eq!(rest.as_ref().unwrap().count, 3);
        assert!(rest.unwrap().same_item_same_components(&incoming));
        let before = slot.clone();
        let mut different = incoming.clone();
        different
            .data_mut()
            .set_other("example:marker", Some(Tag::Long(20)));
        assert!(!insert_stack(&mut slot, different, 64).1);
        assert_eq!(slot, before);
        slot.as_mut().unwrap().count = 99;
        assert!(!insert_stack(&mut slot, incoming, 64).1);
        assert_eq!(slot.unwrap().count, 99);
    }
}
