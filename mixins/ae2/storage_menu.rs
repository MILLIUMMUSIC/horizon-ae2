// SPDX-License-Identifier: LGPL-3.0-or-later
// MEStorageMenu's server interaction program. World/player/slot/container-item
// adapters must supply real state, identity and callbacks; this is not a menu UI.
use super::ae2_incremental_update::Updates;
use super::ae2_me_interaction::InventoryAction as Action;
use super::ae2_storage_helper::{self as storage, Action as Mode, Context as StorageContext};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    CouldNotExtractFluid,
    FluidExtractionChanged,
    CouldNotInsertFluid,
}
/// Source filling with a null key and a real container context throws before
/// transfer. Preserve that fault boundary as a typed rejection for the native
/// caller, so a malformed request cannot unwind a Rust server thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NullFluidKey;

/// Stack/slot/container handles retain object identity. Cloning a handle is
/// not ItemStack.copy; copy_stack is the explicit source copy operation.
pub trait Context: StorageContext {
    type Stack: Clone;
    type Item: PartialEq;
    type Slot;
    type Container;
    fn connected(&mut self) -> bool;
    fn has_locator(&self) -> bool;
    fn open_craft_amount(&mut self, key: &Self::Key);
    fn carried(&self) -> Self::Stack;
    fn set_carried(&mut self, stack: Self::Stack);
    fn empty_stack(&self) -> Self::Stack;
    fn is_empty(&self, stack: &Self::Stack) -> bool;
    fn count(&self, stack: &Self::Stack) -> i32;
    fn max_stack(&self, stack: &Self::Stack) -> i32;
    fn item(&self, stack: &Self::Stack) -> Self::Item;
    fn key_of_stack(&self, stack: &Self::Stack) -> Option<Self::Key>;
    fn key_matches(&self, key: &Self::Key, stack: &Self::Stack) -> bool;
    fn key_max_stack(&self, key: &Self::Key) -> i32;
    fn to_stack(&mut self, key: &Self::Key, count: i32) -> Self::Stack;
    fn copy_stack(&mut self, stack: &Self::Stack) -> Self::Stack;
    fn set_count(&mut self, stack: &Self::Stack, count: i32);
    fn grow(&mut self, stack: &Self::Stack, count: i32);
    fn shrink(&mut self, stack: &Self::Stack, count: i32);
    fn creative(&self) -> bool;
    fn player_items_len(&self) -> usize;
    fn add_to_player(&mut self, stack: &Self::Stack) -> bool;
    fn quick_move_destinations(&mut self, stack: &Self::Stack) -> Vec<Self::Slot>;
    fn placeable_amount(&mut self, slot: &Self::Slot, key: &Self::Key) -> i32;
    fn slot_item(&self, slot: &Self::Slot) -> Self::Stack;
    fn slot_set_by_player(&mut self, slot: &Self::Slot, stack: Self::Stack);
    fn has_fluid_bucket(&self, key: &Self::Key) -> bool;
    fn is_bucket(&self, stack: &Self::Stack) -> bool;
    fn bucket_key(&self) -> Self::Key;
    fn default_bucket(&mut self) -> Self::Stack;
    fn amount_per_unit(&self, key: &Self::Key) -> i32;
    fn filling_context(&mut self, key: Option<&Self::Key>) -> Option<Self::Container>;
    fn emptying_context(&mut self) -> Option<Self::Container>;
    fn container_content(&mut self, container: &Self::Container) -> Option<(Self::Key, i64)>;
    fn container_insert(
        &mut self,
        container: &Self::Container,
        key: &Self::Key,
        amount: i64,
        mode: Mode,
    ) -> i64;
    fn container_extract(
        &mut self,
        container: &Self::Container,
        key: &Self::Key,
        amount: i64,
        mode: Mode,
    ) -> i64;
    fn play_fill_sound(&mut self, container: &Self::Container, key: &Self::Key);
    fn play_empty_sound(&mut self, container: &Self::Container, key: &Self::Key);
    fn error(&mut self, error: Error);
    fn unhandled(&mut self, action: Action);
}

/// Server half of handleInteraction. The packet's player/window identity gate
/// must already have run. A serial of -1 targets an empty virtual slot; every
/// other serial must resolve to the retained full server key for this menu.
pub fn interact<C: Context>(
    ctx: &mut C,
    updates: &Updates<C::Key>,
    serial: i64,
    action: Action,
) -> Result<(), NullFluidKey>
where
    C::Key: Ord + Clone,
{
    if !ctx.connected() {
        return Ok(());
    }
    if serial == -1 {
        network_interaction(ctx, None, action)?;
    } else if let Some(key) = updates.by_serial(serial) {
        network_interaction(ctx, Some(key), action)?;
    }
    Ok(())
}

