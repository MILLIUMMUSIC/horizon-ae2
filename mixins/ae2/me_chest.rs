// SPDX-License-Identifier: LGPL-3.0-or-later
// MEChestBlockEntity's server handler/power/input/display/menu program.
// Copyright AlgorithmX2 / TeamAppliedEnergistics; see NOTICE.md.
// Inventory/registry/grid/world and stack identity are binding callbacks.
use super::ae2_energy::PowerEvent;
use super::ae2_storage_helper::{self as storage, Action, Statistic};

pub const BLOCK: &str = "ae2:chest";
pub const SLOTS: usize = 2;
pub const INPUT: usize = 0;
pub const CELL: usize = 1;
pub const LOCAL_CAPACITY: f64 = 500.0; // multiplied by CONFIG at construction

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Multiplier {
    One,
    Config,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    UnreadableCell,
    OutOfPower,
    Connected,
}
#[derive(Clone, Debug)]
pub struct State<C> {
    pub handler: Option<C>,
    pub cached: bool,
    pub fluid_handler: bool,
    pub idle: f64,
    pub priority: i32,
    /// Source CellState ordinal: ABSENT, EMPTY, NOT_EMPTY, TYPES_FULL, FULL.
    pub client_cell_state: u8,
    pub client_powered: bool,
    pub was_online: bool,
}
impl<C> Default for State<C> {
    fn default() -> Self {
        Self {
            handler: None,
            cached: false,
            fluid_handler: false,
            idle: 0.0,
            priority: 0,
            client_cell_state: 0,
            client_powered: false,
            was_online: false,
        }
    }
}
/// Stack clones retain object identity; shrink mutates the original handle.
/// set_input_direct must deliver the real inventory's guarded onChange/save
/// callbacks, including the source's single reentrant notification layer.
/// Cell callbacks access the actual underlying StorageCell, not grid storage.
pub trait Context {
    type Cell: Clone;
    type Stack: Clone;
    type Key;
    type Source: Clone;
    type Fluid;
    fn state(&self) -> &State<Self::Cell>;
    fn state_mut(&mut self) -> &mut State<Self::Cell>;
    fn client_side(&self) -> bool;
    fn level_present(&self) -> bool;
    fn cell_stack(&self) -> Self::Stack;
    fn input_stack(&self) -> Self::Stack;
    fn input_empty(&self) -> bool;
    fn empty_stack(&self) -> Self::Stack;
    fn empty(&self, stack: &Self::Stack) -> bool;
    fn count(&self, stack: &Self::Stack) -> i32;
    fn shrink(&mut self, stack: &Self::Stack, amount: i32);
    fn set_input_direct(&mut self, stack: Self::Stack);
    fn item_key(&self, stack: &Self::Stack) -> Option<Self::Key>;
    fn key_is_item(&self, key: &Self::Key) -> bool;
    fn amount_per_operation(&self, key: &Self::Key) -> i32;
    fn fluid_key(&self, fluid: &Self::Fluid) -> Option<Self::Key>;
    fn fluid_amount(&self, fluid: &Self::Fluid) -> i32;
    fn machine_source(&self) -> Self::Source;
    fn load_cell(&mut self, stack: &Self::Stack) -> Option<Self::Cell>;
    fn has_cell_handler(&mut self, stack: &Self::Stack) -> bool;
    fn cell_idle(&self, cell: &Self::Cell) -> f64;
    fn cell_status(&self, cell: &Self::Cell) -> u8;
    fn cell_insert(
        &mut self,
        cell: &Self::Cell,
        key: &Self::Key,
        amount: i64,
        mode: Action,
        source: &Self::Source,
    ) -> i64;
    fn cell_extract(
        &mut self,
        cell: &Self::Cell,
        key: &Self::Key,
        amount: i64,
        mode: Action,
        source: &Self::Source,
    ) -> i64;
    fn persist_cell(&mut self, cell: &Self::Cell);
    fn set_node_idle(&mut self, idle: f64);
    fn create_fluid_handler(&mut self);
    fn node_powered(&self) -> bool;
    fn node_online(&self) -> bool;
    fn grid_present(&self) -> bool;
    fn grid_powered(&self) -> bool;
    fn local_power(&self) -> f64;
    fn factor(&self, multiplier: Multiplier) -> f64;
    fn grid_extract(&mut self, amount: f64, mode: Action, multiplier: Multiplier) -> f64;
    fn local_extract(&mut self, amount: f64, mode: Action) -> f64;
    fn request_storage_update(&mut self);
    fn receive_power_event(&mut self);
    fn invalidate_capabilities(&mut self);
    fn mark_for_update(&mut self);
    fn block_entity_changed(&mut self);
    fn mount_cell(&mut self, cell: &Self::Cell, priority: i32);
    fn open_storage_menu(&mut self);
    fn open_cell_menu(&mut self);
}

