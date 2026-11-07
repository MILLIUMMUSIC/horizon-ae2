// SPDX-License-Identifier: LGPL-3.0-or-later
// Loaded Drive nodes, basic-cell mounts and original inv.itemN persistence.
// The adapter also mirrors item slots into Horizon's Items transport. There is
// one authoritative cell per slot, never a second inventory of stored contents.

use super::{
    Game, ae2_cells as codec,
    ae2_drive::{Drive, DriveCell, SLOTS},
    ae2_keys::Key,
    ae2_network::{Kind, Network, Pos},
    ae2_service::{StorageMounts, StorageProvider, StorageService},
};
use horizon_client::{containers::BlockEntity, items::ItemStack};
use horizon_world::nbt::{JavaString, Tag};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Default)]
pub struct Source {
    pub player: Option<u128>,
    pub machine_node: Option<u64>,
}
type Service = StorageService<Key, Source>;
struct Host {
    drive: Arc<Mutex<Drive<Key>>>,
    items: Vec<Option<ItemStack>>,
    inv: Option<Tag>,
    observed: bool,
    blocked: bool,
    refresh: bool,
    registration: Option<(u64, u64)>,
}
impl Default for Host {
    fn default() -> Self {
        Self {
            drive: Arc::new(Mutex::new(Drive::default())),
            items: vec![None; SLOTS],
            inv: None,
            observed: false,
            blocked: false,
            refresh: true,
            registration: None,
        }
    }
}

#[cfg(test)]
fn decode_inv(tag: &Tag) -> Result<Vec<Option<ItemStack>>, &'static str> {
    super::ae2_inventory_codec::read_slots(tag, SLOTS)
}
fn encode_inv(items: &[Option<ItemStack>], previous: Option<&Tag>) -> Tag {
    super::ae2_inventory_codec::write_slots(
        (0..SLOTS).map(|slot| items.get(slot).and_then(Option::as_ref)),
        previous,
    )
}

