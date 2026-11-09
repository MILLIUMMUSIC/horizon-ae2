// SPDX-License-Identifier: LGPL-3.0-or-later
// Retained native ME Chest hosts. Rules live in ae2_me_chest; the cell and
// owned power are shared by the host and its grid provider, never copied stores.
use super::{
    Game,
    ae2_backend::CellBackend,
    ae2_cells as codec,
    ae2_drive_glue::{Service, Source},
    ae2_energy::PowerEvent,
    ae2_keys::{Key, KeyType},
    ae2_me_chest::{self as rules, Context, Multiplier},
    ae2_mounts::{Counts, Storage, add_count},
    ae2_network::{ChestPower, Kind, Network, Pos},
    ae2_power::SharedPower,
    ae2_service::{StorageMounts, StorageProvider},
    ae2_storage::CellState,
    ae2_storage_helper::Action,
};
use horizon_client::{
    containers::{BlockEntity, ModBlockEntity},
    items::ItemStack,
};
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

const MAGIC: i32 = 0x41453243;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
type Stack = Arc<Mutex<Option<ItemStack>>>;
type Cell = Arc<Mutex<CellHost>>;
fn stack(item: Option<ItemStack>) -> Stack {
    Arc::new(Mutex::new(item))
}
fn value(s: &Stack) -> Option<ItemStack> {
    s.lock()
        .unwrap()
        .as_ref()
        .filter(|s| s.count > 0 && &*s.item != "minecraft:air")
        .cloned()
}
fn id(be: &ModBlockEntity) -> Option<u64> {
    if be.data.len() < 3 || be.data[0] != MAGIC {
        return None;
    }
    Some(u64::from(be.data[1] as u32) | (u64::from(be.data[2] as u32) << 32))
}
fn identify(be: &mut ModBlockEntity) -> u64 {
    if let Some(id) = id(be) {
        return id;
    }
    let id = NEXT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .expect("AE2 Chest identity exhausted");
    be.data = vec![MAGIC, id as u32 as i32, (id >> 32) as u32 as i32, 0, 0];
    id
}
fn number(tag: Option<&Tag>) -> f64 {
    match tag {
        Some(Tag::Byte(v)) => f64::from(*v),
        Some(Tag::Short(v)) => f64::from(*v),
        Some(Tag::Int(v)) => f64::from(*v),
        Some(Tag::Long(v)) => *v as f64,
        Some(Tag::Float(v)) => f64::from(f32::from_bits(*v)),
        Some(Tag::Double(v)) => f64::from_bits(*v),
        _ => 0.,
    }
}
fn priority(tag: Option<&Tag>) -> i32 {
    match tag {
        Some(Tag::Long(v)) => *v as i32,
        Some(Tag::Float(v)) => super::ae2_reporting::floor(f64::from(f32::from_bits(*v))),
        Some(Tag::Double(v)) => super::ae2_reporting::floor(f64::from_bits(*v)),
        _ => number(tag) as i32,
    }
}
fn status(cell: &CellBackend<Key>) -> u8 {
    match cell.state() {
        CellState::Empty => 1,
        CellState::NotEmpty => 2,
        CellState::TypesFull => 3,
        CellState::Full => 4,
    }
}
struct CellHost {
    backend: CellBackend<Key>,
    item: Stack,
    dirty: bool,
    blink: bool,
}
impl CellHost {
    fn persist(&mut self) -> bool {
        let mut item = self.item.lock().unwrap();
        let Some(stack) = item.as_mut() else {
            return false;
        };
        if codec::persist_backend(stack, &self.backend).is_err() {
            return false;
        }
        self.dirty = false;
        true
    }
    fn transfer(&mut self, key: &Key, amount: u64, commit: bool, insert: bool) -> u64 {
        let result = if insert {
            self.backend.insert(key.clone(), amount, commit)
        } else {
            self.backend.extract(key, amount, commit)
        };
        if commit && result > 0 {
            self.dirty = true;
            self.blink = true;
        }
        result
    }
}
impl Storage<Key, Source> for CellHost {
    fn preferred(&mut self, key: &Key, _: &Source) -> bool {
        // ChestMonitorHandler delegates to the actual cell; it does not use
        // DriveWatcher's existing-stack preference for ordinary basic cells.
        match &self.backend {
            CellBackend::Creative(cell) => cell.preferred(key),
            _ => false,
        }
    }
    fn insert(&mut self, key: &Key, amount: u64, commit: bool, _: &Source) -> u64 {
        let result = self.transfer(key, amount, commit, true);
        if commit && result > 0 {
            self.persist();
        }
        result
    }
    fn extract(&mut self, key: &Key, amount: u64, commit: bool, _: &Source) -> u64 {
        let result = self.transfer(key, amount, commit, false);
        if commit && result > 0 {
            self.persist();
        }
        result
    }
    fn available(&mut self, out: &mut Counts<Key>) {
        for (key, amount) in self.backend.stacks() {
            add_count(out, key.clone(), *amount as i64);
        }
    }
}
#[derive(Default)]
struct Provider {
    cell: Option<Cell>,
    priority: i32,
}
impl StorageProvider<Key, Source> for Provider {
    fn mount(&mut self, mounts: &mut StorageMounts<'_, Key, Source>) -> Result<(), &'static str> {
        if let Some(cell) = &self.cell {
            mounts.mount(cell.clone(), self.priority)?;
        }
        Ok(())
    }
}
struct Host {
    token: u64,
    state: rules::State<Cell>,
    items: [Stack; 2],
    echo: Vec<Option<ItemStack>>,
    inv: Option<Tag>,
    observed: bool,
    blocked: bool,
    power: SharedPower,
    power_echo: Option<Tag>,
    profile: ChestPower,
    provider: Arc<Mutex<Provider>>,
    registration: Option<(u64, u64)>,
    refresh: bool,
    notify: [bool; 2],
    priority_changed: bool,
    notifying: bool,
    node_idle: Option<f64>,
    dirty: bool,
    mark: bool,
    light_on: bool,
}
impl Host {
    fn new(token: u64) -> Self {
        let profile = ChestPower::new(1., 0).unwrap();
        // AEBasePoweredBlockEntity's store exists before its managed node.
        let power = SharedPower::new(0., 10000.);
        power.set_maximum(profile.capacity());
        Self {
            token,
            state: rules::State::default(),
            items: [stack(None), stack(None)],
            echo: vec![None; 2],
            inv: None,
            observed: false,
            blocked: false,
            power,
            power_echo: None,
            profile,
            provider: Arc::default(),
            registration: None,
            refresh: true,
            notify: [false; 2],
            priority_changed: false,
            notifying: false,
            node_idle: None,
            dirty: false,
            mark: false,
            light_on: false,
        }
    }
    fn observe(&mut self, items: &horizon_client::items::Items, be: &ModBlockEntity) {
        if be.items.iter().skip(2).flatten().any(|s| s.count > 0) {
            self.blocked = true;
            self.refresh = true;
            return;
        }
        let mut native = be.items.iter().take(2).cloned().collect::<Vec<_>>();
        native.resize(2, None);
        let inv = be.nbt.get(&JavaString::from("inv")).cloned();
        let power = be
            .nbt
            .get(&JavaString::from("internalCurrentPower"))
            .cloned();
        let amount = number(power.as_ref());
        if !amount.is_finite() {
            self.blocked = true;
            self.refresh = true;
            return;
        }
        if !self.observed || self.echo != native || self.inv != inv || self.blocked {
            match super::ae2_inventory_codec::choose_slots(
                inv.as_ref(),
                self.inv.as_ref(),
                &native,
                self.observed,
                self.blocked,
                2,
            ) {
                Ok(chosen) => {
                    // Input must resolve before the rule core's nonempty key
                    // contract is used. Invalid originals remain untouched.
                    if chosen[0]
                        .as_ref()
                        .is_some_and(|s| codec::item_key(s, items).is_err())
                    {
                        self.blocked = true;
                        self.refresh = true;
                        return;
                    }
                    let original_change = !self.observed || inv != self.inv;
                    for (i, item) in chosen.into_iter().enumerate() {
                        if value(&self.items[i]) != item {
                            self.items[i] = stack(item);
                            if self.observed {
                                if i == 1 {
                                    self.state.handler = None;
                                    self.state.cached = false;
                                    self.refresh = true;
                                }
                                if !original_change {
                                    self.notify[i] = true;
                                }
                            }
                        }
                    }
                    self.blocked = false;
                }
                Err(_) => {
                    self.blocked = true;
                    self.refresh = true;
                    return;
                }
            }
            self.echo = native;
            self.inv = inv;
        }
        if !self.observed || self.power_echo != power {
            self.power.set_stored(amount);
            self.power_echo = power;
        }
        let p = priority(be.nbt.get(&JavaString::from("priority")));
        if !self.observed {
            self.state.priority = p;
            self.profile = self.profile.with_priority(p);
        } else if self.state.priority != p {
            self.state.priority = p;
            self.priority_changed = true;
        }
        self.observed = true;
    }
    fn flush(&mut self, be: &mut ModBlockEntity) -> bool {
        if self.blocked || id(be) != Some(self.token) {
            return false;
        }
        if let Some(cell) = &self.state.handler {
            let mut cell = cell.lock().unwrap();
            if cell.dirty && !cell.persist() {
                return false;
            }
        }
        let items = self.items.iter().map(value).collect::<Vec<_>>();
        let inv = super::ae2_inventory_codec::write_slots(
            items.iter().map(Option::as_ref),
            self.inv.as_ref(),
        );
        let power = Tag::Double(self.power.amount().to_bits());
        let changed = self.dirty
            || be.items.get(..2) != Some(items.as_slice())
            || be.nbt.get(&JavaString::from("inv")) != Some(&inv)
            || be.nbt.get(&JavaString::from("internalCurrentPower")) != Some(&power);
        be.items.resize(be.items.len().max(2), None);
        be.items[..2].clone_from_slice(&items);
        be.nbt.insert(JavaString::from("inv"), inv.clone());
        be.nbt
            .insert(JavaString::from("internalCurrentPower"), power.clone());
        be.data.resize(5, 0);
        be.data[3] = i32::from(self.state.client_cell_state);
        be.data[4] = i32::from(self.state.client_powered);
        self.echo = items;
        self.inv = Some(inv);
        self.power_echo = Some(power);
        self.dirty = false;
        changed
    }
}
struct Frame<'a, 'g> {
    host: &'a mut Host,
    game: &'a Game<'g>,
    network: &'a mut Network,
    pos: Pos,
    menus: Vec<bool>,
}
impl Frame<'_, '_> {
    fn events(&mut self) {
        for event in self.host.power.take_events() {
            rules::emit_power_event(self, event);
        }
    }
    fn notifications(&mut self) {
        self.events();
        if std::mem::take(&mut self.host.priority_changed) {
            let p = self.host.state.priority;
            rules::set_priority(self, p);
        }
        if std::mem::take(&mut self.host.notify[1]) {
            rules::on_cell_slot_changed(self);
        }
        if std::mem::take(&mut self.host.notify[0]) {
            rules::on_input_slot_changed(self);
        }
        if let Some(cell) = self.host.state.handler.clone() {
            let blink = std::mem::take(&mut cell.lock().unwrap().blink);
            if blink {
                rules::recalculate_display(self);
            }
        }
    }
}
impl Context for Frame<'_, '_> {
    type Cell = Cell;
    type Stack = Stack;
    type Key = Key;
    type Source = Source;
    type Fluid = (Key, i32);
    fn state(&self) -> &rules::State<Cell> {
        &self.host.state
    }
    fn state_mut(&mut self) -> &mut rules::State<Cell> {
        &mut self.host.state
    }
    fn client_side(&self) -> bool {
        false
    }
    fn level_present(&self) -> bool {
        true
    }
    fn cell_stack(&self) -> Stack {
        self.host.items[1].clone()
    }
    fn input_stack(&self) -> Stack {
        self.host.items[0].clone()
    }
    fn input_empty(&self) -> bool {
        value(&self.host.items[0]).is_none()
    }
    fn empty_stack(&self) -> Stack {
        stack(None)
    }
    fn empty(&self, s: &Stack) -> bool {
        value(s).is_none()
    }
    fn count(&self, s: &Stack) -> i32 {
        value(s).map_or(0, |s| s.count as i32)
    }
    fn shrink(&mut self, s: &Stack, n: i32) {
        if let Some(item) = s.lock().unwrap().as_mut() {
            item.count = (item.count as i32).wrapping_sub(n).max(0) as u32;
        }
    }
    fn set_input_direct(&mut self, s: Stack) {
        self.host.items[0] = s;
        self.host.dirty = true;
        if !self.host.notifying {
            self.host.notifying = true;
            rules::on_input_slot_changed(self);
            self.host.notifying = false;
        }
    }
    fn item_key(&self, s: &Stack) -> Option<Key> {
        codec::item_key(value(s).as_ref()?, self.game.items()).ok()
    }
    fn key_is_item(&self, key: &Key) -> bool {
        key.kind() == KeyType::Item
    }
    fn amount_per_operation(&self, key: &Key) -> i32 {
        if key.kind() == KeyType::Fluid { 125 } else { 1 }
    }
    fn fluid_key(&self, fluid: &(Key, i32)) -> Option<Key> {
        (fluid.0.kind() == KeyType::Fluid && fluid.1 > 0).then(|| fluid.0.clone())
    }
    fn fluid_amount(&self, fluid: &(Key, i32)) -> i32 {
        fluid.1
    }
    fn machine_source(&self) -> Source {
        Source {
            player: None,
            machine_node: self.network.node_service(self.pos).map(|r| r.1),
        }
    }
    fn load_cell(&mut self, s: &Stack) -> Option<Cell> {
        let item = value(s)?;
        let resolve = |kind, id: &str| codec::resolve_key(self.game.items(), kind, id);
        let backend =
            if let Some(allowed) = super::ae2_tool_glue::accepted(self.game, &item.item) {
                codec::load_restricted_backend(&item, &resolve, allowed)
            } else {
                codec::load_backend(&item, &resolve)
            }
            .ok()?;
        Some(Arc::new(Mutex::new(CellHost {
            backend,
            item: s.clone(),
            dirty: false,
            blink: false,
        })))
    }
    fn has_cell_handler(&mut self, s: &Stack) -> bool {
        value(s).is_some_and(|s| horizon_client::lan::menu::ae2_slot_rules::handled_cell(&s.item))
    }
    fn cell_idle(&self, c: &Cell) -> f64 {
        c.lock().unwrap().backend.idle_drain()
    }
    fn cell_status(&self, c: &Cell) -> u8 {
        status(&c.lock().unwrap().backend)
    }
    fn cell_insert(&mut self, c: &Cell, key: &Key, n: i64, mode: Action, _: &Source) -> i64 {
        let result = if n > 0 {
            c.lock()
                .unwrap()
                .transfer(key, n as u64, mode == Action::Modulate, true) as i64
        } else {
            0
        };
        if result > 0 && mode == Action::Modulate {
            rules::on_cell_content_changed(self);
        }
        result
    }
    fn cell_extract(&mut self, c: &Cell, key: &Key, n: i64, mode: Action, _: &Source) -> i64 {
        let result = if n > 0 {
            c.lock()
                .unwrap()
                .transfer(key, n as u64, mode == Action::Modulate, false) as i64
        } else {
            0
        };
        if result > 0 && mode == Action::Modulate {
            rules::on_cell_content_changed(self);
        }
        result
    }
    fn persist_cell(&mut self, c: &Cell) {
        if !c.lock().unwrap().persist() {
            self.host.dirty = true;
        }
    }
    fn set_node_idle(&mut self, idle: f64) {
        self.host.node_idle = Some(idle);
        let _ = self.network.set_idle(self.pos, idle);
    }
    fn create_fluid_handler(&mut self) {}
    fn node_powered(&self) -> bool {
        self.network.is_powered(self.pos)
    }
    fn node_online(&self) -> bool {
        !self.host.blocked && self.network.is_online(self.pos)
    }
    fn grid_present(&self) -> bool {
        self.network.node_service(self.pos).is_some()
    }
    fn grid_powered(&self) -> bool {
        self.network.is_powered(self.pos)
    }
    fn local_power(&self) -> f64 {
        self.host.power.amount()
    }
    fn factor(&self, _: Multiplier) -> f64 {
        1.
    } // Common config loading remains separate.
    fn grid_extract(&mut self, n: f64, mode: Action, _: Multiplier) -> f64 {
        self.network
            .extract(self.pos, n, 1., mode == Action::Modulate)
    }
    fn local_extract(&mut self, n: f64, mode: Action) -> f64 {
        let result = self.host.power.extract(n, mode == Action::Modulate);
        self.events();
        result
    }
    fn request_storage_update(&mut self) {
        self.host.refresh = true;
    }
    fn receive_power_event(&mut self) {
        self.network
            .owner_storage_event(self.pos, PowerEvent::Receive);
    }
    fn invalidate_capabilities(&mut self) {
        self.host.refresh = true;
    }
    fn mark_for_update(&mut self) {
        let status = rules::cell_status(self);
        let powered = rules::is_powered(self);
        self.host.light_on = powered && status != 0;
        self.host.mark = true;
    }
    fn block_entity_changed(&mut self) {
        self.host.dirty = true;
    }
    fn mount_cell(&mut self, cell: &Cell, priority: i32) {
        let mut p = self.host.provider.lock().unwrap();
        p.cell = Some(cell.clone());
        p.priority = priority;
    }
    fn open_storage_menu(&mut self) {
        self.menus.push(true);
    }
    fn open_cell_menu(&mut self) {
        self.menus.push(false);
    }
}
#[derive(Default)]
pub(super) struct Chests {
    hosts: BTreeMap<Pos, Host>,
}
impl Chests {
    pub(super) fn kind(&self, pos: Pos) -> Option<Kind> {
        self.hosts
            .get(&pos)
            .filter(|h| !h.blocked)
            .map(|h| Kind::Chest(h.profile))
    }
    pub(super) fn forget(
        &mut self,
        pos: Pos,
        network: &mut Network,
        services: &mut BTreeMap<u64, Service>,
    ) {
        if let Some(host) = self.hosts.remove(&pos) {
            if let Some((service, node)) = host.registration
                && let Some(s) = services.get_mut(&service)
            {
                s.remove_node(node);
            }
            network.forget_chest_power(pos);
        }
    }
    pub(super) fn read(
        &mut self,
        game: &mut Game<'_>,
        network: &mut Network,
        services: &mut BTreeMap<u64, Service>,
    ) {
        let positions: Vec<_> = game
            .world
            .block_entities
            .iter()
            .filter_map(|(&(x, y, z), be)| {
                (matches!(be,BlockEntity::Mod(be) if be.id==rules::BLOCK)
                    && game.world.loaded(x, z)
                    && game.native_block([x, y, z]) == rules::BLOCK)
                    .then_some([x, y, z])
            })
            .collect();
        let present: BTreeSet<_> = positions.iter().copied().collect();
        let gone: Vec<_> = self
            .hosts
            .keys()
            .filter(|p| !present.contains(*p))
            .copied()
            .collect();
        for pos in gone {
            self.forget(pos, network, services);
        }
        for pos in positions {
            let key = (pos[0], pos[1], pos[2]);
            let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&key) else {
                continue;
            };
            let token = identify(be);
            if self.hosts.get(&pos).is_some_and(|h| h.token != token) {
                self.forget(pos, network, services);
            }
            let Some(BlockEntity::Mod(be)) = game.world.block_entities.get(&key) else {
                continue;
            };
            let host = self.hosts.entry(pos).or_insert_with(|| Host::new(token));
            host.observe(game.items(), be);
            network.bind_chest_power(pos, host.power.clone());
        }
    }
    pub(super) fn reconcile(
        &mut self,
        game: &Game<'_>,
        network: &mut Network,
        services: &mut BTreeMap<u64, Service>,
    ) {
        for (&pos, host) in &mut self.hosts {
            let registration = network.node_service(pos);
            if registration != host.registration {
                if let Some((service, node)) = host.registration
                    && let Some(s) = services.get_mut(&service)
                {
                    s.remove_node(node);
                }
                host.registration = None;
                host.refresh = true;
            }
            if !host.blocked {
                let mut frame = Frame {
                    host,
                    game,
                    network,
                    pos,
                    menus: Vec::new(),
                };
                frame.notifications();
                rules::on_node_state_changed(&mut frame);
                if let Some(idle) = frame.host.node_idle {
                    let _ = frame.network.set_idle(pos, idle);
                }
                if frame.host.refresh || frame.host.registration.is_none() {
                    frame.host.provider.lock().unwrap().cell = None;
                    rules::mount(&mut frame);
                }
            } else {
                host.provider.lock().unwrap().cell = None;
            }
            if let Some((service, node)) = registration {
                let s = services.entry(service).or_default();
                if host.registration.is_none() {
                    s.add_node(node, host.provider.clone())
                        .expect("unique Chest provider");
                    host.registration = registration;
                    host.refresh = false;
                } else if host.refresh {
                    s.refresh_node(node).expect("registered Chest provider");
                    host.refresh = false;
                }
            }
        }
    }
    pub(super) fn tick(
        &mut self,
        game: &Game<'_>,
        network: &mut Network,
        pos: Pos,
        be: &mut ModBlockEntity,
    ) {
        let token = identify(be);
        let host = self.hosts.entry(pos).or_insert_with(|| Host::new(token));
        if host.token != token || host.blocked {
            return;
        }
        host.observe(game.items(), be);
        if host.blocked {
            return;
        }
        let mut frame = Frame {
            host,
            game,
            network,
            pos,
            menus: Vec::new(),
        };
        frame.notifications();
        rules::server_tick(&mut frame);
        frame.events();
        frame.host.flush(be);
    }
    pub(super) fn flush(&mut self, game: &mut Game<'_>) {
        for (&pos, host) in &mut self.hosts {
            let Some(BlockEntity::Mod(be)) =
                game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
            else {
                continue;
            };
            if be.id == rules::BLOCK && host.flush(be) {
                game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
            }
        }
    }
    pub(super) fn updates(&mut self) -> Vec<(Pos, u64, bool)> {
        self.hosts
            .iter_mut()
            .filter_map(|(&pos, host)| {
                std::mem::take(&mut host.mark).then_some((pos, host.token, host.light_on))
            })
            .collect()
    }
}
pub(super) fn apply(game: &mut Game<'_>, updates: Vec<(Pos, u64, bool)>) {
    for (pos, token, on) in updates {
        if game.native_block(pos) != rules::BLOCK
            || !matches!(game.world.block_entities.get(&(pos[0],pos[1],pos[2])),Some(BlockEntity::Mod(be))if id(be)==Some(token))
        {
            continue;
        }
        game.native_set_block_property_with(
            pos,
            "lights_on",
            if on { "true" } else { "false" },
            horizon_world::level::flags::UPDATE_ALL,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_client::{items::Items, save};
    use horizon_world::loot::PackOverlay;

    fn items() -> Items {
        Items::catalog(
            &std::path::PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap()),
            &PackOverlay::NONE,
        )
        .unwrap()
    }
    fn entity() -> ModBlockEntity {
        ModBlockEntity {
            id: rules::BLOCK.into(),
            items: vec![None, Some(ItemStack::new("ae2:item_storage_cell_1k", 1))],
            nbt: Compound::from([(
                "internalCurrentPower".into(),
                Tag::Double(250.0f64.to_bits()),
            )]),
            ..Default::default()
        }
    }
    #[test]
    fn native_tick_take_keeps_identity_but_actual_save_load_creates_a_fresh_host() {
        let mut be = entity();
        let without_identity = save::encode_block_entity(
            (3, 70, -8),
            &BlockEntity::Mod(Box::new(be.clone())),
            rules::BLOCK,
        );
        let token = identify(&mut be);
        let mut taken = std::mem::take(&mut be);
        assert_eq!(identify(&mut taken), token);
        assert!(be.data.is_empty());
        be = taken;
        let saved =
            save::encode_block_entity((3, 70, -8), &BlockEntity::Mod(Box::new(be)), rules::BLOCK);
        assert_eq!(saved, without_identity, "identity is not a saved AE2 field");
        let (pos, BlockEntity::Mod(mut loaded)) = save::decode_block_entity(&saved).unwrap() else {
            panic!("native saved Chest");
        };
        assert_eq!(pos, (3, 70, -8));
        assert!(loaded.data.is_empty());
        assert_ne!(identify(&mut loaded), token);
    }
    #[test]
    fn live_cell_and_energy_survive_unchanged_echo_and_roundtrip_through_native_save() {
        let catalog = items();
        let mut be = entity();
        be.items.resize(9, None); // A menu carrier may retain empty native padding.
        be.nbt.insert(
            "inv".into(),
            super::super::ae2_inventory_codec::write_slots(
                be.items[..2].iter().map(Option::as_ref),
                Some(&Tag::compound([("test:unknown", Tag::Int(9))])),
            ),
        );
        let mut host = Host::new(identify(&mut be));
        host.observe(&catalog, &be);
        assert!(!host.blocked);
        let cell_item = host.items[1].clone();
        let backend = codec::load_backend(&value(&cell_item).unwrap(), &|kind, id| {
            codec::resolve_key(&catalog, kind, id)
        })
        .unwrap();
        let cell = Arc::new(Mutex::new(CellHost {
            backend,
            item: cell_item.clone(),
            dirty: false,
            blink: false,
        }));
        host.state.handler = Some(cell.clone());
        host.state.cached = true;
        let iron = codec::item_key(&ItemStack::new("minecraft:iron_ingot", 1), &catalog).unwrap();
        assert_eq!(cell.lock().unwrap().transfer(&iron, 17, true, true), 17);
        assert_eq!(host.power.extract(24.5, true), 24.5);
        host.observe(&catalog, &be);
        assert!(Arc::ptr_eq(&host.items[1], &cell_item));
        assert!(Arc::ptr_eq(host.state.handler.as_ref().unwrap(), &cell));
        assert_eq!(host.power.amount(), 225.5);
        // The grid-facing provider mutates and persists this very same cell.
        assert_eq!(
            cell.lock()
                .unwrap()
                .extract(&iron, 4, true, &Source::default()),
            4
        );
        host.observe(&catalog, &be);
        assert!(host.flush(&mut be));
        assert_eq!(be.items.len(), 9);
        assert!(be.items[2..].iter().all(Option::is_none));
        assert_eq!(
            be.nbt[&JavaString::from("inv")].get("test:unknown"),
            Some(&Tag::Int(9))
        );
        assert!(be.nbt[&JavaString::from("inv")].get("item2").is_none());
        assert!(!host.flush(&mut be));
        let saved =
            save::encode_block_entity((0, 70, 0), &BlockEntity::Mod(Box::new(be)), rules::BLOCK);
        let (_, BlockEntity::Mod(mut loaded)) = save::decode_block_entity(&saved).unwrap() else {
            panic!("native saved Chest");
        };
        let mut restarted = Host::new(identify(&mut loaded));
        restarted.observe(&catalog, &loaded);
        assert_eq!(restarted.power.amount(), 225.5);
        let backend = codec::load_backend(&value(&restarted.items[1]).unwrap(), &|kind, id| {
            codec::resolve_key(&catalog, kind, id)
        })
        .unwrap();
        assert_eq!(backend.stored_count(), 13);
        assert_eq!(backend.stacks().get(&iron), Some(&13));
    }
    #[test]
    fn malformed_inventory_extra_slots_and_nonfinite_power_are_preserved_until_repaired() {
        let catalog = items();
        for variant in 0..4 {
            let mut be = entity();
            match variant {
                0 => {
                    be.nbt.insert("inv".into(), Tag::Int(9));
                }
                1 => be.items.push(Some(ItemStack::new("minecraft:diamond", 1))),
                2 => {
                    be.nbt.insert(
                        "internalCurrentPower".into(),
                        Tag::Double(f64::NAN.to_bits()),
                    );
                }
                _ => be.items[0] = Some(ItemStack::new("missing:unknown_input", 1)),
            }
            let mut host = Host::new(identify(&mut be));
            let original = be.clone();
            host.observe(&catalog, &be);
            assert!(host.blocked, "variant {variant}");
            assert!(!host.flush(&mut be));
            assert_eq!(be, original);
            be.nbt.remove(&JavaString::from("inv"));
            be.nbt.insert(
                "internalCurrentPower".into(),
                Tag::Double(42.0f64.to_bits()),
            );
            be.items.truncate(2);
            be.items[0] = None;
            host.observe(&catalog, &be);
            assert!(!host.blocked, "repair {variant}");
            assert!(host.flush(&mut be));
            assert_eq!(host.power.amount(), 42.);
        }
    }
    #[test]
    fn a_stale_host_cannot_flush_into_a_replacement_at_the_same_position() {
        let catalog = items();
        let mut old = entity();
        let mut host = Host::new(identify(&mut old));
        host.observe(&catalog, &old);
        host.power.extract(70., true);
        let mut replacement = entity();
        assert_ne!(identify(&mut replacement), host.token);
        let before = replacement.clone();
        assert!(!host.flush(&mut replacement));
        assert_eq!(replacement, before);
        assert!(host.flush(&mut old));
        assert_eq!(
            number(old.nbt.get(&JavaString::from("internalCurrentPower"))),
            180.
        );
    }
}