pub fn network_interaction<C: Context>(
    ctx: &mut C,
    key: Option<&C::Key>,
    action: Action,
) -> Result<(), NullFluidKey> {
    if !ctx.connected() {
        return Ok(());
    }
    if action == Action::AutoCraft {
        if ctx.has_locator()
            && let Some(key) = key
        {
            ctx.open_craft_amount(key);
        }
        return Ok(());
    }
    // The source continues through the null/item-key switch after attempting
    // these operations; do not return early merely because an action is fluid.
    match action {
        Action::FillItem => try_fill_container(ctx, key, false, false)?,
        Action::FillItemMoveToPlayer => try_fill_container(ctx, key, true, false)?,
        Action::FillEntireItem => try_fill_container(ctx, key, false, true)?,
        Action::FillEntireItemMoveToPlayer => try_fill_container(ctx, key, true, true)?,
        Action::EmptyItem => empty_container(ctx, false),
        Action::EmptyEntireItem => empty_container(ctx, true),
        _ => {}
    }
    let Some(key) = key else {
        match action {
            Action::SplitOrPlaceSingle | Action::RollDown => put_carried(ctx, true),
            Action::PickupOrSetDown => put_carried(ctx, false),
            _ => {}
        }
        return Ok(());
    };
    if !ctx.is_item(key) {
        return Ok(());
    }
    match action {
        Action::ShiftClick => {
            move_one_stack(ctx, key);
        }
        Action::RollDown => {
            let carried = ctx.carried();
            if !ctx.is_empty(&carried)
                && let Some(what) = ctx.key_of_stack(&carried)
            {
                let inserted = storage::powered_insert(ctx, &what, 1, Mode::Modulate);
                if inserted > 0 {
                    let current = ctx.carried();
                    ctx.shrink(&current, 1);
                }
            }
        }
        Action::RollUp | Action::PickupSingle => {
            let carried = ctx.carried();
            if !ctx.is_empty(&carried)
                && (ctx.count(&carried) >= ctx.max_stack(&carried)
                    || !ctx.key_matches(key, &carried))
            {
                return Ok(());
            }
            let extracted = storage::powered_extraction(ctx, key, 1, Mode::Modulate);
            if extracted > 0 {
                if ctx.is_empty(&carried) {
                    let stack = ctx.to_stack(key, 1);
                    ctx.set_carried(stack);
                } else {
                    ctx.grow(&carried, 1);
                }
            }
        }
        Action::PickupOrSetDown => {
            if !ctx.is_empty(&ctx.carried()) {
                put_carried(ctx, false);
            } else {
                let extracted = storage::powered_extraction(
                    ctx,
                    key,
                    i64::from(ctx.key_max_stack(key)),
                    Mode::Modulate,
                );
                let stack = if extracted > 0 {
                    ctx.to_stack(key, extracted as i32)
                } else {
                    ctx.empty_stack()
                };
                ctx.set_carried(stack);
            }
        }
        Action::SplitOrPlaceSingle => {
            if !ctx.is_empty(&ctx.carried()) {
                put_carried(ctx, true);
            } else {
                let mut extracted =
                    ctx.extract(key, i64::from(ctx.key_max_stack(key)), Mode::Simulate);
                if extracted > 0 {
                    extracted = extracted.wrapping_add(1) >> 1;
                    extracted = storage::powered_extraction(ctx, key, extracted, Mode::Modulate);
                }
                let stack = if extracted > 0 {
                    ctx.to_stack(key, extracted as i32)
                } else {
                    ctx.empty_stack()
                };
                ctx.set_carried(stack);
            }
        }
        Action::CreativeDuplicate => {
            if ctx.creative() {
                let stack = ctx.to_stack(key, 1);
                let count = ctx.max_stack(&stack);
                ctx.set_count(&stack, count);
                ctx.set_carried(stack);
            }
        }
        Action::MoveRegion => {
            let count = ctx.player_items_len();
            for _ in 0..count {
                if !move_one_stack(ctx, key) {
                    break;
                }
            }
        }
        _ => ctx.unhandled(action),
    }
    Ok(())
}

pub fn put_carried<C: Context>(ctx: &mut C, single: bool) {
    let held = ctx.carried();
    let Some(key) = ctx.key_of_stack(&held) else {
        return;
    };
    let amount = if single { 1 } else { ctx.count(&held) };
    let inserted = storage::powered_insert(ctx, &key, i64::from(amount), Mode::Modulate);
    // Platform.getInsertionRemainder returns EMPTY, or a copy with the Java
    // narrowed remainder; it never shrinks the retained original held object.
    let count = ctx.count(&held);
    let remainder = if inserted >= i64::from(count) {
        ctx.empty_stack()
    } else {
        let count = i64::from(count).wrapping_sub(inserted) as i32;
        if count <= 0 {
            ctx.empty_stack()
        } else {
            let copy = ctx.copy_stack(&held);
            ctx.set_count(&copy, count);
            copy
        }
    };
    ctx.set_carried(remainder);
}

