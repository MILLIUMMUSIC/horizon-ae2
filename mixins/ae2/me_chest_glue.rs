// SPDX-License-Identifier: LGPL-3.0-or-later
// Native ME Chest: one cell, the side input slot, and one withdraw stack.
// The source terminal menu (BasicCellChestMenu) is not this screen.

use super::{
    Game,
    ae2_cells as codec,
    ae2_keys::Key,
    ae2_me_chest::{Chest, BLOCK, CELL_SLOT, INPUT, LOCAL_CAPACITY, SLOTS, WITHDRAW},
};
use horizon_client::containers::{BlockEntity, ModBlockEntity};
use horizon_client::items::ItemStack;
use horizon_client::menu::Input;
use horizon_world::nbt::{Compound, JavaString, Tag};

pub(super) fn entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && block == BLOCK {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if block == BLOCK {
        *ticks = true;
    }
}

pub(super) fn container(block: &str, out: &mut Option<horizon_client::containers::ModContainer>) {
    if block != BLOCK {
        return;
    }
    // Slot 0 is the non-front input inventory. The cell and withdraw slots are
    // menu-only; a static face table cannot split the front the way the source does.
    *out = Some(horizon_client::containers::ModContainer {
        slots: SLOTS,
        faces: std::array::from_fn(|_| vec![INPUT]),
        inputs: vec![INPUT],
        outputs: vec![CELL_SLOT],
        menu: Some("minecraft:generic_9x1".to_owned()),
    });
}

pub(super) fn screen(block: &str, out: &mut Option<super::native_hooks::NativeScreen>) {
    if block != BLOCK {
        return;
    }
    *out = Some(super::native_hooks::NativeScreen {
        width: 176.0,
        height: 166.0,
        slots: (0..SLOTS)
            .map(|slot| match slot {
                INPUT => Some([26.0, 35.0]),
                CELL_SLOT => Some([80.0, 35.0]),
                WITHDRAW => Some([134.0, 35.0]),
                _ => None,
            })
            .collect(),
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: false,
    });
}

pub(super) fn tick(game: &mut Game<'_>, pos: [i32; 3], be: &mut ModBlockEntity) {
    if be.id != BLOCK {
        return;
    }
    sync(game, pos, be, false);
}

pub(super) fn before_click(
    game: &mut Game<'_>,
    slot: Option<usize>,
    button: i32,
    input: Input,
) {
    if button != 0 || !matches!(input, Input::Pickup | Input::QuickMove) {
        return;
    }
    if slot != Some(300 + WITHDRAW) {
        return;
    }
    let Some(pos) = game.container else {
        return;
    };
    if game.native_block([pos.0, pos.1, pos.2]) != BLOCK {
        return;
    }
    let key = (pos.0, pos.1, pos.2);
    let Some(snapshot) = game.world.block_entities.get(&key).and_then(|entity| {
        let BlockEntity::Mod(be) = entity else {
            return None;
        };
        (be.id == BLOCK).then(|| be.clone())
    }) else {
        return;
    };
    let mut updated = snapshot;
    sync(game, [pos.0, pos.1, pos.2], &mut updated, true);
    if let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&key) {
        *be = updated;
        game.world.modified.insert((pos.0 >> 4, pos.2 >> 4));
    }
}

pub(super) fn slot_rule(
    game: &Game<'_>,
    slot: usize,
    stack: &ItemStack,
) -> Option<(bool, u32)> {
    let pos = game.container?;
    if game.native_block([pos.0, pos.1, pos.2]) != BLOCK || !(300..309).contains(&slot) {
        return None;
    }
    let index = slot - 300;
    if index == CELL_SLOT {
        return Some((
            stack.count > 0 && horizon_client::lan::menu::ae2_slot_rules::handled_cell(&stack.item),
            1,
        ));
    }
    if index == WITHDRAW {
        return Some((false, 64));
    }
    if index > WITHDRAW {
        return Some((false, 0));
    }
    let max = game
        .items()
        .get(&stack.item)
        .map(|item| item.max_stack)
        .unwrap_or(64);
    let allowed = stack.count > 0 && input_allowed(game, pos, stack);
    Some((allowed, max))
}