#[derive(Default)]
pub(super) struct Drives {
    hosts: BTreeMap<Pos, Host>,
    services: BTreeMap<u64, Service>,
    chests: BTreeMap<Pos, ChestHost>,
}
impl Drives {
    /// Observe commands/slot changes before topology reconciliation. Unchanged
    /// slot echoes must never overwrite a cell modified during the level tick.
    pub(super) fn read(&mut self, game: &mut Game<'_>) {
        let rows: Vec<_> = game
            .world
            .block_entities
            .iter()
            .filter_map(|(&(x, y, z), entity)| {
                let BlockEntity::Mod(be) = entity else {
                    return None;
                };
                if be.id != Kind::Drive.block()
                    || !game.world.loaded(x, z)
                    || game.native_block([x, y, z]) != Kind::Drive.block()
                {
                    return None;
                }
                let mut items = be.items.iter().take(SLOTS).cloned().collect::<Vec<_>>();
                items.resize(SLOTS, None);
                let priority = match be.nbt.get(&JavaString::from("priority")) {
                    Some(Tag::Int(p)) => *p,
                    _ => 0,
                };
                Some((
                    [x, y, z],
                    items,
                    be.nbt.get(&JavaString::from("inv")).cloned(),
                    priority,
                ))
            })
            .collect();
        let present: BTreeSet<_> = rows.iter().map(|row| row.0).collect();
        let gone: Vec<_> = self
            .hosts
            .keys()
            .filter(|p| !present.contains(*p))
            .copied()
            .collect();
        for pos in gone {
            let host = self.hosts.remove(&pos).unwrap();
            if let Some((service, node)) = host.registration
                && let Some(service) = self.services.get_mut(&service)
            {
                service.remove_node(node);
            }
        }
        for (pos, source_items, inv, priority) in rows {
            let host = self.hosts.entry(pos).or_default();
            let mut drive = host.drive.lock().unwrap();
            if drive.priority != priority {
                drive.priority = priority;
                host.refresh = true;
            }
            if !host.observed || host.items != source_items || host.inv != inv {
                let source_echo = source_items.clone();
                let chosen = super::ae2_inventory_codec::choose_slots(
                    inv.as_ref(),
                    host.inv.as_ref(),
                    &source_items,
                    host.observed,
                    host.blocked,
                    SLOTS,
                );
                match chosen {
                    Ok(items) => {
                        for slot in 0..SLOTS {
                            if !host.observed || host.blocked || host.items[slot] != items[slot] {
                                drive.slots[slot] = items[slot]
                                    .as_ref()
                                    .and_then(|stack| {
                                        let resolve = |kind, id: &str| {
                                            codec::resolve_key(game.items(), kind, id)
                                        };
                                        if let Some(allowed) =
                                            super::ae2_tool_glue::accepted(game, &stack.item)
                                        {
                                            codec::load_restricted_backend(stack, &resolve, allowed)
                                                .ok()
                                        } else {
                                            codec::load_backend(stack, &resolve).ok()
                                        }
                                    })
                                    .map(|cell| Arc::new(Mutex::new(DriveCell::new(cell))));
                                host.refresh = true;
                            }
                        }
                        if let Some(BlockEntity::Mod(be)) =
                            game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
                        {
                            be.items.resize(be.items.len().max(SLOTS), None);
                            if be.items[..SLOTS] != items {
                                be.items[..SLOTS].clone_from_slice(&items);
                                game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
                            }
                        }
                        host.items = items;
                        host.blocked = false;
                    }
                    Err(error) => {
                        // Preserve the rejected original NBT, unmount it and
                        // wait for an explicit valid update. Never truncate it.
                        eprintln!("AE2 Drive {pos:?}: {error}");
                        drive.slots = std::array::from_fn(|_| None);
                        host.blocked = true;
                        host.refresh = true;
                        host.items = source_echo;
                    }
                }
                host.inv = inv;
                host.observed = true;
            }
        }
    }
    pub(super) fn reconcile(&mut self, network: &mut Network) {
        for (pos, host) in &mut self.hosts {
            let registration = network.node_service(*pos);
            if host.registration != registration {
                if let Some((service, node)) = host.registration
                    && let Some(service) = self.services.get_mut(&service)
                {
                    service.remove_node(node);
                }
                host.registration = None;
                host.refresh = true;
            }
            let mut drive = host.drive.lock().unwrap();
            let online = !host.blocked && network.is_online(*pos);
            if drive.online != online {
                drive.online = online;
                host.refresh = true;
            }
            let idle = drive.idle_drain();
            drop(drive);
            if registration.is_some() {
                network.set_idle(*pos, idle).expect("loaded Drive node");
            }
            if let Some((service, node)) = registration {
                let storage = self.services.entry(service).or_default();
                if host.registration.is_none() {
                    storage
                        .add_node(node, host.drive.clone())
                        .expect("unique Drive registration");
                    host.registration = registration;
                    host.refresh = false;
                } else if host.refresh {
                    storage
                        .refresh_node(node)
                        .expect("registered Drive provider");
                    host.refresh = false;
                }
            }
        }
        let mut used: BTreeSet<_> = self
            .hosts
            .values()
            .filter_map(|h| h.registration.map(|r| r.0))
            .collect();
        used.extend(
            self.chests
                .values()
                .filter_map(|host| host.registration.map(|pair| pair.0)),
        );
        self.services.retain(|id, _| used.contains(id));
    }
    pub(super) fn end_tick(&mut self) {
        for service in self.services.values_mut() {
            service.end_tick(&mut |_, _, _, _| {});
        }
    }
    pub(super) fn persist(&mut self, game: &mut Game<'_>) {
        for (pos, host) in &mut self.hosts {
            if host.blocked {
                continue;
            }
            let mut items = host.items.clone();
            let drive = host.drive.lock().unwrap();
            let mut failed = false;
            for (slot, cell) in drive.slots.iter().enumerate() {
                if let Some(cell) = cell {
                    let cell = cell.lock().unwrap();
                    if cell.dirty
                        && let Some(stack) = items[slot].as_mut()
                    {
                        if codec::persist_backend(stack, &cell.cell).is_err() {
                            failed = true;
                            break;
                        }
                    }
                }
            }
            if failed {
                continue;
            }
            let inv = encode_inv(&items, host.inv.as_ref());
            let packed = drive.packed_state();
            if let Some(BlockEntity::Mod(be)) =
                game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
            {
                if be.id != Kind::Drive.block() {
                    continue;
                }
                let changed = be.items.get(..SLOTS) != Some(&items[..])
                    || be.nbt.get(&JavaString::from("inv")) != Some(&inv);
                be.items.resize(be.items.len().max(SLOTS), None);
                be.items[..SLOTS].clone_from_slice(&items);
                be.nbt.insert(JavaString::from("inv"), inv.clone());
                // Two ContainerData words transport the original packed u32.
                be.data = vec![(packed & 0xffff) as i32, (packed >> 16) as i32];
                if changed {
                    game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
                }
                // Clear dirty bits only after every slot encoded and the
                // complete inventory reached the still-existing block entity.
                for cell in drive.slots.iter().flatten() {
                    cell.lock().unwrap().dirty = false;
                }
            } else {
                continue;
            }
            host.items = items;
            host.inv = Some(inv);
        }
        self.persist_chests(game);
    }
}