pub fn update_handler<C: Context>(ctx: &mut C) {
    if ctx.state().cached {
        return;
    }
    ctx.state_mut().handler = None;
    ctx.state_mut().fluid_handler = false;
    let stack = ctx.cell_stack();
    if !ctx.empty(&stack) {
        ctx.state_mut().cached = true;
        if let Some(cell) = ctx.load_cell(&stack) {
            ctx.state_mut().idle = 1.0 + ctx.cell_idle(&cell);
            ctx.state_mut().handler = Some(cell);
            ctx.set_node_idle(ctx.state().idle);
            if ctx.state().handler.is_some() {
                ctx.create_fluid_handler();
                ctx.state_mut().fluid_handler = true;
            }
        }
    }
}
pub fn is_powered<C: Context>(ctx: &C) -> bool {
    if ctx.client_side() {
        ctx.state().client_powered
    } else {
        ctx.node_powered() || ctx.local_power() > 1.0
    }
}
pub fn link<C: Context>(ctx: &mut C) -> Link {
    update_handler(ctx);
    if ctx.state().handler.is_none() {
        Link::UnreadableCell
    } else if !is_powered(ctx) {
        Link::OutOfPower
    } else {
        Link::Connected
    }
}
pub fn cell_status<C: Context>(ctx: &mut C) -> u8 {
    if ctx.client_side() {
        return ctx.state().client_cell_state;
    }
    update_handler(ctx);
    let stack = ctx.cell_stack();
    let handled = ctx.has_cell_handler(&stack);
    ctx.state()
        .handler
        .as_ref()
        .filter(|_| handled)
        .map_or(0, |cell| ctx.cell_status(cell))
}
pub fn recalculate_display<C: Context>(ctx: &mut C) {
    let mut changed = false;
    let status = cell_status(ctx);
    if ctx.state().client_cell_state != status {
        ctx.state_mut().client_cell_state = status;
        changed = true;
    }
    let powered = is_powered(ctx);
    if ctx.state().client_powered != powered {
        ctx.state_mut().client_powered = powered;
        changed = true;
    }
    if changed {
        ctx.mark_for_update();
    }
}
/// AEBasePoweredBlockEntity's public multiplier wrapper around the Chest's
/// grid-first/local-remainder extraction. Re-read the factor before division:
/// the source PowerMultiplier enum's multiplier is a mutable public field.
pub fn extract_power<C: Context>(
    ctx: &mut C,
    amount: f64,
    mode: Action,
    multiplier: Multiplier,
) -> f64 {
    let amount = amount * ctx.factor(multiplier);
    let mut stash = 0.0;
    if ctx.grid_present() {
        stash = ctx.grid_extract(amount, mode, Multiplier::One);
        if stash >= amount {
            return stash / ctx.factor(multiplier);
        }
    }
    let result = ctx.local_extract(amount - stash, mode) + stash;
    result / ctx.factor(multiplier)
}
pub fn emit_power_event<C: Context>(ctx: &mut C, event: PowerEvent) {
    if event == PowerEvent::Receive {
        ctx.receive_power_event();
    } else {
        recalculate_display(ctx);
    }
}
pub fn server_tick<C: Context>(ctx: &mut C) {
    // Keep this grid snapshot for the powered check, as in the source.
    if !ctx.grid_present() || !ctx.grid_powered() {
        extract_power(ctx, ctx.state().idle, Action::Modulate, Multiplier::Config);
        recalculate_display(ctx);
    }
    if !ctx.input_empty() {
        store_input(ctx);
    }
}
pub fn monitor_insert<C: Context>(
    ctx: &mut C,
    cell: &C::Cell,
    key: &C::Key,
    amount: i64,
    mode: Action,
    source: &C::Source,
) -> i64 {
    let inserted = ctx.cell_insert(cell, key, amount, mode, source);
    if inserted > 0 && mode == Action::Modulate {
        recalculate_display(ctx);
    }
    inserted
}
pub fn monitor_extract<C: Context>(
    ctx: &mut C,
    cell: &C::Cell,
    key: &C::Key,
    amount: i64,
    mode: Action,
    source: &C::Source,
) -> i64 {
    let extracted = ctx.cell_extract(cell, key, amount, mode, source);
    if extracted > 0 && mode == Action::Modulate {
        recalculate_display(ctx);
    }
    extracted
}
struct InputTransfer<'a, C: Context> {
    ctx: &'a mut C,
    cell: C::Cell,
    source: C::Source,
}
impl<C: Context> storage::Context for InputTransfer<'_, C> {
    type Key = C::Key;
    fn extract(&mut self, key: &Self::Key, amount: i64, mode: Action) -> i64 {
        monitor_extract(self.ctx, &self.cell, key, amount, mode, &self.source)
    }
    fn insert(&mut self, key: &Self::Key, amount: i64, mode: Action) -> i64 {
        monitor_insert(self.ctx, &self.cell, key, amount, mode, &self.source)
    }
    fn power(&mut self, amount: f64, mode: Action) -> f64 {
        extract_power(self.ctx, amount, mode, Multiplier::Config)
    }
    fn amount_per_operation(&self, key: &Self::Key) -> i32 {
        self.ctx.amount_per_operation(key)
    }
    fn is_item(&self, key: &Self::Key) -> bool {
        self.ctx.key_is_item(key)
    }
    // This adapter is exclusively the source's MachineSource(this), whose
    // player() is empty. Native menu actions use their actual player source.
    fn statistic(&mut self, _: Statistic, _: i32) {}
}
pub fn store_input<C: Context>(ctx: &mut C) {
    if ctx.input_empty() {
        return;
    }
    update_handler(ctx);
    if let Some(cell) = ctx.state().handler.clone() {
        let stack = ctx.input_stack();
        if ctx.empty(&stack) {
            return;
        }
        let key = ctx.item_key(&stack).expect("nonempty source AEItemKey");
        let count = ctx.count(&stack);
        let source = ctx.machine_source();
        let inserted = storage::powered_insert(
            &mut InputTransfer { ctx, cell, source },
            &key,
            i64::from(count),
            Action::Modulate,
        );
        if inserted >= i64::from(ctx.count(&stack)) {
            let empty = ctx.empty_stack();
            ctx.set_input_direct(empty);
        } else {
            ctx.shrink(&stack, inserted as i32);
            ctx.set_input_direct(stack);
        }
    }
}
pub fn input_allowed<C: Context>(ctx: &mut C, stack: &C::Stack) -> bool {
    if is_powered(ctx) {
        update_handler(ctx);
        let Some(cell) = ctx.state().handler.clone() else {
            return false;
        };
        let Some(key) = ctx.item_key(stack) else {
            return false;
        };
        let count = i64::from(ctx.count(stack));
        let source = ctx.machine_source();
        return monitor_insert(ctx, &cell, &key, count, Action::Simulate, &source) > 0;
    }
    false
}
pub fn on_cell_slot_changed<C: Context>(ctx: &mut C) {
    ctx.state_mut().handler = None;
    ctx.state_mut().cached = false;
    ctx.request_storage_update();
    if ctx.level_present() {
        ctx.invalidate_capabilities();
        ctx.mark_for_update();
    }
}
pub fn on_input_slot_changed<C: Context>(ctx: &mut C) {
    if !ctx.input_empty() {
        store_input(ctx);
    }
}
pub fn set_priority<C: Context>(ctx: &mut C, priority: i32) {
    ctx.state_mut().priority = priority;
    ctx.state_mut().handler = None;
    ctx.state_mut().cached = false;
    ctx.request_storage_update();
}
pub fn on_node_state_changed<C: Context>(ctx: &mut C) {
    let online = ctx.node_online();
    if ctx.state().was_online != online {
        ctx.state_mut().was_online = online;
        ctx.request_storage_update();
        recalculate_display(ctx);
    }
}
pub fn mount<C: Context>(ctx: &mut C) {
    if ctx.node_online() {
        update_handler(ctx);
        if let Some(cell) = ctx.state().handler.clone() {
            ctx.mount_cell(&cell, ctx.state().priority);
        }
    }
}
pub fn on_cell_content_changed<C: Context>(ctx: &mut C) {
    if let Some(cell) = ctx.state().handler.clone() {
        ctx.persist_cell(&cell);
    }
    ctx.block_entity_changed();
}
pub fn open_gui<C: Context>(ctx: &mut C) -> bool {
    update_handler(ctx);
    if ctx.state().handler.is_some() {
        let stack = ctx.cell_stack();
        if ctx.has_cell_handler(&stack) {
            ctx.open_storage_menu();
            return true;
        }
    }
    false
}
pub fn open_cell_inventory<C: Context>(ctx: &mut C) {
    ctx.open_cell_menu();
}