fn input_allowed(game: &Game<'_>, pos: (i32, i32, i32), stack: &ItemStack) -> bool {
    let Some(BlockEntity::Mod(be)) = game.world.block_entities.get(&pos) else {
        return false;
    };
    let mut chest = load(game, be).chest;
    let Ok(key) = codec::item_key(stack, game.items()) else {
        return false;
    };
    chest.allow_input(&key, u64::from(stack.count))
}

struct Loaded {
    chest: Chest<Key>,
    cell_item: Option<ItemStack>,
    readable: bool,
    input_parsed: bool,
}

fn load(game: &Game<'_>, be: &ModBlockEntity) -> Loaded {
    let mut chest = Chest::new();
    chest.set_power(power(&be.nbt));
    chest.set_idle(idle(&be.data));
    chest.set_priority(priority(&be.nbt));
    let cell_item = be.items.get(CELL_SLOT).cloned().flatten();
    let readable = match cell_item.as_ref() {
        Some(stack) => match load_cell(game, stack) {
            Ok(cell) => {
                chest.set_cell(Some(cell));
                true
            }
            Err(error) => {
                eprintln!("AE2 ME Chest cell: {error}");
                false
            }
        },
        None => true,
    };
    let mut input_parsed = false;
    if let Some(stack) = be.items.get(INPUT).cloned().flatten()
        && let Ok(key) = codec::item_key(&stack, game.items())
    {
        chest.set_input(key, u64::from(stack.count));
        input_parsed = true;
    }
    Loaded {
        chest,
        cell_item,
        readable,
        input_parsed,
    }
}

fn sync(game: &Game<'_>, pos: [i32; 3], be: &mut ModBlockEntity, withdraw: bool) {
    if be.items.len() != SLOTS {
        be.items.resize(SLOTS, None);
    }
    let online = super::ae2_grid_glue::chest_online(game.dim.id(), pos);
    let Loaded {
        mut chest,
        cell_item,
        readable,
        input_parsed,
    } = load(game, be);
    let input_before = be.items.get(INPUT).cloned().flatten();
    let withdraw_empty = be
        .items
        .get(WITHDRAW)
        .and_then(|slot| slot.as_ref())
        .is_none();
    if online {
        if withdraw && withdraw_empty {
            if let Some(key) = super::ae2_grid_glue::chest_first_item(game.dim.id(), pos) {
                let max = u64::from(key.properties().max_stack.max(1));
                let got = super::ae2_grid_glue::chest_transfer(game.dim.id(), pos, &key, max, false);
                if got > 0
                    && let Ok(stack) = codec::key_to_item(&key, got as u32)
                {
                    be.items[WITHDRAW] = Some(stack);
                }
            }
        } else if !withdraw {
            if let Some(stack) = be.items.get(INPUT).cloned().flatten()
                && let Ok(key) = codec::item_key(&stack, game.items())
            {
                let got = super::ae2_grid_glue::chest_transfer(
                    game.dim.id(),
                    pos,
                    &key,
                    u64::from(stack.count),
                    true,
                );
                let left = u64::from(stack.count).saturating_sub(got);
                be.items[INPUT] = (left > 0).then(|| {
                    let mut stack = stack;
                    stack.count = left as u32;
                    stack
                });
            }
        }
    } else if withdraw && withdraw_empty {
        chest.update_handler();
        if let Some(key) = chest.first_item() {
            let max = u64::from(key.properties().max_stack.max(1));
            let amount = chest.stored(&key).min(max);
            if amount > 0 && codec::key_to_item(&key, amount as u32).is_ok() {
                let got = chest.powered_extract(&key, amount, true);
                if got > 0
                    && let Ok(stack) = codec::key_to_item(&key, got as u32)
                {
                    be.items[WITHDRAW] = Some(stack);
                }
            }
        }
    } else if !withdraw {
        chest.server_tick();
    }
    if readable && !online {
        write_cell(be, cell_item, chest.cell());
    }
    if input_parsed && !online {
        write_input(be, input_before, chest.input().map(|(_, count)| *count));
    }
    write_power(&mut be.nbt, chest.power());
    write_idle(&mut be.data, chest.idle());
    if !be.nbt.contains_key(&JavaString::from("priority")) {
        be.nbt
            .insert(JavaString::from("priority"), Tag::Int(chest.priority()));
    }
}