struct ChestCells {
    cell: Option<Arc<Mutex<DriveCell<Key>>>>,
    priority: i32,
    online: bool,
}
impl StorageProvider<Key, Source> for ChestCells {
    fn mount(&mut self, mounts: &mut StorageMounts<'_, Key, Source>) -> Result<(), &'static str> {
        if self.online
            && let Some(cell) = &self.cell
        {
            mounts.mount(cell.clone(), self.priority)?;
        }
        Ok(())
    }
}
struct ChestHost {
    provider: Arc<Mutex<ChestCells>>,
    item: Option<ItemStack>,
    registration: Option<(u64, u64)>,
    refresh: bool,
}
impl Default for ChestHost {
    fn default() -> Self {
        Self {
            provider: Arc::new(Mutex::new(ChestCells {
                cell: None,
                priority: 0,
                online: false,
            })),
            item: None,
            registration: None,
            refresh: true,
        }
    }
}
impl Drives {
    pub(super) fn read_chests(&mut self, game: &mut Game<'_>) {
        let slot = super::ae2_me_chest::CELL_SLOT;
        let rows: Vec<_> = game
            .world
            .block_entities
            .iter()
            .filter_map(|(&(x, y, z), entity)| {
                let BlockEntity::Mod(be) = entity else {
                    return None;
                };
                if be.id != super::ae2_me_chest::BLOCK
                    || !game.world.loaded(x, z)
                    || game.native_block([x, y, z]) != super::ae2_me_chest::BLOCK
                {
                    return None;
                }
                let item = be.items.get(slot).cloned().flatten();
                let priority = match be.nbt.get(&JavaString::from("priority")) {
                    Some(Tag::Int(value)) => *value,
                    _ => 0,
                };
                Some(([x, y, z], item, priority))
            })
            .collect();
        let present: BTreeSet<_> = rows.iter().map(|row| row.0).collect();
        let gone: Vec<_> = self
            .chests
            .keys()
            .filter(|pos| !present.contains(*pos))
            .copied()
            .collect();
        for pos in gone {
            let host = self.chests.remove(&pos).unwrap();
            if let Some((service, node)) = host.registration
                && let Some(service) = self.services.get_mut(&service)
            {
                service.remove_node(node);
            }
        }
        for (pos, item, priority) in rows {
            let host = self.chests.entry(pos).or_default();
            let mut cells = host.provider.lock().unwrap();
            if cells.priority != priority {
                cells.priority = priority;
                host.refresh = true;
            }
            if host.item != item {
                cells.cell = item.as_ref().and_then(|stack| chest_cell(game, stack));
                host.item = item;
                host.refresh = true;
            }
        }
    }
    pub(super) fn reconcile_chests(&mut self, network: &mut Network) {
        for (pos, host) in &mut self.chests {
            let registration = network.node_service(*pos);
            if host.registration != registration {
                if let Some((service, node)) = host.registration
                    && let Some(service) = self.services.get_mut(&service)
                {
                    service.remove_node(node);
                }
                host.registration = None;
                host.refresh = true;
            }
            let mut cells = host.provider.lock().unwrap();
            let online = network.is_online(*pos);
            if cells.online != online {
                cells.online = online;
                host.refresh = true;
            }
            let idle = cells.cell.as_ref().map_or(0.0, |cell| {
                1.0 + cell.lock().unwrap().cell.idle_drain()
            });
            drop(cells);
            if registration.is_some() {
                let _ = network.set_idle(*pos, idle);
            }
            if let Some((service, node)) = registration {
                let storage = self.services.entry(service).or_default();
                if host.registration.is_none() {
                    if storage.add_node(node, host.provider.clone()).is_ok() {
                        host.registration = registration;
                        host.refresh = false;
                    }
                } else if host.refresh {
                    let _ = storage.refresh_node(node);
                    host.refresh = false;
                }
            }
        }
    }
    pub(super) fn transfer(
        &self,
        pos: Pos,
        key: &Key,
        amount: u64,
        insert: bool,
        commit: bool,
    ) -> u64 {
        let Some(service) = self.service_of(pos) else {
            return 0;
        };
        let Some(storage) = self.services.get(&service) else {
            return 0;
        };
        let source = Source::default();
        if insert {
            storage.inventory().insert(key, amount, commit, &source)
        } else {
            storage.inventory().extract(key, amount, commit, &source)
        }
    }
    pub(super) fn first_item(&self, pos: Pos) -> Option<Key> {
        let service = self.service_of(pos)?;
        for host in self.hosts.values() {
            if host.registration.map(|pair| pair.0) != Some(service) {
                continue;
            }
            let drive = host.drive.lock().unwrap();
            for slot in drive.slots.iter().flatten() {
                let cell = slot.lock().unwrap();
                if let Some((key, _)) = cell.cell.stacks().iter().find(|(_, amount)| **amount > 0) {
                    return Some(key.clone());
                }
            }
        }
        for host in self.chests.values() {
            if host.registration.map(|pair| pair.0) != Some(service) {
                continue;
            }
            let cells = host.provider.lock().unwrap();
            if let Some(cell) = &cells.cell {
                let cell = cell.lock().unwrap();
                if let Some((key, _)) = cell.cell.stacks().iter().find(|(_, amount)| **amount > 0)
                {
                    return Some(key.clone());
                }
            }
        }
        None
    }
    fn service_of(&self, pos: Pos) -> Option<u64> {
        self.chests
            .get(&pos)
            .and_then(|host| host.registration.map(|pair| pair.0))
            .or_else(|| {
                self.hosts
                    .get(&pos)
                    .and_then(|host| host.registration.map(|pair| pair.0))
            })
    }
    fn persist_chests(&mut self, game: &mut Game<'_>) {
        let slot = super::ae2_me_chest::CELL_SLOT;
        for (pos, host) in &mut self.chests {
            let cells = host.provider.lock().unwrap();
            let Some(cell) = &cells.cell else {
                continue;
            };
            let mut cell = cell.lock().unwrap();
            if !cell.dirty {
                continue;
            }
            let Some(item) = host.item.as_mut() else {
                continue;
            };
            if codec::persist_backend(item, &cell.cell).is_err() {
                continue;
            }
            cell.dirty = false;
            let item = item.clone();
            drop(cell);
            drop(cells);
            if let Some(BlockEntity::Mod(be)) =
                game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
                && be.id == super::ae2_me_chest::BLOCK
            {
                be.items.resize(be.items.len().max(slot + 1), None);
                be.items[slot] = Some(item);
                game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
            }
        }
    }
}

fn chest_cell(game: &Game<'_>, stack: &ItemStack) -> Option<Arc<Mutex<DriveCell<Key>>>> {
    let resolve = |kind, id: &str| codec::resolve_key(game.items(), kind, id);
    let cell = if let Some(allowed) = super::ae2_tool_glue::accepted(game, &stack.item) {
        codec::load_restricted_backend(stack, &resolve, allowed).ok()?
    } else {
        codec::load_backend(stack, &resolve).ok()?
    };
    Some(Arc::new(Mutex::new(DriveCell::new(cell))))
}

#[cfg(test)]
mod tests {
    use super::super::{
        ae2_grid::Direction,
        ae2_keys::{KeyProperties, KeyType},
        ae2_network::Snapshot,
        ae2_storage::{Cell, Tier},
    };
    use super::*;
    fn node(pos: Pos, kind: Kind) -> Snapshot {
        Snapshot {
            pos,
            kind,
            front: Direction::North,
            exposed: 63,
            stored: 0.0,
            saved_buffer: 0.0,
        }
    }
    fn setup_host(drives: &mut Drives, pos: Pos, amount: u64) -> Arc<Mutex<DriveCell<Key>>> {
        let mut cell = Cell::new(Tier::K1, KeyType::Item);
        cell.insert(
            Key::new(
                KeyType::Item,
                "minecraft:iron_ingot".into(),
                vec![],
                KeyProperties::default(),
            )
            .unwrap(),
            amount,
            true,
        );
        let cell = Arc::new(Mutex::new(DriveCell::new(cell)));
        let host = Host::default();
        host.drive.lock().unwrap().slots[0] = Some(cell.clone());
        drives.hosts.insert(pos, host);
        cell
    }
    fn run(network: &mut Network, drives: &mut Drives) {
        drives.reconcile(network);
        network.begin_tick();
        network.end_tick();
        drives.reconcile(network);
    }
    fn iron() -> Key {
        Key::new(
            KeyType::Item,
            "minecraft:iron_ingot".into(),
            vec![],
            KeyProperties::default(),
        )
        .unwrap()
    }
    #[test]
    fn creative_component_backend_mounts_on_real_native_network_and_unmounts_offline() {
        let mut stack = ItemStack::new("ae2:creative_storage_cell", 1);
        stack.data_mut().set_other(
            "ae2:storage_cell_config_inv",
            Some(Tag::List(vec![
                codec::generic_to_tag(&super::super::ae2_keys::GenericStack {
                    what: iron(),
                    amount: 0,
                })
                .unwrap(),
            ])),
        );
        let original = stack.clone();
        let backend = codec::load_backend(&stack, &|_, _| Some(KeyProperties::default())).unwrap();
        let cell = Arc::new(Mutex::new(DriveCell::new(backend)));
        let host = Host::default();
        host.drive.lock().unwrap().slots[0] = Some(cell.clone());
        let pos = [0, 0, 0];
        let mut drives = Drives::default();
        drives.hosts.insert(pos, host);
        let mut network = Network::default();
        let drive = node(pos, Kind::Drive);
        network.sync(vec![drive, node([0, 0, 1], Kind::CreativeEnergyCell)]);
        run(&mut network, &mut drives);
        let service = network.node_service(pos).unwrap().0;
        let inventory = drives.services[&service].inventory().clone();
        assert_eq!(
            inventory.extract(&iron(), 1_000_000_000_000, true, &Source::default()),
            1_000_000_000_000
        );
        assert_eq!(drives.hosts[&pos].drive.lock().unwrap().idle_drain(), 2.0);
        assert_eq!(
            drives.hosts[&pos].drive.lock().unwrap().packed_state() & 7,
            3
        );
        codec::persist_backend(&mut stack, &cell.lock().unwrap().cell).unwrap();
        assert_eq!(stack, original);
        network.sync(vec![drive]);
        run(&mut network, &mut drives);
        assert_eq!(inventory.extract(&iron(), 1, true, &Source::default()), 0);
    }
    #[test]
    fn native_drive_binding_unmounts_on_power_loss_and_remounts_same_cell() {
        let mut drives = Drives::default();
        let mut network = Network::default();
        let pos = [0, 0, 0];
        let cell = setup_host(&mut drives, pos, 64);
        let drive = node(pos, Kind::Drive);
        let source = node([0, 0, 1], Kind::CreativeEnergyCell);
        network.sync(vec![drive, source]);
        run(&mut network, &mut drives);
        let registration = network.node_service(pos).unwrap();
        let inventory = drives.services[&registration.0].inventory().clone();
        assert_eq!(
            inventory.extract(&iron(), 64, false, &Source::default()),
            64
        );
        network.sync(vec![drive]);
        run(&mut network, &mut drives);
        assert_eq!(inventory.extract(&iron(), 64, true, &Source::default()), 0);
        assert_eq!(cell.lock().unwrap().cell.stored_count(), 64);
        network.sync(vec![drive, source]);
        for _ in 0..40 {
            run(&mut network, &mut drives);
        }
        let id = network.node_service(pos).unwrap().0;
        assert_eq!(
            drives.services[&id]
                .inventory()
                .extract(&iron(), 64, true, &Source::default()),
            64
        );
        assert!(cell.lock().unwrap().dirty);
    }
    #[test]
    fn native_drive_mounts_follow_grid_merge_split_without_copying_contents() {
        let mut drives = Drives::default();
        let mut network = Network::default();
        let a = [0, 0, 0];
        let b = [2, 0, 0];
        let ca = setup_host(&mut drives, a, 64);
        let cb = setup_host(&mut drives, b, 50);
        let nodes = vec![
            node(a, Kind::Drive),
            node(b, Kind::Drive),
            node([0, 0, 1], Kind::CreativeEnergyCell),
            node([2, 0, 1], Kind::CreativeEnergyCell),
        ];
        network.sync(nodes.clone());
        run(&mut network, &mut drives);
        let ida = network.node_service(a).unwrap().0;
        let idb = network.node_service(b).unwrap().0;
        assert_ne!(ida, idb);
        let olda = drives.services[&ida].inventory().clone();
        let oldb = drives.services[&idb].inventory().clone();
        let mut merged = nodes.clone();
        merged.push(node([1, 0, 0], Kind::EnergyCell));
        network.sync(merged);
        run(&mut network, &mut drives);
        let id = network.node_service(a).unwrap().0;
        assert_eq!(id, network.node_service(b).unwrap().0);
        assert_eq!(
            drives.services[&id]
                .inventory()
                .extract(&iron(), 200, false, &Source::default()),
            114
        );
        // Only the retained service still owns the mounts; the discarded one
        // must be empty even when an external capability keeps its handle.
        let discarded = if id == ida { oldb } else { olda };
        assert_eq!(discarded.extract(&iron(), 200, true, &Source::default()), 0);
        network.sync(nodes);
        for _ in 0..40 {
            run(&mut network, &mut drives);
        }
        let ida = network.node_service(a).unwrap().0;
        let idb = network.node_service(b).unwrap().0;
        assert_ne!(ida, idb);
        assert_eq!(
            drives.services[&ida]
                .inventory()
                .extract(&iron(), 200, true, &Source::default()),
            64
        );
        assert_eq!(
            drives.services[&idb]
                .inventory()
                .extract(&iron(), 200, true, &Source::default()),
            50
        );
        assert_eq!(
            ca.lock().unwrap().cell.stored_count() + cb.lock().unwrap().cell.stored_count(),
            0
        );
    }
    #[test]
    fn original_inv_item_slots_roundtrip_and_preserve_components() {
        let mut items = vec![None; SLOTS];
        let mut stack = ItemStack::new("ae2:item_storage_cell_1k", 1);
        stack
            .data_mut()
            .set_other("ae2:upgrades", Some(Tag::List(vec![])));
        items[9] = Some(stack);
        let inv = encode_inv(&items, None);
        assert_eq!(decode_inv(&inv).unwrap(), items);
        assert!(matches!(inv.get("item0"), Some(Tag::Compound(c)) if c.is_empty()));
        assert!(inv.get("item9").is_some());
        assert!(inv.get("item10").is_none());
    }
    #[test]
    fn malformed_drive_inventory_is_rejected_without_changing_the_original() {
        let bad = Tag::compound([("item0", Tag::String(JavaString::from("broken")))]);
        let saved = bad.clone();
        assert!(decode_inv(&bad).is_err());
        assert_eq!(bad, saved);
        let bad_count = Tag::compound([(
            "item0",
            horizon_client::save::item_to_nbt(&ItemStack::new("ae2:item_storage_cell_1k", 2)),
        )]);
        // AEBaseInvBlockEntity uses setItemDirect; slot input limits do not
        // clamp a positive stack count loaded directly from original NBT.
        assert_eq!(
            decode_inv(&bad_count).unwrap()[0].as_ref().unwrap().count,
            2
        );
        let zero = Tag::compound([(
            "item0",
            horizon_client::save::item_to_nbt(&ItemStack::new("ae2:item_storage_cell_1k", 0)),
        )]);
        assert!(decode_inv(&zero).is_err());
    }
}