/// The fluid handler is an insertion-only virtual sink, with no exposed stored
/// fluid. Its two capacity/validity interfaces deliberately differ: getCapacity
/// and isFluidValid depend on the current cell, indexed tank queries do not.
pub fn fluid_capacity<C: Context>(ctx: &C) -> i32 {
    if ctx.state().handler.is_some() {
        1000
    } else {
        0
    }
}
pub fn fluid_valid<C: Context>(ctx: &C) -> bool {
    ctx.state().handler.is_some()
}
pub fn tank_capacity(tank: i32) -> i32 {
    if tank == 0 { 1000 } else { 0 }
}
pub fn tank_valid(tank: i32) -> bool {
    tank == 0
}
/// The caller must have the actual bound FluidHandler. Front-side capability
/// admission and retained capability lifetime belong to the world adapter.
pub fn fill_fluid<C: Context>(ctx: &mut C, fluid: &C::Fluid, mode: Action) -> i32 {
    update_handler(ctx);
    if let Some(cell) = ctx.state().handler.clone()
        && let Some(key) = ctx.fluid_key(fluid)
    {
        let amount = i64::from(ctx.fluid_amount(fluid));
        let source = ctx.machine_source();
        return storage::powered_insert(&mut InputTransfer { ctx, cell, source }, &key, amount, mode)
            as i32;
    }
    0
}
/// Source front-side capabilities expose the cell slot, other sides the input.
/// The menu opens the cell's own storage on top, and a separate cell-slot menu
/// on other faces. No withdrawal stack exists in either saved inventory.
pub fn inventory_slot(front: bool) -> usize {
    if front { CELL } else { INPUT }
}