fn load_cell(
    game: &Game<'_>,
    stack: &ItemStack,
) -> Result<super::ae2_backend::CellBackend<Key>, String> {
    let resolve = |kind, id: &str| codec::resolve_key(game.items(), kind, id);
    if let Some(allowed) = super::ae2_tool_glue::accepted(game, &stack.item) {
        codec::load_restricted_backend(stack, &resolve, allowed)
    } else {
        codec::load_backend(stack, &resolve)
    }
}

fn write_cell(
    be: &mut ModBlockEntity,
    mut stack: Option<ItemStack>,
    cell: Option<&super::ae2_backend::CellBackend<Key>>,
) {
    let Some(cell) = cell else {
        if stack.is_none() {
            be.items[CELL_SLOT] = None;
        }
        return;
    };
    let Some(item) = stack.as_mut() else {
        return;
    };
    if codec::persist_backend(item, cell).is_ok() {
        be.items[CELL_SLOT] = Some(item.clone());
    }
}

fn write_input(be: &mut ModBlockEntity, before: Option<ItemStack>, left: Option<u64>) {
    match (before, left) {
        (Some(_), Some(0) | None) | (None, None) => be.items[INPUT] = None,
        (Some(mut stack), Some(count)) if u64::from(stack.count) != count => {
            stack.count = count as u32;
            be.items[INPUT] = (count > 0).then_some(stack);
        }
        (Some(stack), Some(_)) => be.items[INPUT] = Some(stack),
        (None, Some(_)) => {}
    }
}

fn power(nbt: &Compound) -> f64 {
    match nbt.get(&JavaString::from("internalCurrentPower")) {
        Some(Tag::Double(bits)) => {
            let value = f64::from_bits(*bits);
            if value.is_finite() {
                value.clamp(0.0, LOCAL_CAPACITY)
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn write_power(nbt: &mut Compound, value: f64) {
    nbt.insert(
        JavaString::from("internalCurrentPower"),
        Tag::Double(value.to_bits()),
    );
}

fn priority(nbt: &Compound) -> i32 {
    match nbt.get(&JavaString::from("priority")) {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    }
}

fn idle(data: &[i32]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let bits = (data[0] as u32 as u64) | ((data[1] as u32 as u64) << 32);
    let value = f64::from_bits(bits);
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn write_idle(data: &mut Vec<i32>, value: f64) {
    let bits = value.to_bits();
    data.resize(2, 0);
    data[0] = bits as u32 as i32;
    data[1] = (bits >> 32) as u32 as i32;
}

/// Integrated play sends the menu click to the server. The client's
/// `container` key is not the chest, so the extract has to happen here,
/// before `container_click` reads the slots.
pub(super) fn server_click(
    game: &mut Game<'_>,
    host: &crate::game::lan::LanHost,
    conn: u32,
    packet: &horizon_protocol::packets::play::ContainerClick,
) {
    if packet.button != 0 || !matches!(packet.input, 0 | 1) || packet.slot != WITHDRAW as i16 {
        return;
    }
    let Some(menu) = host.hub.guest(conn).and_then(|guest| guest.menu.as_ref()) else {
        return;
    };
    if menu.window != packet.container {
        return;
    }
    let horizon_client::lan::menu::Slots::Block(pos) = menu.slots else {
        return;
    };
    let Some(horizon_client::containers::BlockEntity::Mod(mut be)) =
        game.world.block_entities.take(&pos)
    else {
        return;
    };
    if be.id != BLOCK {
        game.world
            .block_entities
            .put(pos, horizon_client::containers::BlockEntity::Mod(be));
        return;
    }
    sync(game, [pos.0, pos.1, pos.2], &mut be, true);
    game.world
        .block_entities
        .put(pos, horizon_client::containers::BlockEntity::Mod(be));
    game.world.modified.insert((pos.0 >> 4, pos.2 >> 4));
}

