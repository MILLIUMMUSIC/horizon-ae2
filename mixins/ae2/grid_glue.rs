// SPDX-License-Identifier: LGPL-3.0-or-later
// Native full-block network lifecycle. Only server level hooks modify networks.
// Every dimension owns independent topology and EnergyService state.

use super::{
    Game,
    ae2_grid::Direction,
    ae2_network::{Kind, Network, Snapshot},
};
use horizon_client::containers::BlockEntity;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

#[derive(Default)]
struct Dimension {
    network: Network,
    drives: super::ae2_drive_glue::Drives,
    chargers: super::ae2_charger_inventory::Inventories,
    cells: super::ae2_energy_cell_glue::Displays,
    cables: super::ae2_cable_bus::Hosts,
}
static NETWORKS: Mutex<BTreeMap<String, Dimension>> = Mutex::new(BTreeMap::new());

pub(super) fn owns(block: &str) -> bool {
    Kind::from_block(block).is_some() || block == super::ae2_cable_bus::BLOCK
}
fn get_double(nbt: &Compound, key: &str) -> f64 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Double(v)) if f64::from_bits(*v).is_finite() => f64::from_bits(*v).max(0.0),
        _ => 0.0,
    }
}
fn direction(game: &Game<'_>, pos: [i32; 3]) -> Direction {
    match game.native_block_property(pos, "facing").as_deref() {
        Some("down") => Direction::Down,
        Some("up") => Direction::Up,
        Some("south") => Direction::South,
        Some("west") => Direction::West,
        Some("east") => Direction::East,
        _ => Direction::North,
    }
}
fn snapshots(game: &Game<'_>, state: &mut Dimension) -> Vec<Snapshot> {
    let mut snapshots = vec![];
    let mut cable_positions = BTreeSet::new();
    for (&(x, y, z), entity) in &game.world.block_entities {
        let BlockEntity::Mod(be) = entity else {
            continue;
        };
        if be.id == super::ae2_cable_bus::BLOCK {
            let pos = [x, y, z];
            if !game.world.loaded(x, z) || game.native_block(pos) != super::ae2_cable_bus::BLOCK {
                continue;
            }
            cable_positions.insert(pos);
            if let Ok(snapshot) = state.cables.snapshot(pos, &be.nbt, &mut state.network) {
                snapshots.extend(snapshot);
            }
            continue;
        }
        let Some(kind) = Kind::from_block(&be.id) else {
            continue;
        };
        if !game.world.loaded(x, z) || game.native_block([x, y, z]) != kind.block() {
            continue;
        }
        let saved_buffer = match be.nbt.get(&JavaString::from("proxy")) {
            Some(Tag::Compound(node)) => get_double(node, "e"),
            _ => 0.0,
        };
        snapshots.push(Snapshot {
            pos: [x, y, z],
            kind,
            front: direction(game, [x, y, z]),
            exposed: 63,
            stored: get_double(&be.nbt, "internalCurrentPower"),
            saved_buffer,
        });
    }
    state.cables.retain(&cable_positions);
    snapshots
}
pub(super) fn pre_tick(game: &mut Game<'_>, dimension: &str) {
    let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    let state = networks.entry(dimension.to_owned()).or_default();
    state.chargers.sync(game);
    state.drives.read(game);
    state.drives.read_chests(game);
    let nodes = snapshots(game, state);
    state.network.sync(nodes);
    state.drives.reconcile(&mut state.network);
    state.drives.reconcile_chests(&mut state.network);
    state.network.begin_tick();
}
// Placement must test the just-added cable's actual connected shape without
// advancing TickManager or charging/draining any service a second time.
pub(super) fn cables_changed(game: &mut Game<'_>) {
    let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    let state = networks.entry(game.dim.id().to_owned()).or_default();
    let nodes = snapshots(game, state);
    state.network.sync(nodes);
    let poses: Vec<_> = game
        .world
        .block_entities
        .iter()
        .filter_map(|(&(x, y, z), be)| {
            matches!(be,BlockEntity::Mod(be) if be.id==super::ae2_cable_bus::BLOCK)
                .then_some([x, y, z])
        })
        .collect();
    for pos in poses {
        if let Some(visual) = state.network.cable_visual(pos)
            && let Some(BlockEntity::Mod(be)) =
                game.world.block_entities.get_mut(&(pos[0], pos[1], pos[2]))
        {
            be.data = vec![
                horizon_client_render::ae2_cable_view::MAGIC,
                i32::from(visual.connections),
            ];
            be.data.extend(visual.channels);
            if let Some(log) = &mut game.world.block_log {
                log.push((pos[0], pos[1], pos[2]));
            }
        }
    }
}
pub(super) fn post_tick(game: &mut Game<'_>, dimension: &str) {
    // Block placement, removal and /data may run inside the level tick. Notice
    // those changes before persisting; unchanged NBT is not an external refill
    // and must never restore energy already extracted during this tick.
    let states = {
        let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
        let Some(state) = networks.get_mut(dimension) else {
            return;
        };
        state.chargers.sync(game);
        state.drives.read(game);
        state.drives.read_chests(game);
        let nodes = snapshots(game, state);
        state.network.sync(nodes);
        state.drives.reconcile(&mut state.network);
        state.drives.reconcile_chests(&mut state.network);
        let states = state.network.end_tick();
        for persisted in &states {
            let Kind::Cable(cable) = persisted.kind else {
                continue;
            };
            let [x, y, z] = persisted.pos;
            let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&(x, y, z)) else {
                continue;
            };
            if let Some(visual) = state.network.cable_visual(persisted.pos) {
                let mut data = vec![
                    horizon_client_render::ae2_cable_view::MAGIC,
                    i32::from(visual.connections),
                ];
                data.extend(visual.channels);
                if be.data != data {
                    be.data = data;
                    if let Some(log) = &mut game.world.block_log {
                        log.push((x, y, z));
                    }
                }
            }
            if be.id == super::ae2_cable_bus::BLOCK
                && state
                    .cables
                    .persist(persisted.pos, &mut be.nbt, cable, persisted.buffer_share)
            {
                game.world.modified.insert((x >> 4, z >> 4));
            }
        }
        state.drives.reconcile(&mut state.network);
        state.drives.reconcile_chests(&mut state.network);
        state.drives.end_tick();
        state.drives.persist(game);
        // Energy writes are now saved; comparator reads must see those values.
        states
    };
    let controllers: Vec<_> = states
        .iter()
        .filter_map(|state| state.controller_model.map(|model| (state.pos, model)))
        .collect();
    for state in states {
        if matches!(state.kind, Kind::Cable(_)) {
            continue;
        }
        let [x, y, z] = state.pos;
        let Some(BlockEntity::Mod(be)) = game.world.block_entities.get_mut(&(x, y, z)) else {
            continue;
        };
        if be.id != state.kind.block() {
            continue;
        }
        let before = be.nbt.clone();
        if let Some(energy) = state.stored
            && state.kind != Kind::CreativeEnergyCell
        {
            be.nbt.insert(
                JavaString::from("internalCurrentPower"),
                Tag::Double(energy.to_bits()),
            );
        }
        let proxy = be
            .nbt
            .entry(JavaString::from("proxy"))
            .or_insert_with(|| Tag::Compound(Compound::new()));
        if let Tag::Compound(proxy) = proxy {
            proxy.entry(JavaString::from("p")).or_insert(Tag::Int(-1));
            if state.buffer_share > 0.0 {
                proxy.insert(
                    JavaString::from("e"),
                    Tag::Double(state.buffer_share.to_bits()),
                );
            } else {
                proxy.remove(&JavaString::from("e"));
            }
        }
        if be.nbt != before {
            game.world.modified.insert((x >> 4, z >> 4));
        }
    }
    // ControllerBlockEntity.updateState uses UPDATE_CLIENTS, preserving the BE
    // and deliberately avoiding neighbour notifications during spatial moves.
    for (pos, model) in controllers {
        if game.native_block(pos) != super::ae2_controller::BLOCK {
            continue;
        }
        let neighbours = Direction::ALL.map(|direction| {
            let delta = direction.offset();
            game.native_block(std::array::from_fn(|i| pos[i].wrapping_add(delta[i])))
                == super::ae2_controller::BLOCK
        });
        let kind = super::ae2_controller::render_type(pos, neighbours);
        let flags = horizon_world::level::flags::UPDATE_CLIENTS;
        game.native_set_block_property_with(pos, "state", model, flags);
        game.native_set_block_property_with(pos, "type", kind, flags);
    }
    let cell_updates = NETWORKS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(dimension)
        .map(|state| state.cells.collect(game))
        .unwrap_or_default();
    super::ae2_energy_cell_glue::apply(game, cell_updates);
}
pub(super) fn chest_online(dimension: &str, pos: [i32; 3]) -> bool {
    NETWORKS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(dimension)
        .is_some_and(|state| state.network.is_online(pos))
}
/// Grid-powered insert or extract through the chest node's storage service.
pub(super) fn chest_transfer(
    dimension: &str,
    pos: [i32; 3],
    key: &super::ae2_keys::Key,
    amount: u64,
    insert: bool,
) -> u64 {
    let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    let Some(state) = networks.get_mut(dimension) else {
        return 0;
    };
    if !state.network.is_online(pos) {
        return 0;
    }
    let simulated = state.drives.transfer(pos, key, amount, insert, false);
    if simulated == 0 {
        return 0;
    }
    let available = state.network.extract(pos, simulated as f64, 1.0, false);
    let moved = ((available + 0.9) as u64).min(simulated);
    if moved == 0 {
        return 0;
    }
    state.network.extract(pos, moved as f64, 1.0, true);
    state.drives.transfer(pos, key, moved, insert, true)
}
pub(super) fn chest_first_item(dimension: &str, pos: [i32; 3]) -> Option<super::ae2_keys::Key> {
    let networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    let state = networks.get(dimension)?;
    if !state.network.is_online(pos) {
        return None;
    }
    state.drives.first_item(pos)
}
pub(super) fn extract(
    dimension: &str,
    pos: [i32; 3],
    amount: f64,
    multiplier: f64,
    commit: bool,
) -> f64 {
    let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    networks
        .get_mut(dimension)
        .map_or(0.0, |n| n.network.extract(pos, amount, multiplier, commit))
}
pub(super) fn reset() {
    NETWORKS.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

pub(super) fn cell_stored(dimension: &str, pos: [i32; 3], kind: Kind, saved: f64) -> Option<f64> {
    NETWORKS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(dimension)?
        .network
        .storage_for_snapshot(pos, kind, saved)
}

pub(super) fn node_placed(dimension: &str, pos: [i32; 3]) {
    if let Some(state) = NETWORKS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_mut(dimension)
    {
        state.network.forget_node(pos);
    }
}
pub(super) fn flush_drives(game: &mut Game<'_>) {
    if game.serving.is_none() && game.options.dedicated.is_none() {
        return;
    }
    let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(state) = networks.get_mut(game.dim.id()) {
        state.drives.read(game);
        state.drives.reconcile(&mut state.network);
        state.drives.persist(game);
    }
}

pub(super) fn sync_chargers(game: &mut Game<'_>) {
    if game.serving.is_none() && game.options.dedicated.is_none() {
        return;
    }
    let mut networks = NETWORKS.lock().unwrap_or_else(|e| e.into_inner());
    let state = networks.entry(game.dim.id().to_owned()).or_default();
    state.chargers.sync(game);
}
