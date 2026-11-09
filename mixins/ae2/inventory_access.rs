// SPDX-License-Identifier: LGPL-3.0-or-later
// InternalInventory insertion (MIT; LICENSES/AE2-API-MIT), AppEngInternalInventory
// extraction and InternalInventoryItemHandler's wrapped-stack gate.
// Copyright TeamAppliedEnergistics; 2026 Horizon port contributors.
// Stack::clone retains a handle; copy_stack explicitly copies the source stack.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    SlotCheck,
    SlotRead,
}
pub trait Context {
    type Stack: Clone;
    fn size(&self) -> i32;
    fn stack(&mut self, slot: i32) -> Result<Self::Stack, Fault>;
    /// AppEng's override reads its backing list directly; it does not dispatch
    /// through the public getStackInSlot used by the interface and wrapper.
    fn stored_stack(&mut self, slot: i32) -> Result<Self::Stack, Fault>;
    fn empty_stack(&self) -> Self::Stack;
    fn empty(&self, stack: &Self::Stack) -> bool;
    fn count(&self, stack: &Self::Stack) -> i32;
    fn maximum(&self, stack: &Self::Stack) -> i32;
    fn copy_stack(&mut self, stack: &Self::Stack) -> Self::Stack;
    fn set_count(&mut self, stack: &Self::Stack, count: i32);
    fn slot_limit(&mut self, slot: i32) -> i32;
    fn valid(&mut self, slot: i32, stack: &Self::Stack) -> bool;
    fn same(&mut self, a: &Self::Stack, b: &Self::Stack) -> bool;
    fn allow_extract(&mut self, slot: i32, amount: i32) -> bool;
    /// AppEng's setter includes its guarded onChange/save notification.
    fn set_direct(&mut self, slot: i32, stack: Self::Stack);
    /// The explicit second notification on a full committed extraction is
    /// intentional: AppEng's setItemDirect already notifies once.
    fn notify(&mut self, slot: i32);
    fn wrapped(&self, stack: &Self::Stack) -> bool;
}
fn check<C: Context>(ctx: &C, slot: i32) -> Result<(), Fault> {
    if slot < 0 || slot >= ctx.size() {
        Err(Fault::SlotCheck)
    } else {
        Ok(())
    }
}
pub fn insert<C: Context>(
    ctx: &mut C,
    slot: i32,
    stack: C::Stack,
    simulate: bool,
) -> Result<C::Stack, Fault> {
    check(ctx, slot)?;
    if ctx.empty(&stack) || !ctx.valid(slot, &stack) {
        return Ok(stack);
    }
    let in_slot = ctx.stack(slot)?;
    let max_space = ctx.slot_limit(slot).min(ctx.maximum(&stack));
    let free_space = max_space.wrapping_sub(ctx.count(&in_slot));
    if free_space <= 0 || (!ctx.empty(&in_slot) && !ctx.same(&in_slot, &stack)) {
        return Ok(stack);
    }
    let amount = ctx.count(&stack).min(free_space);
    if !simulate {
        let new_item = ctx.copy_stack(if ctx.empty(&in_slot) {
            &stack
        } else {
            &in_slot
        });
        ctx.set_count(&new_item, ctx.count(&in_slot).wrapping_add(amount));
        ctx.set_direct(slot, new_item);
    }
    // A source callback can have changed the caller's retained stack. Re-read
    // it here, as the original does after setItemDirect returns.
    if free_space >= ctx.count(&stack) {
        Ok(ctx.empty_stack())
    } else {
        let rest = ctx.copy_stack(&stack);
        ctx.set_count(&rest, ctx.count(&rest).wrapping_sub(amount));
        Ok(rest)
    }
}
pub fn extract<C: Context>(
    ctx: &mut C,
    slot: i32,
    amount: i32,
    simulate: bool,
) -> Result<C::Stack, Fault> {
    check(ctx, slot)?;
    if !ctx.allow_extract(slot, amount) {
        return Ok(ctx.empty_stack());
    }
    let stack = ctx.stored_stack(slot)?;
    let take = ctx.count(&stack).min(amount.min(ctx.maximum(&stack)));
    if take <= 0 {
        return Ok(ctx.empty_stack());
    }
    if ctx.count(&stack) <= take {
        if simulate {
            Ok(ctx.copy_stack(&stack))
        } else {
            ctx.set_direct(slot, ctx.empty_stack());
            ctx.notify(slot);
            Ok(stack)
        }
    } else {
        let result = ctx.copy_stack(&stack);
        if !simulate {
            ctx.set_count(&stack, ctx.count(&stack).wrapping_sub(take));
            ctx.notify(slot);
        }
        ctx.set_count(&result, take);
        Ok(result)
    }
}
pub fn wrapped_extract<C: Context>(
    ctx: &mut C,
    slot: i32,
    amount: i32,
    simulate: bool,
) -> Result<C::Stack, Fault> {
    // InternalInventoryItemHandler checks the actual stack before the
    // inventory's argument check and filter, including for an invalid slot.
    let stack = ctx.stack(slot)?;
    if ctx.wrapped(&stack) {
        return Ok(ctx.empty_stack());
    }
    extract(ctx, slot, amount, simulate)
}
