// SPDX-License-Identifier: LGPL-3.0-or-later
// EnergyService / GridEnergyStorage / EnergyOverlayGrid rules from pinned AE2.
// Copyright AlgorithmX2 / TeamAppliedEnergistics; 2026 Horizon port contributors.
// World capabilities, event delivery and persistence adapters live elsewhere.

use super::ae2_energy::{MAX_MAXIMUM, PowerEvent, StoredEnergy};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
pub type NodeId = u64;
pub type ServiceId = u64;
const LOCAL: u64 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    None,
    Read,
    Write,
    ReadWrite,
}
impl Flow {
    fn reads(self) -> bool {
        matches!(self, Self::Read | Self::ReadWrite)
    }
    fn writes(self) -> bool {
        matches!(self, Self::Write | Self::ReadWrite)
    }
}

#[derive(Clone, Debug)]
struct SharedEnergy {
    energy: StoredEnergy,
    events: Vec<PowerEvent>,
}
/// One powered host's actual store, independent of its managed grid node.
/// Joining/leaving a service clones this handle, never its energy amount.
/// The level binding drains owner events outside energy-service locks.
#[derive(Clone, Debug)]
pub struct SharedPower(Arc<Mutex<SharedEnergy>>);
impl SharedPower {
    pub fn new(amount: f64, maximum: f64) -> Self {
        Self(Arc::new(Mutex::new(SharedEnergy {
            energy: StoredEnergy::new(amount, maximum),
            events: Vec::new(),
        })))
    }
    pub fn same_storage(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn amount(&self) -> f64 {
        self.0.lock().unwrap().energy.amount()
    }
    pub fn maximum(&self) -> f64 {
        self.0.lock().unwrap().energy.maximum()
    }
    pub fn take_events(&self) -> Vec<PowerEvent> {
        std::mem::take(&mut self.0.lock().unwrap().events)
    }
    pub fn insert(&self, amount: f64, commit: bool) -> f64 {
        self.insert_with_events(amount, commit).0
    }
    fn insert_with_events(&self, amount: f64, commit: bool) -> (f64, Vec<PowerEvent>) {
        let mut shared = self.0.lock().unwrap();
        let result = shared.energy.insert(amount, commit);
        shared.events.extend_from_slice(&result.1);
        result
    }
    pub fn extract(&self, amount: f64, commit: bool) -> f64 {
        self.extract_with_events(amount, commit).0
    }
    fn extract_with_events(&self, amount: f64, commit: bool) -> (f64, Vec<PowerEvent>) {
        let mut shared = self.0.lock().unwrap();
        let result = shared.energy.extract(amount, commit);
        shared.events.extend_from_slice(&result.1);
        result
    }
    pub fn set_stored(&self, amount: f64) {
        self.set_with_events(amount);
    }
    fn set_with_events(&self, amount: f64) -> Vec<PowerEvent> {
        let mut shared = self.0.lock().unwrap();
        let events = shared.energy.set_stored(amount);
        shared.events.extend_from_slice(&events);
        events
    }
    pub fn set_maximum(&self, maximum: f64) {
        self.resize_with_events(maximum);
    }
    fn resize_with_events(&self, maximum: f64) -> Vec<PowerEvent> {
        let mut shared = self.0.lock().unwrap();
        let events = shared.energy.set_maximum(maximum);
        shared.events.extend_from_slice(&events);
        events
    }
}

#[derive(Clone, Debug)]
enum StoreKind {
    Finite(StoredEnergy),
    Shared(SharedPower),
    Creative,
}
#[derive(Clone, Debug)]
pub struct Store {
    kind: StoreKind,
    pub priority: i32,
    pub flow: Flow,
    pub public: bool,
}
impl Store {
    pub fn shared(power: SharedPower, priority: i32, flow: Flow) -> Self {
        Self {
            kind: StoreKind::Shared(power),
            priority,
            flow,
            public: true,
        }
    }
    pub fn finite(amount: f64, maximum: f64, priority: i32) -> Self {
        Self {
            kind: StoreKind::Finite(StoredEnergy::new(amount, maximum)),
            priority,
            flow: Flow::ReadWrite,
            public: true,
        }
    }
    pub fn creative() -> Self {
        Self {
            kind: StoreKind::Creative,
            priority: i32::MAX,
            flow: Flow::ReadWrite,
            public: true,
        }
    }
    pub fn amount(&self) -> f64 {
        match &self.kind {
            StoreKind::Finite(s) => s.amount(),
            StoreKind::Shared(s) => s.amount(),
            StoreKind::Creative => MAX_MAXIMUM,
        }
    }
    pub fn maximum(&self) -> f64 {
        match &self.kind {
            StoreKind::Finite(s) => s.maximum(),
            StoreKind::Shared(s) => s.maximum(),
            StoreKind::Creative => MAX_MAXIMUM,
        }
    }
    fn insert(&mut self, amount: f64, commit: bool) -> (f64, Vec<PowerEvent>) {
        match &mut self.kind {
            StoreKind::Finite(s) => {
                let (inserted, events) = s.insert(amount, commit);
                (amount - inserted, events)
            }
            StoreKind::Creative => (0.0, vec![]),
            StoreKind::Shared(s) => {
                let (inserted, events) = s.insert_with_events(amount, commit);
                (amount - inserted, events)
            }
        }
    }
    fn extract(&mut self, amount: f64, commit: bool) -> (f64, Vec<PowerEvent>) {
        match &mut self.kind {
            StoreKind::Finite(s) => s.extract(amount, commit),
            StoreKind::Shared(s) => s.extract_with_events(amount, commit),
            StoreKind::Creative => (amount, vec![]),
        }
    }
    fn set(&mut self, amount: f64) -> Vec<PowerEvent> {
        match &mut self.kind {
            StoreKind::Finite(s) => s.set_stored(amount),
            StoreKind::Shared(s) => s.set_with_events(amount),
            StoreKind::Creative => vec![],
        }
    }
    fn resize(&mut self, maximum: f64) -> Vec<PowerEvent> {
        match &mut self.kind {
            StoreKind::Finite(s) => s.set_maximum(maximum),
            StoreKind::Shared(s) => s.resize_with_events(maximum),
            StoreKind::Creative => vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnergyNode {
    pub idle: f64,
    pub store: Option<Store>,
    pub passive_rate: Option<f64>,
}
impl Default for EnergyNode {
    fn default() -> Self {
        Self {
            idle: 1.0,
            store: None,
            passive_rate: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct RemovedNode {
    pub node: EnergyNode,
    pub saved_buffer: f64,
}
#[derive(Clone, Debug, Default)]
pub struct TickResult {
    pub power_changed: Option<bool>,
    pub thresholds: Vec<u64>,
}

/// One ME grid's energy service. IDs break equal-priority ties deterministically;
/// upstream uses identityHashCode (not a portable ordering contract).
#[derive(Clone, Debug)]
pub struct EnergyService {
    nodes: BTreeMap<NodeId, EnergyNode>,
    stores: BTreeMap<u64, Store>,
    providers: BTreeSet<(Reverse<i32>, u64)>,
    requesters: BTreeSet<(i32, Reverse<u64>)>,
    buffer_per_node: f64,
    drain: f64,
    avg_drain: f64,
    avg_injection: f64,
    tick_drain: f64,
    tick_injection: f64,
    available: f64,
    available_age: u32,
    provider_maximum: f64,
    public_power: bool,
    has_power: bool,
    ticks_since_power_change: u64,
    thresholds: BTreeMap<u64, f64>,
    last_stored: f64,
}

impl Default for EnergyService {
    fn default() -> Self {
        Self::new(25.0)
    }
}
impl EnergyService {
    pub fn new(buffer_per_node: f64) -> Self {
        assert!(buffer_per_node.is_finite() && buffer_per_node >= 0.0);
        let mut stores = BTreeMap::new();
        stores.insert(LOCAL, Store::finite(0.0, 0.0, i32::MIN));
        Self {
            nodes: BTreeMap::new(),
            stores,
            providers: BTreeSet::from([(Reverse(i32::MIN), LOCAL)]),
            requesters: BTreeSet::from([(i32::MIN, Reverse(LOCAL))]),
            buffer_per_node,
            drain: 0.0,
            avg_drain: 0.0,
            avg_injection: 0.0,
            tick_drain: 0.0,
            tick_injection: 0.0,
            available: 0.0,
            available_age: 0,
            provider_maximum: 0.0,
            public_power: false,
            has_power: true,
            ticks_since_power_change: 900,
            thresholds: BTreeMap::new(),
            last_stored: -1.0,
        }
    }
    fn store_id(node: NodeId) -> u64 {
        node.checked_add(1).expect("node ID overflow")
    }
    fn events(&mut self, id: u64, events: Vec<PowerEvent>) {
        let store = &self.stores[&id];
        if !store.public {
            return;
        }
        for event in events {
            match event {
                PowerEvent::Provide if store.flow.reads() => {
                    self.providers.insert((Reverse(store.priority), id));
                }
                PowerEvent::Receive if store.flow.writes() => {
                    self.requesters.insert((store.priority, Reverse(id)));
                }
                _ => {}
            }
        }
    }
    fn resize_buffer(&mut self) {
        let events = self
            .stores
            .get_mut(&LOCAL)
            .unwrap()
            .resize((self.buffer_per_node * self.nodes.len() as f64).min(MAX_MAXIMUM));
        self.events(LOCAL, events);
    }
    pub fn add_node(
        &mut self,
        id: NodeId,
        node: EnergyNode,
        saved_buffer: f64,
    ) -> Result<(), &'static str> {
        if self.nodes.contains_key(&id) {
            return Err("node already belongs to energy service");
        }
        if !node.idle.is_finite()
            || node.idle < 0.0
            || node.passive_rate.is_some_and(|v| !v.is_finite() || v < 0.0)
            || !saved_buffer.is_finite()
            || saved_buffer < 0.0
        {
            return Err("invalid node power");
        }
        let store = node.store.clone();
        self.drain += node.idle;
        self.nodes.insert(id, node);
        self.resize_buffer();
        if let Some(store) = store {
            let sid = Self::store_id(id);
            if store.public {
                if store.flow.reads() {
                    self.available += store.amount();
                    self.provider_maximum += store.maximum();
                    self.providers.insert((Reverse(store.priority), sid));
                }
                if store.flow.writes() {
                    self.requesters.insert((store.priority, Reverse(sid)));
                }
            }
            self.stores.insert(sid, store);
        }
        if saved_buffer > 0.0 {
            let (_, events) = self
                .stores
                .get_mut(&LOCAL)
                .unwrap()
                .insert(saved_buffer, true);
            self.events(LOCAL, events);
        }
        Ok(())
    }
    pub fn remove_node(&mut self, id: NodeId) -> Option<RemovedNode> {
        if !self.nodes.contains_key(&id) {
            return None;
        }
        let saved_buffer = self.node_energy_share();
        if saved_buffer > 0.0 {
            let (_, events) = self
                .stores
                .get_mut(&LOCAL)
                .unwrap()
                .extract(saved_buffer, true);
            self.events(LOCAL, events);
        }
        let mut node = self.nodes.remove(&id).unwrap();
        self.resize_buffer();
        self.drain -= node.idle;
        let sid = Self::store_id(id);
        if let Some(store) = self.stores.remove(&sid) {
            if store.public {
                if store.flow != Flow::Write {
                    self.provider_maximum -= store.maximum();
                    self.available -= store.amount();
                }
                self.providers.remove(&(Reverse(store.priority), sid));
                self.requesters.remove(&(store.priority, Reverse(sid)));
            }
            node.store = Some(store);
        }
        Some(RemovedNode { node, saved_buffer })
    }
    pub fn set_idle(&mut self, id: NodeId, idle: f64) -> Result<(), &'static str> {
        if !idle.is_finite() || idle < 0.0 {
            return Err("invalid idle power");
        }
        let node = self.nodes.get_mut(&id).ok_or("unknown node")?;
        self.drain += idle - node.idle;
        node.idle = idle;
        Ok(())
    }
    pub fn store(&self, id: NodeId) -> Option<&Store> {
        self.stores.get(&Self::store_id(id))
    }
    /// Source storagePowerChangeHandler path for an event the powered host
    /// delivered. Local host work uses this after draining its owner queue.
    pub fn owner_storage_event(
        &mut self,
        id: NodeId,
        event: PowerEvent,
    ) -> Result<(), &'static str> {
        let sid = Self::store_id(id);
        if !self.stores.contains_key(&sid) {
            return Err("unknown power storage");
        }
        self.events(sid, vec![event]);
        Ok(())
    }
    pub fn buffer_amount(&self) -> f64 {
        self.stores[&LOCAL].amount()
    }
    pub fn node_energy_share(&self) -> f64 {
        if self.nodes.is_empty() {
            0.0
        } else {
            self.buffer_amount() / self.nodes.len() as f64
        }
    }
    pub fn maximum(&self) -> f64 {
        self.provider_maximum + self.stores[&LOCAL].maximum()
    }
    pub fn base_idle(&self) -> f64 {
        self.drain
    }
    pub fn is_powered(&self) -> bool {
        self.public_power
    }
    pub fn average_drain(&self) -> f64 {
        self.avg_drain
    }
    pub fn average_injection(&self) -> f64 {
        self.avg_injection
    }
    pub fn set_storage_amount(&mut self, id: NodeId, amount: f64) -> Result<(), &'static str> {
        if !amount.is_finite() {
            return Err("invalid stored energy");
        }
        let sid = Self::store_id(id);
        let events = self
            .stores
            .get_mut(&sid)
            .ok_or("unknown power storage")?
            .set(amount);
        self.events(sid, events);
        Ok(())
    }
    pub fn register_threshold(&mut self, id: u64, value: f64) -> Result<(), &'static str> {
        if !value.is_finite() {
            return Err("invalid threshold");
        }
        self.thresholds.insert(id, value);
        Ok(())
    }
    pub fn unregister_threshold(&mut self, id: u64) -> bool {
        self.thresholds.remove(&id).is_some()
    }
    pub fn refresh_power(&mut self) {
        self.available_age = 0;
        self.available = self
            .providers
            .iter()
            .map(|(_, id)| self.stores[id].amount())
            .sum();
    }
    pub fn stored_power(&mut self) -> f64 {
        if self.available_age > 90 {
            self.refresh_power();
        }
        self.available.max(0.0)
    }

    /// IAEPowerStorage insertion returns overflow. Requester lists are pruned
    /// only on committed overflow, then reactivated by RECEIVE_POWER events.
    pub fn inject_provider(&mut self, amount: f64, commit: bool) -> f64 {
        assert!(amount.is_finite() && amount >= 0.0);
        let mut left = amount;
        let order: Vec<_> = self.requesters.iter().copied().collect();
        for key @ (_, Reverse(id)) in order {
            if left <= 0.0 {
                break;
            }
            let (rest, events) = self.stores.get_mut(&id).unwrap().insert(left, commit);
            left = rest;
            self.events(id, events);
            if left > 0.0 && commit {
                self.requesters.remove(&key);
            }
        }
        let overflow = left.max(0.0);
        if commit {
            self.tick_injection += amount - overflow;
        }
        overflow
    }
    pub fn extract_provider(&mut self, amount: f64, commit: bool) -> f64 {
        assert!(amount.is_finite() && amount >= 0.0);
        let mut extracted = 0.0;
        let order: Vec<_> = self.providers.iter().copied().collect();
        for key @ (_, id) in order {
            if extracted >= amount {
                break;
            }
            let requested = amount - extracted;
            let (value, events) = self.stores.get_mut(&id).unwrap().extract(requested, commit);
            extracted += value;
            self.events(id, events);
            if value < requested && commit {
                self.providers.remove(&key);
            }
        }
        let result = extracted.min(amount);
        if commit {
            if extracted > amount {
                let (_, events) = self
                    .stores
                    .get_mut(&LOCAL)
                    .unwrap()
                    .insert(extracted - amount, true);
                self.events(LOCAL, events);
            }
            self.available -= result;
            self.tick_drain += result;
        }
        result
    }
    /// Upstream can report more than max_required: it stops between stores,
    /// rather than clamping the final store's capacity contribution.
    pub fn provider_demand(&self, max_required: f64) -> f64 {
        assert!(max_required.is_finite() && max_required >= 0.0);
        let mut required = 0.0;
        for (_, Reverse(id)) in &self.requesters {
            if required >= max_required {
                break;
            }
            let store = &self.stores[id];
            if store.flow != Flow::Read {
                required += (store.maximum() - store.amount()).max(0.0);
            }
        }
        required
    }
    fn before_idle(&mut self) -> Vec<u64> {
        let mut crossed = vec![];
        if !self.thresholds.is_empty() {
            let old = self.last_stored;
            self.last_stored = self.stored_power();
            let low = old.min(self.last_stored);
            let high = old.max(self.last_stored);
            crossed.extend(
                self.thresholds
                    .iter()
                    .filter(|(_, value)| **value >= low && **value <= high)
                    .map(|(id, _)| *id),
            );
        }
        self.avg_drain *= 39.0 / 40.0;
        self.avg_injection *= 39.0 / 40.0;
        self.avg_drain += self.tick_drain / 40.0;
        self.avg_injection += self.tick_injection / 40.0;
        self.tick_drain = 0.0;
        self.tick_injection = 0.0;
        crossed
    }
    fn after_idle(&mut self, currently_has_power: bool, thresholds: Vec<u64>) -> TickResult {
        if currently_has_power == self.has_power {
            self.ticks_since_power_change = self.ticks_since_power_change.wrapping_add(1);
        } else {
            self.ticks_since_power_change = 0;
        }
        self.has_power = currently_has_power;
        let mut changed = None;
        let next = if self.has_power && self.ticks_since_power_change > 30 {
            true
        } else if !self.has_power {
            false
        } else {
            self.public_power
        };
        if next != self.public_power {
            self.public_power = next;
            changed = Some(next);
        }
        self.available_age = self.available_age.saturating_add(1);
        TickResult {
            power_changed: changed,
            thresholds,
        }
    }
}

#[derive(Clone, Debug)]
struct Overlay {
    services: Vec<ServiceId>,
    active_generator: Option<(ServiceId, NodeId)>,
}
#[derive(Clone, Debug, Default)]
pub struct EnergySystem {
    services: BTreeMap<ServiceId, EnergyService>,
    links: BTreeMap<(ServiceId, ServiceId), usize>,
    overlays: BTreeMap<ServiceId, Overlay>,
    overlay_for: BTreeMap<ServiceId, ServiceId>,
}
impl EnergySystem {
    pub fn add_service(
        &mut self,
        id: ServiceId,
        service: EnergyService,
    ) -> Result<(), &'static str> {
        if self.services.contains_key(&id) {
            return Err("duplicate energy service");
        }
        self.services.insert(id, service);
        Ok(())
    }
    pub fn service(&self, id: ServiceId) -> Option<&EnergyService> {
        self.services.get(&id)
    }
    pub fn service_mut(&mut self, id: ServiceId) -> Option<&mut EnergyService> {
        self.services.get_mut(&id)
    }
    /// Grid removal immediately clears the overlay's current passive generator.
    /// Other generators are reconsidered on the next server-start tick.
    pub fn remove_node(&mut self, service: ServiceId, node: NodeId) -> Option<RemovedNode> {
        let removed = self.services.get_mut(&service)?.remove_node(node)?;
        if let Some(root) = self.overlay_for.get(&service)
            && let Some(overlay) = self.overlays.get_mut(root)
            && overlay.active_generator == Some((service, node))
        {
            overlay.active_generator = None;
        }
        Some(removed)
    }
    fn invalidate(&mut self, service: ServiceId) {
        if let Some(root) = self.overlay_for.get(&service).copied()
            && let Some(overlay) = self.overlays.remove(&root)
        {
            for id in overlay.services {
                self.overlay_for.remove(&id);
            }
        }
    }
    pub fn connect(&mut self, a: ServiceId, b: ServiceId) -> Result<(), &'static str> {
        if a == b || !self.services.contains_key(&a) || !self.services.contains_key(&b) {
            return Err("invalid overlay connection");
        }
        *self.links.entry((a.min(b), a.max(b))).or_default() += 1;
        self.invalidate(a);
        self.invalidate(b);
        Ok(())
    }
    pub fn disconnect(&mut self, a: ServiceId, b: ServiceId) -> bool {
        let key = (a.min(b), a.max(b));
        let Some(count) = self.links.get_mut(&key) else {
            return false;
        };
        *count -= 1;
        if *count == 0 {
            self.links.remove(&key);
        }
        self.invalidate(a);
        self.invalidate(b);
        true
    }
    pub fn remove_service(&mut self, id: ServiceId) -> Option<EnergyService> {
        let service = self.services.remove(&id)?;
        self.links.retain(|(a, b), _| *a != id && *b != id);
        self.invalidate(id);
        Some(service)
    }
    fn overlay(&mut self, start: ServiceId) -> ServiceId {
        assert!(self.services.contains_key(&start));
        if let Some(id) = self.overlay_for.get(&start) {
            return *id;
        }
        let mut found = BTreeSet::new();
        let mut stack = vec![start];
        while let Some(id) = stack.pop() {
            if !found.insert(id) {
                continue;
            }
            for (a, b) in self.links.keys() {
                if *a == id {
                    stack.push(*b);
                } else if *b == id {
                    stack.push(*a);
                }
            }
        }
        let root = *found.first().unwrap();
        let mut services: Vec<_> = found.into_iter().collect();
        services.sort_by(|a, b| {
            self.services[b]
                .maximum()
                .total_cmp(&self.services[a].maximum())
                .then(a.cmp(b))
        });
        for id in &services {
            self.overlay_for.insert(*id, root);
        }
        self.overlays.insert(
            root,
            Overlay {
                services,
                active_generator: None,
            },
        );
        root
    }
    pub fn connected_services(&mut self, start: ServiceId) -> Vec<ServiceId> {
        let root = self.overlay(start);
        self.overlays[&root].services.clone()
    }
    pub fn extract(&mut self, start: ServiceId, amount: f64, multiplier: f64, commit: bool) -> f64 {
        assert!(multiplier.is_finite() && multiplier > 0.0 && amount.is_finite() && amount >= 0.0);
        let requested = amount * multiplier;
        assert!(requested.is_finite());
        let mut extracted = 0.0;
        for id in self.connected_services(start) {
            extracted += self
                .services
                .get_mut(&id)
                .unwrap()
                .extract_provider(requested - extracted, commit);
            if extracted >= requested {
                break;
            }
        }
        extracted / multiplier
    }
    pub fn inject(&mut self, start: ServiceId, amount: f64, commit: bool) -> f64 {
        let mut left = amount;
        for id in self.connected_services(start) {
            left = self
                .services
                .get_mut(&id)
                .unwrap()
                .inject_provider(left, commit);
            if left <= 0.0 {
                break;
            }
        }
        left
    }
    pub fn demand(&mut self, start: ServiceId, max_required: f64) -> f64 {
        let mut required = 0.0;
        for id in self.connected_services(start) {
            required += self.services[&id].provider_demand(max_required - required);
            if required >= max_required {
                break;
            }
        }
        required
    }
    pub fn begin_tick(&mut self) {
        let ids: Vec<_> = self.services.keys().copied().collect();
        for id in ids {
            self.overlay(id);
        }
        for overlay in self.overlays.values_mut() {
            let mut current = overlay.active_generator.filter(|(service, node)| {
                self.services
                    .get(service)
                    .is_some_and(|s| s.nodes.get(node).is_some_and(|n| n.passive_rate.is_some()))
            });
            for service in &overlay.services {
                for (node, spec) in &self.services[service].nodes {
                    if let Some(rate) = spec.passive_rate {
                        let old =
                            current.and_then(|(s, n)| self.services[&s].nodes[&n].passive_rate);
                        if old.is_none_or(|old| rate > old) {
                            current = Some((*service, *node));
                        }
                    }
                }
            }
            overlay.active_generator = current;
        }
    }
    pub fn generator_suppressed(&mut self, service: ServiceId, node: NodeId) -> bool {
        let root = self.overlay(service);
        self.overlays[&root].active_generator != Some((service, node))
    }
    pub fn end_tick(
        &mut self,
        service: ServiceId,
        channel_power: f64,
        multiplier: f64,
    ) -> TickResult {
        let root = self.overlay(service);
        if let Some((owner, node)) = self.overlays[&root].active_generator
            && owner == service
            && let Some(rate) = self.services[&owner]
                .nodes
                .get(&node)
                .and_then(|n| n.passive_rate)
        {
            self.inject(owner, rate, true);
        }
        let s = self.services.get_mut(&service).unwrap();
        let thresholds = s.before_idle();
        let base_idle = s.drain;
        // Preserve EnergyService's base-drain condition and comparison, even
        // though channel_power is added to the amount requested for extraction.
        let powered = if base_idle > 0.0001 {
            self.extract(service, base_idle + channel_power, multiplier, true) >= base_idle - 0.001
        } else {
            self.extract(service, 0.1, multiplier, false) > 0.0
        };
        self.services
            .get_mut(&service)
            .unwrap()
            .after_idle(powered, thresholds)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn shared_power_owner_queue_and_grid_write_flow_preserve_actual_store_identity() {
        use super::*;
        let power = SharedPower::new(0.0, 500.0);
        // As in the owning block's initial load, initialize threshold state
        // before charging; StoredEnergyAmount's constructor starts eligible.
        power.set_stored(0.0);
        let cloned = power.clone();
        assert!(cloned.same_storage(&power));
        let mut service = EnergyService::default();
        service
            .add_node(
                1,
                EnergyNode {
                    idle: 0.0,
                    store: Some(Store::shared(power.clone(), 19, Flow::Write)),
                    passive_rate: None,
                },
                0.0,
            )
            .unwrap();
        assert_eq!(service.inject_provider(1000.0, true), 475.0);
        assert_eq!(power.amount(), 500.0);
        assert_eq!(cloned.take_events(), [PowerEvent::Provide]);
        assert!(power.take_events().is_empty());
        // The service may use its private 25 AE buffer, but cannot drain the
        // Chest's local 500 AE. The owner can, without copying an amount.
        assert_eq!(service.extract_provider(1000.0, true), 25.0);
        assert_eq!(power.extract(10.0, false), 10.0);
        assert_eq!(power.amount(), 500.0);
        assert!(power.take_events().is_empty());
        assert_eq!(power.extract(10.0, true), 10.0);
        assert_eq!(power.take_events(), [PowerEvent::Receive]);
        service.owner_storage_event(1, PowerEvent::Receive).unwrap();
        assert_eq!(service.inject_provider(40.0, true), 5.0);
        assert_eq!(service.store(1).unwrap().amount(), 500.0);
        let removed = service.remove_node(1).unwrap();
        assert_eq!(power.extract(7.0, true), 7.0);
        service
            .add_node(1, removed.node, removed.saved_buffer)
            .unwrap();
        assert_eq!(service.store(1).unwrap().amount(), 493.0);
    }
    use super::*;
    fn spec(idle: f64, store: Option<Store>) -> EnergyNode {
        EnergyNode {
            idle,
            store,
            passive_rate: None,
        }
    }
    fn one() -> EnergySystem {
        let mut s = EnergySystem::default();
        s.add_service(0, EnergyService::default()).unwrap();
        s
    }
    #[test]
    fn node_buffer_share_survives_move_and_removal_is_proportional() {
        let mut a = EnergyService::default();
        a.add_node(1, spec(1.0, None), 0.0).unwrap();
        a.add_node(2, spec(2.0, None), 0.0).unwrap();
        assert_eq!(a.maximum(), 50.0);
        assert_eq!(a.inject_provider(40.0, true), 0.0);
        assert_eq!(a.node_energy_share(), 20.0);
        let moved = a.remove_node(1).unwrap();
        assert_eq!(moved.saved_buffer, 20.0);
        assert_eq!(a.buffer_amount(), 20.0);
        assert_eq!(a.base_idle(), 2.0);
        let mut b = EnergyService::default();
        b.add_node(1, moved.node, moved.saved_buffer).unwrap();
        assert_eq!(b.buffer_amount(), 20.0);
    }
    #[test]
    fn extraction_prioritizes_dense_and_insertion_prioritizes_local_buffer() {
        let mut a = EnergyService::default();
        a.add_node(1, spec(0.0, Some(Store::finite(0.0, 200000.0, 200))), 0.0)
            .unwrap();
        a.add_node(2, spec(0.0, Some(Store::finite(0.0, 1600000.0, 1600))), 0.0)
            .unwrap();
        assert_eq!(a.inject_provider(150.0, true), 0.0);
        assert_eq!(a.buffer_amount(), 50.0);
        assert_eq!(a.store(1).unwrap().amount(), 100.0);
        assert_eq!(a.store(2).unwrap().amount(), 0.0);
        a.set_storage_amount(2, 70.0).unwrap();
        assert_eq!(a.extract_provider(80.0, true), 80.0);
        assert_eq!(a.store(2).unwrap().amount(), 0.0);
        assert_eq!(a.store(1).unwrap().amount(), 90.0);
    }
    #[test]
    fn simulation_does_not_change_contents_or_remove_exhausted_providers() {
        let mut a = EnergyService::default();
        a.add_node(1, spec(0.0, Some(Store::finite(10.0, 10.0, 100))), 0.0)
            .unwrap();
        assert_eq!(a.extract_provider(100.0, false), 10.0);
        assert_eq!(a.store(1).unwrap().amount(), 10.0);
        assert_eq!(a.inject_provider(100.0, false), 75.0);
        assert_eq!(a.buffer_amount(), 0.0);
        assert_eq!(a.extract_provider(5.0, true), 5.0);
    }
    #[test]
    fn exhausted_storage_rejoins_via_threshold_events() {
        let mut a = EnergyService::default();
        a.add_node(1, spec(0.0, Some(Store::finite(1.0, 10.0, 100))), 0.0)
            .unwrap();
        assert_eq!(a.extract_provider(5.0, true), 1.0);
        a.set_storage_amount(1, 2.0).unwrap();
        assert_eq!(a.extract_provider(1.0, true), 1.0);
        a.inject_provider(1000.0, true);
        assert_eq!(a.inject_provider(1.0, true), 1.0);
        a.extract_provider(2.0, true);
        assert_eq!(a.inject_provider(1.0, true), 0.0);
    }
    #[test]
    fn flow_restrictions_and_private_buffers_are_honored() {
        let mut a = EnergyService::default();
        let mut private = Store::finite(10.0, 10.0, 100);
        private.public = false;
        a.add_node(1, spec(0.0, Some(private)), 0.0).unwrap();
        assert_eq!(a.extract_provider(1.0, true), 0.0);
        let mut write = Store::finite(10.0, 20.0, 100);
        write.flow = Flow::Write;
        a.add_node(2, spec(0.0, Some(write)), 0.0).unwrap();
        assert_eq!(a.extract_provider(1.0, true), 0.0);
    }
    #[test]
    fn creative_store_provides_without_depletion() {
        let mut a = EnergyService::default();
        a.add_node(1, spec(0.0, Some(Store::creative())), 0.0)
            .unwrap();
        assert_eq!(a.extract_provider(1e10, true), 1e10);
        assert_eq!(a.store(1).unwrap().amount(), MAX_MAXIMUM);
    }
    #[test]
    fn overlay_order_uses_total_capacity_and_multiset_links() {
        let mut s = one();
        s.add_service(1, EnergyService::new(50.0)).unwrap();
        s.service_mut(0)
            .unwrap()
            .add_node(1, spec(0.0, None), 0.0)
            .unwrap();
        s.service_mut(1)
            .unwrap()
            .add_node(1, spec(0.0, None), 0.0)
            .unwrap();
        s.connect(0, 1).unwrap();
        s.connect(0, 1).unwrap();
        assert_eq!(s.connected_services(0), vec![1, 0]);
        s.inject(0, 60.0, true);
        assert_eq!(s.service(1).unwrap().buffer_amount(), 50.0);
        assert_eq!(s.service(0).unwrap().buffer_amount(), 10.0);
        s.disconnect(0, 1);
        assert_eq!(s.connected_services(0).len(), 2);
        s.disconnect(0, 1);
        assert_eq!(s.connected_services(0), vec![0]);
    }
    #[test]
    fn multiplier_and_simulation_apply_once_across_overlay() {
        let mut s = one();
        s.service_mut(0)
            .unwrap()
            .add_node(1, spec(0.0, None), 20.0)
            .unwrap();
        assert_eq!(s.extract(0, 5.0, 2.0, false), 5.0);
        assert_eq!(s.service(0).unwrap().buffer_amount(), 20.0);
        assert_eq!(s.extract(0, 15.0, 2.0, true), 10.0);
        assert_eq!(s.service(0).unwrap().buffer_amount(), 0.0);
    }
    #[test]
    fn public_power_restarts_after_thirty_one_stable_ticks_and_drops_immediately() {
        let mut s = one();
        s.service_mut(0)
            .unwrap()
            .add_node(1, spec(0.0, None), 0.0)
            .unwrap();
        s.end_tick(0, 0.0, 1.0);
        assert!(!s.service(0).unwrap().is_powered());
        s.inject(0, 5.0, true);
        for _ in 0..31 {
            s.end_tick(0, 0.0, 1.0);
            assert!(!s.service(0).unwrap().is_powered());
        }
        assert_eq!(s.end_tick(0, 0.0, 1.0).power_changed, Some(true));
        s.extract(0, 5.0, 1.0, true);
        assert_eq!(s.end_tick(0, 0.0, 1.0).power_changed, Some(false));
    }
    #[test]
    fn only_strongest_passive_generator_runs_across_overlay() {
        let mut s = one();
        s.add_service(1, EnergyService::default()).unwrap();
        s.service_mut(0)
            .unwrap()
            .add_node(
                1,
                EnergyNode {
                    idle: 0.0,
                    store: None,
                    passive_rate: Some(5.0),
                },
                0.0,
            )
            .unwrap();
        s.service_mut(1)
            .unwrap()
            .add_node(
                1,
                EnergyNode {
                    idle: 0.0,
                    store: None,
                    passive_rate: Some(10.0),
                },
                0.0,
            )
            .unwrap();
        s.connect(0, 1).unwrap();
        s.begin_tick();
        assert!(s.generator_suppressed(0, 1));
        assert!(!s.generator_suppressed(1, 1));
        s.end_tick(0, 0.0, 1.0);
        s.end_tick(1, 0.0, 1.0);
        assert_eq!(
            s.service(0).unwrap().buffer_amount() + s.service(1).unwrap().buffer_amount(),
            10.0
        );
    }
    #[test]
    fn unrelated_link_changes_keep_another_overlays_active_generator() {
        let mut s = one();
        s.add_service(1, EnergyService::default()).unwrap();
        s.add_service(2, EnergyService::default()).unwrap();
        s.service_mut(2)
            .unwrap()
            .add_node(
                1,
                EnergyNode {
                    idle: 0.0,
                    store: None,
                    passive_rate: Some(5.0),
                },
                0.0,
            )
            .unwrap();
        s.begin_tick();
        s.connect(0, 1).unwrap();
        s.end_tick(2, 0.0, 1.0);
        assert_eq!(s.service(2).unwrap().buffer_amount(), 5.0);
        s.disconnect(0, 1);
        s.end_tick(2, 0.0, 1.0);
        assert_eq!(s.service(2).unwrap().buffer_amount(), 10.0);
    }
    #[test]
    fn removing_active_generator_clears_it_until_next_start_tick() {
        let mut s = one();
        for (id, rate) in [(1, 10.0), (2, 5.0)] {
            s.service_mut(0)
                .unwrap()
                .add_node(
                    id,
                    EnergyNode {
                        idle: 0.0,
                        store: None,
                        passive_rate: Some(rate),
                    },
                    0.0,
                )
                .unwrap();
        }
        s.begin_tick();
        assert!(!s.generator_suppressed(0, 1));
        s.remove_node(0, 1).unwrap();
        assert!(s.generator_suppressed(0, 1));
        s.end_tick(0, 0.0, 1.0);
        assert_eq!(s.service(0).unwrap().buffer_amount(), 0.0);
        s.begin_tick();
        s.end_tick(0, 0.0, 1.0);
        assert_eq!(s.service(0).unwrap().buffer_amount(), 5.0);
    }
    #[test]
    fn cache_refresh_and_inclusive_thresholds_follow_original_order() {
        let mut s = one();
        s.service_mut(0)
            .unwrap()
            .add_node(1, spec(0.0, None), 0.0)
            .unwrap();
        s.inject(0, 20.0, true);
        assert_eq!(s.service_mut(0).unwrap().stored_power(), 0.0);
        s.service_mut(0).unwrap().refresh_power();
        assert_eq!(s.service_mut(0).unwrap().stored_power(), 20.0);
        s.service_mut(0)
            .unwrap()
            .register_threshold(1, 20.0)
            .unwrap();
        assert_eq!(s.end_tick(0, 0.0, 1.0).thresholds, vec![1]);
        assert_eq!(s.end_tick(0, 0.0, 1.0).thresholds, vec![1]);
    }
    #[test]
    fn averages_are_updated_before_current_idle_drain() {
        let mut s = one();
        s.service_mut(0)
            .unwrap()
            .add_node(1, spec(1.0, None), 25.0)
            .unwrap();
        s.end_tick(0, 0.0, 1.0);
        assert_eq!(s.service(0).unwrap().average_drain(), 0.0);
        s.end_tick(0, 0.0, 1.0);
        assert_eq!(s.service(0).unwrap().average_drain(), 0.025);
    }
}