pub fn move_one_stack<C: Context>(ctx: &mut C, key: &C::Key) -> bool {
    if ctx.extract(key, i64::from(ctx.key_max_stack(key)), Mode::Simulate) <= 0 {
        return false;
    }
    let sample = ctx.to_stack(key, 1);
    let slots = ctx.quick_move_destinations(&sample);
    for slot in slots {
        let amount = ctx.placeable_amount(&slot, key);
        if amount <= 0 {
            continue;
        }
        let extracted = storage::powered_extraction(ctx, key, i64::from(amount), Mode::Modulate);
        if extracted == 0 {
            return false;
        }
        let current = ctx.slot_item(&slot);
        let stack = if !ctx.is_empty(&current) {
            let count = ctx.count(&current).wrapping_add(extracted as i32);
            let copy = ctx.copy_stack(&current);
            ctx.set_count(&copy, count);
            copy
        } else {
            ctx.to_stack(key, extracted as i32)
        };
        ctx.slot_set_by_player(&slot, stack);
        return true;
    }
    false
}

pub fn try_fill_container<C: Context>(
    ctx: &mut C,
    key: Option<&C::Key>,
    move_to_player: bool,
    all: bool,
) -> Result<(), NullFluidKey> {
    let mut grabbed = false;
    if ctx.is_empty(&ctx.carried()) && key.is_some_and(|k| ctx.has_fluid_bucket(k)) {
        let bucket = ctx.bucket_key();
        if ctx.extract(&bucket, 1, Mode::Modulate) >= 1 {
            let stack = ctx.default_bucket();
            ctx.set_carried(stack);
            grabbed = true;
        }
    }
    let before = ctx.item(&ctx.carried());
    fill_container(ctx, key, all)?;
    let current = ctx.carried();
    if grabbed
        && ctx.is_bucket(&current)
        && let Some(key) = ctx.key_of_stack(&current)
    {
        let inserted = ctx.insert(&key, i64::from(ctx.count(&current)), Mode::Modulate);
        let copy = ctx.copy_stack(&current);
        ctx.shrink(
            &copy,
            inserted.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        );
        ctx.set_carried(copy);
    }
    let current = ctx.carried();
    if move_to_player && ctx.item(&current) != before && ctx.add_to_player(&current) {
        let empty = ctx.empty_stack();
        ctx.set_carried(empty);
    }
    Ok(())
}
fn fill_container<C: Context>(
    ctx: &mut C,
    key: Option<&C::Key>,
    all: bool,
) -> Result<(), NullFluidKey> {
    let Some(container) = ctx.filling_context(key) else {
        return Ok(());
    };
    let key = key.ok_or(NullFluidKey)?;
    let amount = if all {
        i64::MAX
    } else {
        i64::from(ctx.amount_per_unit(key))
    };
    let mut filled = false;
    for _ in 0..if all { 256 } else { 1 } {
        let can_pull = storage::powered_extraction(ctx, key, amount, Mode::Simulate);
        if can_pull <= 0 {
            break;
        }
        let allowed = ctx.container_insert(&container, key, can_pull, Mode::Simulate);
        if allowed == 0 {
            break;
        }
        let extracted = storage::powered_extraction(ctx, key, allowed, Mode::Modulate);
        if extracted <= 0 {
            ctx.error(Error::CouldNotExtractFluid);
            break;
        }
        if ctx.container_insert(&container, key, extracted, Mode::Modulate) == 0 {
            break;
        }
        filled = true;
    }
    if filled {
        ctx.play_fill_sound(&container, key);
    }
    Ok(())
}
pub fn empty_container<C: Context>(ctx: &mut C, all: bool) {
    let Some(container) = ctx.emptying_context() else {
        return;
    };
    let Some((key, content)) = ctx.container_content(&container) else {
        return;
    };
    if content == 0 {
        return;
    }
    let amount = if all {
        i64::MAX
    } else {
        i64::from(ctx.amount_per_unit(&key))
    };
    let mut emptied = false;
    for _ in 0..if all { 256 } else { 1 } {
        let can_extract = ctx.container_extract(&container, &key, amount, Mode::Simulate);
        if can_extract <= 0 {
            break;
        }
        let allowed = storage::powered_insert(ctx, &key, can_extract, Mode::Simulate);
        if allowed <= 0 {
            break;
        }
        let extracted = ctx.container_extract(&container, &key, allowed, Mode::Modulate);
        if extracted != allowed {
            ctx.error(Error::FluidExtractionChanged);
            break;
        }
        if storage::powered_insert(ctx, &key, extracted, Mode::Modulate) != extracted {
            ctx.error(Error::CouldNotInsertFluid);
            break;
        }
        emptied = true;
    }
    if emptied {
        ctx.play_empty_sound(&container, &key);
    }
}
