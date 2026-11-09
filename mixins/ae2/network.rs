// SPDX-License-Identifier: LGPL-3.0-or-later
// World-independent binding of supported AE2 nodes to topology + energy.
// Cable center profiles are accepted here; the world multipart host/decoder,
// renderer and lifecycle are still separate unfinished adapter work.

use super::ae2_cable::{Cable, Color};
use super::ae2_grid::{
    ChannelMode, ConnectionId, ControllerState, Direction, Flags, Graph, NodeId, NodeSpec,
};
use super::ae2_power::{EnergyNode, EnergyService, EnergySystem, ServiceId, Store};
use std::collections::{BTreeMap, BTreeSet};
pub type Pos = [i32; 3];

/// One primary managed node at a full block/cable center or a particular part.
/// External tunnel/fiber nodes need distinct addresses when those parts land.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeKey {
    pub pos: Pos,
    pub side: Option<Direction>,
}
impl NodeKey {
    pub fn center(pos: Pos) -> Self {
        Self { pos, side: None }
    }
    pub fn part(pos: Pos, side: Direction) -> Self {
        Self {
            pos,
            side: Some(side),
        }
    }
    fn order(self) -> u8 {
        self.side.map_or(0, |d| 6 - d as u8)
    }
}
impl From<Pos> for NodeKey {
    fn from(pos: Pos) -> Self {
        Self::center(pos)
    }
}
impl From<(Pos, Direction)> for NodeKey {
    fn from((pos, side): (Pos, Direction)) -> Self {
        Self::part(pos, side)
    }
}
impl Ord for NodeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pos
            .cmp(&other.pos)
            .then_with(|| self.order().cmp(&other.order()))
    }
}
impl PartialOrd for NodeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Controller,
    Charger,
    Inscriber,
    EnergyCell,
    DenseEnergyCell,
    CreativeEnergyCell,
    Drive,
    Terminal,
    Cable(Cable),
}
impl Kind {
    pub fn from_block(id: &str) -> Option<Self> {
        match id {
            "ae2:controller" => Some(Self::Controller),
            "ae2:charger" => Some(Self::Charger),
            "ae2:inscriber" => Some(Self::Inscriber),
            "ae2:energy_cell" => Some(Self::EnergyCell),
            "ae2:dense_energy_cell" => Some(Self::DenseEnergyCell),
            "ae2:creative_energy_cell" => Some(Self::CreativeEnergyCell),
            "ae2:drive" => Some(Self::Drive),
            _ => None,
        }
    }
    pub fn block(self) -> &'static str {
        match self {
            Self::Controller => super::ae2_controller::BLOCK,
            Self::Charger => "ae2:charger",
            Self::Inscriber => "ae2:inscriber",
            Self::EnergyCell => "ae2:energy_cell",
            Self::DenseEnergyCell => "ae2:dense_energy_cell",
            Self::CreativeEnergyCell => "ae2:creative_energy_cell",
            Self::Drive => "ae2:drive",
            Self::Cable(_) | Self::Terminal => "ae2:cable_bus",
        }
    }
    pub fn capacity(self) -> f64 {
        match self {
            Self::Controller => super::ae2_controller::CAPACITY,
            Self::Charger | Self::Inscriber => 1600.0,
            Self::EnergyCell => 200000.0,
            Self::DenseEnergyCell => 1600000.0,
            Self::CreativeEnergyCell => super::ae2_energy::MAX_MAXIMUM,
            Self::Drive | Self::Cable(_) | Self::Terminal => 0.0,
        }
    }
    pub fn node_flags(self) -> Flags {
        match self {
            Self::Drive | Self::Terminal => Flags::REQUIRE_CHANNEL,
            Self::Controller => Flags::CANNOT_CARRY | Flags::DENSE_CAPACITY,
            Self::Cable(cable) => cable.kind.node_flags(),
            _ => Flags::default(),
        }
    }
    pub fn base_idle_power(self) -> f64 {
        match self {
            Self::Controller => super::ae2_controller::IDLE,
            Self::Drive => 2.0,
            Self::Terminal => 0.5,
            _ => 0.0,
        }
    }
    fn grid_color(self) -> Color {
        match self {
            Self::Cable(cable) => cable.color,
            _ => Color::Transparent,
        }
    }
    fn same_node(self, other: Self) -> bool {
        // CablePart.changeColor updates the existing managed node in place.
        match (self, other) {
            (Self::Cable(a), Self::Cable(b)) => a.kind == b.kind,
            _ => self == other,
        }
    }
    fn energy_node(self, amount: f64) -> EnergyNode {
        // EnergyCellBlockEntity.loadTag calls StoredEnergyAmount.setStored,
        // rather than the constructor's direct assignment. Preserve its
        // MIN_AMOUNT threshold even when this is the first loaded snapshot.
        let amount = if matches!(self, Self::EnergyCell | Self::DenseEnergyCell)
            && amount < super::ae2_energy::MIN_AMOUNT
        {
            0.0
        } else {
            amount
        };
        let store = match self {
            Self::Controller => Some(Store::finite(
                amount.clamp(0.0, self.capacity()),
                self.capacity(),
                super::ae2_controller::STORAGE_PRIORITY,
            )),
            Self::Charger | Self::Inscriber | Self::Drive | Self::Cable(_) | Self::Terminal => None,
            Self::EnergyCell => Some(Store::finite(
                amount.clamp(0.0, self.capacity()),
                self.capacity(),
                200,
            )),
            Self::DenseEnergyCell => Some(Store::finite(
                amount.clamp(0.0, self.capacity()),
                self.capacity(),
                1600,
            )),
            Self::CreativeEnergyCell => Some(Store::creative()),
        };
        EnergyNode {
            idle: self.base_idle_power(),
            store,
            passive_rate: None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub pos: Pos,
    pub side: Option<Direction>,
    pub kind: Kind,
    pub front: Direction,
    /// Exposed in-world faces; the multipart host removes blocked/occupied sides.
    pub exposed: u8,
    pub stored: f64,
    pub saved_buffer: f64,
}
impl Snapshot {
    pub fn key(self) -> NodeKey {
        NodeKey {
            pos: self.pos,
            side: self.side,
        }
    }
    pub fn terminal(pos: Pos, side: Direction, saved_buffer: f64) -> Self {
        Self {
            pos,
            side: Some(side),
            kind: Kind::Terminal,
            front: side,
            exposed: 0,
            stored: 0.0,
            saved_buffer,
        }
    }
    fn exposes(self, direction: Direction) -> bool {
        self.side.is_none()
            && self.kind != Kind::Terminal
            && self.exposed & (1 << direction as u8) != 0
            && (!matches!(self.kind, Kind::Charger | Kind::Inscriber | Kind::Drive)
                || self.front != direction)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Persisted {
    pub pos: Pos,
    pub side: Option<Direction>,
    pub kind: Kind,
    pub stored: Option<f64>,
    pub buffer_share: f64,
    pub powered: bool,
    pub controller_model: Option<&'static str>,
}
#[derive(Clone, Debug)]
struct Node {
    id: NodeId,
    snapshot: Snapshot,
    service: Option<ServiceId>,
}

#[derive(Clone, Debug, Default)]
pub struct Network {
    graph: Graph,
    energy: EnergySystem,
    nodes: BTreeMap<NodeKey, Node>,
    connections: BTreeMap<(NodeId, NodeId), ConnectionId>,
    pivots: BTreeMap<ServiceId, NodeId>,
    channel_power: BTreeMap<ServiceId, f64>,
    controller_states: BTreeMap<ServiceId, ControllerState>,
    next_service: ServiceId,
}
impl Network {
    fn new_service(&mut self, pivot: NodeId) -> ServiceId {
        let id = self.next_service;
        self.next_service += 1;
        self.energy
            .add_service(id, EnergyService::default())
            .expect("unique service ID");
        self.pivots.insert(id, pivot);
        id
    }
    /// Reconcile loaded nodes. A removed unloaded node's last persisted proxy.e
    /// already carries its share; active services deduct it proportionally.
    pub fn sync(&mut self, snapshots: Vec<Snapshot>) {
        let incoming: BTreeMap<_, _> = snapshots.into_iter().map(|s| (s.key(), s)).collect();
        let mut changed = false;
        let gone: Vec<_> = self
            .nodes
            .iter()
            .filter(|(p, n)| {
                incoming
                    .get(*p)
                    .is_none_or(|s| !s.kind.same_node(n.snapshot.kind))
            })
            .map(|(p, _)| *p)
            .collect();
        for pos in gone {
            let n = self.nodes.remove(&pos).unwrap();
            self.graph.remove_node(n.id);
            if let Some(service) = n.service {
                self.energy.remove_node(service, n.id);
            }
            self.connections
                .retain(|(a, b), _| *a != n.id && *b != n.id);
            changed = true;
        }
        for (pos, snapshot) in incoming {
            match self.nodes.get_mut(&pos) {
                Some(node) => {
                    if node.snapshot.front != snapshot.front
                        || node.snapshot.exposed != snapshot.exposed
                        || node.snapshot.kind != snapshot.kind
                    {
                        changed = true;
                    }
                    if node.snapshot.stored.to_bits() != snapshot.stored.to_bits()
                        && let Some(service) = node.service
                        && snapshot.kind != Kind::Charger
                    {
                        let _ = self
                            .energy
                            .service_mut(service)
                            .unwrap()
                            .set_storage_amount(node.id, snapshot.stored);
                    }
                    node.snapshot = snapshot;
                }
                None => {
                    let id = self.graph.add_node(NodeSpec {
                        flags: snapshot.kind.node_flags(),
                        controller: (snapshot.kind == Kind::Controller).then_some(snapshot.pos),
                        ..Default::default()
                    });
                    self.nodes.insert(
                        pos,
                        Node {
                            id,
                            snapshot,
                            service: None,
                        },
                    );
                    changed = true;
                }
            }
        }
        let mut desired = BTreeMap::new();
        for (key, node) in &self.nodes {
            for direction in Direction::ALL {
                if !self.exposes(*key, direction) {
                    continue;
                }
                let delta = direction.offset();
                let neighbor = std::array::from_fn(|i| key.pos[i] + delta[i]);
                let other_key = NodeKey::center(neighbor);
                if let Some(other) = self.nodes.get(&other_key)
                    && node.id < other.id
                    && self.exposes(other_key, direction.opposite())
                    && node
                        .snapshot
                        .kind
                        .grid_color()
                        .compatible(other.snapshot.kind.grid_color())
                {
                    desired.insert((node.id, other.id), Some(direction));
                }
            }
        }
        // CableBusContainer.addPart/addToWorld connects each primary part to
        // the center with GridHelper.createConnection, whose direction is null.
        for (key, node) in &self.nodes {
            if key.side.is_some()
                && let Some(center) = self.nodes.get(&NodeKey::center(key.pos))
                && matches!(center.snapshot.kind, Kind::Cable(_))
            {
                let pair = if center.id < node.id {
                    (center.id, node.id)
                } else {
                    (node.id, center.id)
                };
                desired.insert(pair, None);
            }
        }
        let removed: Vec<_> = self
            .connections
            .keys()
            .filter(|key| !desired.contains_key(key))
            .copied()
            .collect();
        for key in removed {
            self.graph
                .disconnect(self.connections.remove(&key).unwrap());
            changed = true;
        }
        for (key, direction) in desired {
            if let std::collections::btree_map::Entry::Vacant(entry) = self.connections.entry(key) {
                entry.insert(
                    self.graph
                        .connect(key.0, key.1, direction)
                        .expect("unique adjacent connection"),
                );
                changed = true;
            }
        }
        if !changed {
            return;
        }
        // On a split the component containing the old pivot retains its service.
        // On a merge the larger existing grid wins (all current kinds priority 0).
        // Equal-size tie order and full Grid service/event identity remain a port gap.
        for (service, pivot) in &mut self.pivots {
            if !self
                .nodes
                .values()
                .any(|n| n.id == *pivot && n.service == Some(*service))
                && let Some(first) = self
                    .nodes
                    .values()
                    .filter(|n| n.service == Some(*service))
                    .map(|n| n.id)
                    .min()
            {
                *pivot = first;
            }
        }
        let reports = self.graph.calculate(ChannelMode::Default);
        let old_sizes: BTreeMap<_, _> = self
            .pivots
            .keys()
            .map(|s| {
                (
                    *s,
                    self.nodes
                        .values()
                        .filter(|n| n.service == Some(*s))
                        .count(),
                )
            })
            .collect();
        let mut assignments = BTreeMap::new();
        self.channel_power.clear();
        self.controller_states.clear();
        for report in reports {
            let candidates: Vec<_> = self
                .pivots
                .iter()
                .filter(|(_, pivot)| report.nodes.contains(pivot))
                .map(|(s, _)| *s)
                .collect();
            let winner = candidates
                .into_iter()
                .max_by_key(|s| (old_sizes[s], std::cmp::Reverse(*s)))
                .unwrap_or_else(|| self.new_service(*report.nodes.first().unwrap()));
            self.channel_power
                .insert(winner, report.channel_power_usage());
            self.controller_states.insert(winner, report.controller);
            for id in report.nodes {
                assignments.insert(id, winner);
            }
        }
        let moving: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.service != Some(assignments[&n.id]))
            .map(|(p, _)| *p)
            .collect();
        let mut transfers = BTreeMap::new();
        for pos in &moving {
            let node = &self.nodes[pos];
            if let Some(service) = node.service {
                let removed = self
                    .energy
                    .remove_node(service, node.id)
                    .expect("registered energy node");
                transfers.insert(*pos, removed);
            }
        }
        for pos in moving {
            let node = self.nodes.get_mut(&pos).unwrap();
            let service = assignments[&node.id];
            let (spec, buffer) = match transfers.remove(&pos) {
                Some(t) => (t.node, t.saved_buffer),
                None => (
                    node.snapshot.kind.energy_node(node.snapshot.stored),
                    node.snapshot.saved_buffer,
                ),
            };
            self.energy
                .service_mut(service)
                .unwrap()
                .add_node(node.id, spec, buffer)
                .expect("valid full-block energy node");
            node.service = Some(service);
        }
        let used: BTreeSet<_> = self.nodes.values().filter_map(|n| n.service).collect();
        let unused: Vec<_> = self
            .pivots
            .keys()
            .filter(|s| !used.contains(s))
            .copied()
            .collect();
        for service in unused {
            self.energy.remove_service(service);
            self.pivots.remove(&service);
        }
    }
    fn exposes(&self, key: NodeKey, direction: Direction) -> bool {
        let Some(node) = self.nodes.get(&key) else {
            return false;
        };
        node.snapshot.exposes(direction)
            && (!matches!(node.snapshot.kind, Kind::Cable(_))
                || !self.nodes.contains_key(&NodeKey::part(key.pos, direction)))
    }
    pub fn begin_tick(&mut self) {
        self.energy.begin_tick();
    }
    /// A real placement created a new block entity, even if its position,
    /// kind and saved amounts happen to equal the old one's last snapshot.
    pub fn forget_node(&mut self, key: impl Into<NodeKey>) {
        let key = key.into();
        if self.nodes.contains_key(&key) {
            let remaining = self
                .nodes
                .iter()
                .filter(|(k, _)| **k != key)
                .map(|(_, node)| node.snapshot)
                .collect();
            self.sync(remaining);
        }
    }
    /// Destruction/replacement of the physical BE removes all hosted nodes.
    /// Removing/replacing just one part must use forget_node with its slot.
    pub fn forget_host(&mut self, pos: Pos) {
        let remaining = self
            .nodes
            .values()
            .filter(|node| node.snapshot.pos != pos)
            .map(|node| node.snapshot)
            .collect();
        self.sync(remaining);
    }
    /// Read live storage only if the BE still echoes this node's last saved
    /// amount. A newer external /data replacement must not be overwritten.
    pub fn storage_for_snapshot(&self, pos: Pos, kind: Kind, saved: f64) -> Option<f64> {
        let node = self.nodes.get(&NodeKey::center(pos))?;
        if node.snapshot.kind != kind || node.snapshot.stored.to_bits() != saved.to_bits() {
            return None;
        }
        self.energy
            .service(node.service?)?
            .store(node.id)
            .map(Store::amount)
    }
    pub fn extract(
        &mut self,
        key: impl Into<NodeKey>,
        amount: f64,
        multiplier: f64,
        commit: bool,
    ) -> f64 {
        let Some(service) = self.nodes.get(&key.into()).and_then(|n| n.service) else {
            return 0.0;
        };
        self.energy.extract(service, amount, multiplier, commit)
    }
    pub fn end_tick(&mut self) -> Vec<Persisted> {
        for (service, channel_power) in self.channel_power.clone() {
            self.energy.end_tick(service, channel_power, 1.0);
        }
        self.nodes
            .values_mut()
            .map(|node| {
                let service = self.energy.service(node.service.unwrap()).unwrap();
                let stored = service.store(node.id).map(Store::amount);
                if let Some(value) = stored {
                    node.snapshot.stored = value;
                }
                node.snapshot.saved_buffer = service.node_energy_share();
                Persisted {
                    pos: node.snapshot.pos,
                    side: node.snapshot.side,
                    kind: node.snapshot.kind,
                    stored,
                    buffer_share: node.snapshot.saved_buffer,
                    powered: service.is_powered(),
                    controller_model: (node.snapshot.kind == Kind::Controller).then(|| {
                        super::ae2_controller::status(
                            service.is_powered(),
                            self.controller_states.get(&node.service.unwrap())
                                == Some(&ControllerState::Conflict),
                        )
                    }),
                }
            })
            .collect()
    }
    pub fn is_powered(&self, key: impl Into<NodeKey>) -> bool {
        self.nodes
            .get(&key.into())
            .and_then(|n| n.service)
            .and_then(|s| self.energy.service(s))
            .is_some_and(EnergyService::is_powered)
    }
    pub fn cable_visual(&self, pos: Pos) -> Option<super::ae2_cable::VisualState> {
        let mut state = self.cable_stream(pos)?;
        let Kind::Cable(cable) = self.nodes.get(&NodeKey::center(pos))?.snapshot.kind else {
            return None;
        };
        // CableBusContainer.getRenderState applies the smart-type gate in
        // addition to CablePart.getChannelsOnSide's power gate.
        if !cable.kind.is_smart() || !self.is_powered(pos) {
            state.channels = [0; 6];
        }
        Some(state)
    }
    /// Raw source stream channels are independent from the rendering gate in
    /// CablePart.getChannelsOnSide and CableBusContainer.getRenderState.
    pub fn cable_stream(&self, pos: Pos) -> Option<super::ae2_cable::VisualState> {
        let node = self.nodes.get(&NodeKey::center(pos))?;
        let Kind::Cable(cable) = node.snapshot.kind else {
            return None;
        };
        let grid_node = self.graph.node(node.id)?;
        let mut state = super::ae2_cable::VisualState::default();
        for id in grid_node.connections() {
            let connection = self.graph.connection(*id)?;
            if let Some(side) = connection.direction(node.id) {
                state.connections |= 1 << side as u8;
                state.channels[side as usize] = i32::from(cable.kind.visual_channels(
                    connection.used_channels() as i32,
                    grid_node.max_channels(ChannelMode::Default) as i32,
                    ChannelMode::Default,
                    true,
                ));
            }
        }
        // Stream channels for internal side parts without setting a connected
        // in-world face. A real external connection (if present) wins later.
        for side in Direction::ALL {
            if state.connections & (1 << side as u8) != 0 {
                continue;
            }
            if let Some(part) = self.nodes.get(&NodeKey::part(pos, side)) {
                let max = self
                    .graph
                    .node(part.id)?
                    .connections()
                    .iter()
                    .filter_map(|id| self.graph.connection(*id))
                    .map(|c| c.used_channels())
                    .max()
                    .unwrap_or(0);
                state.channels[side as usize] = i32::from(cable.kind.visual_channels(
                    max as i32,
                    grid_node.max_channels(ChannelMode::Default) as i32,
                    ChannelMode::Default,
                    true,
                ));
            }
        }
        Some(state)
    }
    pub fn node_service(&self, key: impl Into<NodeKey>) -> Option<(ServiceId, NodeId)> {
        let node = self.nodes.get(&key.into())?;
        Some((node.service?, node.id))
    }
    /// The original connection direction is absent for an internal host edge.
    /// Return finalized channel use independently from the grid's power state.
    pub fn connection(
        &self,
        from: impl Into<NodeKey>,
        to: impl Into<NodeKey>,
    ) -> Option<(Option<Direction>, u32)> {
        let from = self.nodes.get(&from.into())?.id;
        let to = self.nodes.get(&to.into())?.id;
        let connection = self
            .graph
            .node(from)?
            .connections()
            .iter()
            .filter_map(|id| self.graph.connection(*id))
            .find(|connection| connection.other(from) == Some(to))?;
        Some((connection.direction(from), connection.used_channels()))
    }
    pub fn set_idle(&mut self, key: impl Into<NodeKey>, idle: f64) -> Result<(), &'static str> {
        let (service, node) = self.node_service(key).ok_or("unknown network node")?;
        self.energy
            .service_mut(service)
            .unwrap()
            .set_idle(node, idle)
    }
    pub fn is_online(&self, key: impl Into<NodeKey>) -> bool {
        let key = key.into();
        self.is_powered(key)
            && self
                .nodes
                .get(&key)
                .and_then(|n| self.graph.node(n.id))
                .is_some_and(|n| n.meets_channel_requirements())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ticks(n: &mut Network) {
        for _ in 0..20 {
            n.begin_tick();
            n.end_tick();
        }
    }
    #[test]
    fn all_six_primary_part_nodes_are_distinct_and_use_source_internal_edges() {
        use super::super::ae2_cable::CableType;
        let p = [0, 0, 0];
        let center = cable(p, CableType::Smart, Color::Red);
        let mut nodes = vec![center, snapshot([0, 0, 1], Kind::CreativeEnergyCell, 0.0)];
        // Leave South free to admit the power supply.
        nodes.extend(
            Direction::ALL
                .into_iter()
                .filter(|d| *d != Direction::South)
                .map(|d| Snapshot::terminal(p, d, 0.0)),
        );
        let mut n = Network::default();
        n.sync(nodes);
        ticks(&mut n);
        let center_id = n.node_service(p).unwrap().1;
        let mut ids = BTreeSet::from([center_id]);
        for side in Direction::ALL
            .into_iter()
            .filter(|d| *d != Direction::South)
        {
            let key = NodeKey::part(p, side);
            assert!(ids.insert(n.node_service(key).unwrap().1));
            assert!(n.is_online(key));
            assert_eq!(n.connection(p, key), Some((None, 5)));
            assert_eq!(n.connection(key, p), Some((None, 5)));
            assert_eq!(n.cable_visual(p).unwrap().channels[side as usize], 5);
        }
        assert_eq!(
            n.cable_visual(p).unwrap().connections,
            1 << Direction::South as u8
        );
        // PathingService's ad-hoc finalizer stamps the total demand (five)
        // onto every connection. Controller routing uses one on each leaf.
        let mut controlled = vec![
            center,
            snapshot([0, 0, 1], Kind::Controller, 0.0),
            snapshot([0, 0, 2], Kind::CreativeEnergyCell, 0.0),
        ];
        controlled.extend(
            Direction::ALL
                .into_iter()
                .filter(|d| *d != Direction::South)
                .map(|d| Snapshot::terminal(p, d, 0.0)),
        );
        n.sync(controlled);
        ticks(&mut n);
        for side in Direction::ALL
            .into_iter()
            .filter(|d| *d != Direction::South)
        {
            assert_eq!(n.connection(p, NodeKey::part(p, side)), Some((None, 1)));
        }
        let persisted = n.end_tick();
        assert_eq!(persisted.iter().filter(|s| s.pos == p).count(), 6);
        assert_eq!(
            persisted
                .iter()
                .filter(|s| s.pos == p && s.side.is_some())
                .count(),
            5
        );
        // All six parts can coexist when power is provided on an adjacent host.
        let mut nodes = vec![center];
        nodes.extend(Direction::ALL.map(|d| Snapshot::terminal(p, d, 0.0)));
        n.sync(nodes);
        assert_eq!(n.nodes.len(), 7);
    }
    #[test]
    fn occupied_face_blocks_the_center_and_primary_parts_never_connect_outside() {
        use super::super::ae2_cable::CableType;
        let p = [0, 0, 0];
        let side = NodeKey::part(p, Direction::East);
        let mut terminal = Snapshot::terminal(p, Direction::East, 0.0);
        terminal.exposed = 63;
        let mut n = Network::default();
        n.sync(vec![
            cable(p, CableType::Smart, Color::Transparent),
            terminal,
            snapshot([1, 0, 0], Kind::CreativeEnergyCell, 0.0),
        ]);
        ticks(&mut n);
        assert_eq!(n.connection(p, [1, 0, 0]), None);
        assert_eq!(n.connection(side, [1, 0, 0]), None);
        assert_eq!(n.connection(p, side), Some((None, 1)));
        assert!(!n.is_powered(p));
        assert!(!n.is_online(side));
        assert_ne!(
            n.node_service(side).unwrap().0,
            n.node_service([1, 0, 0]).unwrap().0
        );
    }
    #[test]
    fn part_channel_demand_counts_each_node_and_empty_hosts_do_not_join_parts() {
        use super::super::ae2_cable::CableType;
        let mut nodes = vec![
            cable([0, 0, 0], CableType::Smart, Color::Transparent),
            cable([1, 0, 0], CableType::Smart, Color::Transparent),
            snapshot([-1, 0, 0], Kind::CreativeEnergyCell, 0.0),
        ];
        let mut parts = vec![];
        for (p, excluded) in [([0, 0, 0], Direction::West), ([1, 0, 0], Direction::West)] {
            for side in Direction::ALL
                .into_iter()
                .filter(|d| *d != excluded && *d != Direction::East)
            {
                let part = Snapshot::terminal(p, side, 0.0);
                parts.push(part.key());
                nodes.push(part);
            }
        }
        // Eight source REQUIRE_CHANNEL parts operate in an ad-hoc grid.
        let mut n = Network::default();
        n.sync(nodes.clone());
        ticks(&mut n);
        assert_eq!(parts.len(), 8);
        assert!(parts.iter().all(|k| n.is_online(*k)));
        nodes.push(Snapshot::terminal([1, 0, 0], Direction::East, 0.0));
        n.sync(nodes);
        ticks(&mut n);
        assert!(parts.iter().all(|k| !n.is_online(*k)));
        n.sync(vec![
            Snapshot::terminal([3, 0, 0], Direction::Up, 0.0),
            Snapshot::terminal([3, 0, 0], Direction::Down, 0.0),
        ]);
        assert_ne!(
            n.node_service(([3, 0, 0], Direction::Up)).unwrap().0,
            n.node_service(([3, 0, 0], Direction::Down)).unwrap().0
        );
    }
    #[test]
    fn replacing_one_slot_preserves_siblings_while_replacing_the_be_forgets_all() {
        use super::super::ae2_cable::CableType;
        let p = [0, 0, 0];
        let a = NodeKey::part(p, Direction::Up);
        let b = NodeKey::part(p, Direction::Down);
        let nodes = vec![
            cable(p, CableType::Smart, Color::Transparent),
            Snapshot::terminal(p, Direction::Up, 0.0),
            Snapshot::terminal(p, Direction::Down, 0.0),
        ];
        let mut n = Network::default();
        n.sync(nodes.clone());
        let id = |n: &Network, key| n.node_service(key).unwrap().1;
        let before = (id(&n, NodeKey::center(p)), id(&n, a), id(&n, b));
        n.forget_node(a);
        n.sync(nodes.clone());
        assert_eq!(id(&n, NodeKey::center(p)), before.0);
        assert_ne!(id(&n, a), before.1);
        assert_eq!(id(&n, b), before.2);
        n.forget_node(p);
        n.sync(nodes.clone());
        assert_ne!(id(&n, NodeKey::center(p)), before.0);
        assert_eq!(id(&n, b), before.2);
        let old = n
            .nodes
            .values()
            .map(|node| node.id)
            .collect::<BTreeSet<_>>();
        n.forget_host(p);
        assert!(n.nodes.is_empty());
        n.sync(nodes);
        assert!(n.nodes.values().all(|node| !old.contains(&node.id)));
    }
    #[test]
    fn source_channel_bytes_do_not_disappear_when_the_energy_service_is_unpowered() {
        use super::super::ae2_cable::CableType;
        let mut n = Network::default();
        n.sync(vec![
            cable([0, 0, 0], CableType::Smart, Color::Transparent),
            snapshot([0, 0, -1], Kind::Drive, 0.0),
        ]);
        assert!(!n.is_powered([0, 0, 0]));
        let view = n.cable_stream([0, 0, 0]).unwrap();
        assert_eq!(n.cable_visual([0, 0, 0]).unwrap().channels, [0; 6]);
        assert_eq!(view.connections, 1 << Direction::North as u8);
        assert_eq!(view.channels[Direction::North as usize], 1);
    }

    #[test]
    fn powered_render_channels_require_a_smart_cable_but_stream_channels_do_not() {
        use super::super::ae2_cable::CableType;
        for kind in CableType::VALID {
            let mut n = Network::default();
            n.sync(vec![
                cable([0, 0, 0], kind, Color::Transparent),
                snapshot([0, 0, -1], Kind::Drive, 0.0),
                snapshot([0, 0, 1], Kind::CreativeEnergyCell, 0.0),
            ]);
            ticks(&mut n);
            assert!(n.is_powered([0, 0, 0]));
            assert_eq!(
                n.cable_stream([0, 0, 0]).unwrap().channels[Direction::North as usize],
                1
            );
            assert_eq!(
                n.cable_visual([0, 0, 0]).unwrap().channels[Direction::North as usize],
                i32::from(kind.is_smart()),
                "{kind:?}"
            );
        }
    }

    fn cable(pos: Pos, kind: super::super::ae2_cable::CableType, color: Color) -> Snapshot {
        snapshot(pos, Kind::Cable(Cable { kind, color }), 0.0)
    }
    #[test]
    fn cable_recolour_preserves_node_identity_but_rebuilds_colour_compatible_edges() {
        use super::super::ae2_cable::CableType;
        let mut n = Network::default();
        let a = cable([0, 0, 0], CableType::Smart, Color::Red);
        let mut b = cable([1, 0, 0], CableType::Smart, Color::Blue);
        let supply = snapshot([-1, 0, 0], Kind::EnergyCell, 1000.0);
        n.sync(vec![supply, a, b]);
        let id = n.node_service(b.pos).unwrap().1;
        assert_ne!(
            n.node_service(a.pos).unwrap().0,
            n.node_service(b.pos).unwrap().0
        );
        b.kind = Kind::Cable(Cable {
            kind: CableType::Smart,
            color: Color::Transparent,
        });
        n.sync(vec![supply, a, b]);
        assert_eq!(n.node_service(b.pos).unwrap().1, id);
        assert_eq!(
            n.node_service(a.pos).unwrap().0,
            n.node_service(b.pos).unwrap().0
        );
        assert_eq!(n.extract(b.pos, 10.0, 1.0, true), 10.0);
        b.kind = Kind::Cable(Cable {
            kind: CableType::Smart,
            color: Color::Blue,
        });
        n.sync(vec![supply, a, b]);
        assert_eq!(n.node_service(b.pos).unwrap().1, id);
        assert_ne!(
            n.node_service(a.pos).unwrap().0,
            n.node_service(b.pos).unwrap().0
        );
        assert_eq!(n.extract(b.pos, 10.0, 1.0, true), 0.0);
    }
    #[test]
    fn occupied_or_blocked_cable_face_disconnects_without_replacing_node() {
        use super::super::ae2_cable::CableType;
        let mut n = Network::default();
        let a = cable([0, 0, 0], CableType::Glass, Color::Transparent);
        let mut b = cable([1, 0, 0], CableType::Covered, Color::Red);
        n.sync(vec![a, b]);
        let id = n.node_service(b.pos).unwrap().1;
        b.exposed &= !(1 << Direction::West as u8);
        n.sync(vec![a, b]);
        assert_eq!(n.node_service(b.pos).unwrap().1, id);
        assert_ne!(
            n.node_service(a.pos).unwrap().0,
            n.node_service(b.pos).unwrap().0
        );
        b.exposed = 63;
        n.sync(vec![a, b]);
        assert_eq!(
            n.node_service(a.pos).unwrap().0,
            n.node_service(b.pos).unwrap().0
        );
        // The world adapter cannot infer a center cable from the host block ID.
        assert_eq!(Kind::from_block("ae2:cable_bus"), None);
    }
    #[test]
    fn dense_trunk_supplies_twelve_devices_and_normal_cable_is_a_real_bottleneck() {
        use super::super::ae2_cable::CableType;
        let mut n = Network::default();
        let mut nodes = vec![
            snapshot([-1, 0, 0], Kind::CreativeEnergyCell, 0.0),
            snapshot([0, 0, 0], Kind::Controller, 0.0),
        ];
        for x in 1..=24 {
            nodes.push(cable([x, 0, 0], CableType::DenseSmart, Color::Transparent));
        }
        let drives: Vec<_> = (1..=12).map(|x| [x * 2, 1, 0]).collect();
        nodes.extend(drives.iter().map(|p| snapshot(*p, Kind::Drive, 0.0)));
        n.sync(nodes.clone());
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(drives.iter().all(|p| n.is_online(*p)));
        let dense = n.graph.node(n.node_service([1, 0, 0]).unwrap().1).unwrap();
        assert!(dense.spec.flags.has(Flags::DENSE_CAPACITY));
        assert!(!dense.spec.flags.has(Flags::PREFERRED));
        assert_eq!(dense.max_channels(ChannelMode::Default), 32);
        nodes.iter_mut().find(|s| s.pos == [1, 0, 0]).unwrap().kind = Kind::Cable(Cable {
            kind: CableType::Smart,
            color: Color::Transparent,
        });
        n.sync(nodes);
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(drives.iter().filter(|p| n.is_online(**p)).count() <= 8);
        let normal = n.graph.node(n.node_service([1, 0, 0]).unwrap().1).unwrap();
        assert!(normal.spec.flags.has(Flags::PREFERRED));
        assert_eq!(normal.max_channels(ChannelMode::Default), 8);
    }

    #[test]
    fn placement_after_same_tick_break_never_reuses_a_spent_node_with_identical_saved_nbt() {
        let pos = [1, 2, 3];
        let mut network = Network::default();
        network.sync(vec![snapshot(pos, Kind::EnergyCell, 10000.0)]);
        let old = network.node_service(pos).unwrap().1;
        assert_eq!(network.extract(pos, 1600.0, 1.0, true), 1600.0);
        assert!(
            network
                .storage_for_snapshot(pos, Kind::EnergyCell, 10000.0)
                .unwrap()
                < 10000.0
        );
        network.forget_node(pos);
        network.sync(vec![snapshot(pos, Kind::EnergyCell, 10000.0)]);
        assert_ne!(network.node_service(pos).unwrap().1, old);
        assert_eq!(
            network.storage_for_snapshot(pos, Kind::EnergyCell, 10000.0),
            Some(10000.0)
        );
    }

    #[test]
    fn live_storage_export_sees_same_tick_extraction_but_never_overwrites_new_external_data() {
        let mut network = Network::default();
        network.sync(vec![snapshot([1, 2, 3], Kind::EnergyCell, 10000.0)]);
        assert_eq!(
            network.storage_for_snapshot([1, 2, 3], Kind::EnergyCell, 10000.0),
            Some(10000.0)
        );
        assert_eq!(network.extract([1, 2, 3], 1600.0, 1.0, false), 1600.0);
        assert_eq!(
            network.storage_for_snapshot([1, 2, 3], Kind::EnergyCell, 10000.0),
            Some(10000.0)
        );
        assert_eq!(network.extract([1, 2, 3], 1600.0, 1.0, true), 1600.0);
        assert!(
            network
                .storage_for_snapshot([1, 2, 3], Kind::EnergyCell, 10000.0)
                .unwrap()
                < 10000.0
        );
        assert_eq!(
            network.storage_for_snapshot([1, 2, 3], Kind::EnergyCell, 12000.0),
            None
        );
        assert_eq!(
            network.storage_for_snapshot([1, 2, 3], Kind::DenseEnergyCell, 10000.0),
            None
        );
        network.sync(vec![]);
        assert_eq!(
            network.storage_for_snapshot([1, 2, 3], Kind::EnergyCell, 10000.0),
            None
        );
    }
    #[test]
    fn finite_cell_initial_load_uses_set_stored_threshold_without_touching_controller() {
        for kind in [Kind::EnergyCell, Kind::DenseEnergyCell] {
            let small = kind.energy_node(super::super::ae2_energy::MIN_AMOUNT / 2.0);
            assert_eq!(small.store.unwrap().amount(), 0.0);
            let exact = kind.energy_node(super::super::ae2_energy::MIN_AMOUNT);
            assert_eq!(
                exact.store.unwrap().amount(),
                super::super::ae2_energy::MIN_AMOUNT
            );
        }
        assert_eq!(
            Kind::Controller
                .energy_node(0.0000005)
                .store
                .unwrap()
                .amount(),
            0.0000005
        );
    }
    fn snapshot(pos: Pos, kind: Kind, stored: f64) -> Snapshot {
        Snapshot {
            pos,
            side: None,
            kind,
            front: Direction::North,
            exposed: 63,
            stored,
            saved_buffer: 0.0,
        }
    }
    #[test]
    fn controller_has_source_public_storage_priority_and_three_ae_idle() {
        let mut n = Network::default();
        n.sync(vec![snapshot([0, 0, 0], Kind::Controller, 1000.0)]);
        let (service, id) = n.node_service([0, 0, 0]).unwrap();
        assert_eq!(n.graph.node(id).unwrap().spec.controller, Some([0, 0, 0]));
        assert!(
            n.graph
                .node(id)
                .unwrap()
                .spec
                .flags
                .has(Flags::CANNOT_CARRY | Flags::DENSE_CAPACITY)
        );
        assert_eq!(Kind::Controller.capacity(), 8000.0);
        assert_eq!(n.extract([0, 0, 0], 100.0, 1.0, false), 100.0);
        assert_eq!(
            n.energy
                .service(service)
                .unwrap()
                .store(id)
                .unwrap()
                .amount(),
            1000.0
        );
        assert_eq!(
            n.energy
                .service(service)
                .unwrap()
                .store(id)
                .unwrap()
                .priority,
            0
        );
        let mut saved = Vec::new();
        for _ in 0..40 {
            n.begin_tick();
            saved = n.end_tick();
        }
        let remaining: f64 = saved
            .iter()
            .map(|s| s.stored.unwrap_or(0.0) + s.buffer_share)
            .sum();
        assert!((remaining - (1000.0 - 40.0 * 3.0)).abs() < 1e-9);
        assert_eq!(saved[0].controller_model, Some("online"));
    }
    #[test]
    fn controller_multiblock_supplies_channels_beyond_the_ad_hoc_limit() {
        let mut n = Network::default();
        let mut nodes = vec![snapshot([-1, 0, 0], Kind::CreativeEnergyCell, 0.0)];
        let mut drives = Vec::new();
        for x in 0..3 {
            nodes.push(snapshot([x, 0, 0], Kind::Controller, 0.0));
            for (y, z, front) in [
                (1, 0, Direction::North),
                (-1, 0, Direction::North),
                (0, -1, Direction::North),
                (0, 1, Direction::South),
            ] {
                let pos = [x, y, z];
                let mut drive = snapshot(pos, Kind::Drive, 0.0);
                drive.front = front;
                nodes.push(drive);
                drives.push(pos);
            }
        }
        n.sync(nodes.clone());
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(
            drives.iter().all(|p| n.is_online(*p)),
            "twelve devices get real controller paths"
        );
        for node in &mut nodes {
            if node.kind == Kind::Controller {
                node.kind = Kind::Charger;
            }
        }
        n.sync(nodes);
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(
            drives.iter().all(|p| !n.is_online(*p)),
            "same connected devices exceed ad-hoc eight"
        );
    }
    #[test]
    fn powered_controller_conflicts_follow_source_seven_block_bound_and_direct_connectivity() {
        let mut n = Network::default();
        let mut nodes = vec![snapshot([-1, 0, 0], Kind::CreativeEnergyCell, 0.0)];
        nodes.extend((0..8).map(|x| snapshot([x, 0, 0], Kind::Controller, 0.0)));
        n.sync(nodes.clone());
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(
            n.end_tick()
                .iter()
                .filter(|s| s.kind == Kind::Controller)
                .all(|s| s.controller_model == Some("conflicted"))
        );
        nodes.pop();
        n.sync(nodes);
        n.begin_tick();
        assert!(
            n.end_tick()
                .iter()
                .filter(|s| s.kind == Kind::Controller)
                .all(|s| s.controller_model == Some("online"))
        );
        n.sync(vec![
            snapshot([0, 0, 0], Kind::Controller, 0.0),
            snapshot([1, 0, 0], Kind::CreativeEnergyCell, 0.0),
            snapshot([2, 0, 0], Kind::Controller, 0.0),
        ]);
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(
            n.end_tick()
                .iter()
                .filter(|s| s.kind == Kind::Controller)
                .all(|s| s.controller_model == Some("conflicted"))
        );
    }
    #[test]
    fn finite_cell_drives_charger_only_through_an_exposed_face() {
        let mut n = Network::default();
        let charger = snapshot([0, 0, 0], Kind::Charger, 0.0);
        let back = snapshot([0, 0, 1], Kind::EnergyCell, 2000.0);
        n.sync(vec![charger, back]);
        n.begin_tick();
        assert_eq!(n.extract([0, 0, 0], 800.0, 1.0, true), 800.0);
        let saved = n.end_tick();
        assert_eq!(
            saved
                .iter()
                .find(|s| s.kind == Kind::EnergyCell)
                .unwrap()
                .stored,
            Some(1200.0)
        );
        let mut front = back;
        front.pos = [0, 0, -1];
        n.sync(vec![charger, front]);
        n.begin_tick();
        assert_eq!(n.extract([0, 0, 0], 800.0, 1.0, true), 0.0);
    }
    #[test]
    fn merge_and_split_do_not_duplicate_or_erase_buffer_energy() {
        let mut n = Network::default();
        let mut a = snapshot([0, 0, 0], Kind::Charger, 0.0);
        a.saved_buffer = 10.0;
        let mut b = snapshot([2, 0, 0], Kind::Charger, 0.0);
        b.saved_buffer = 20.0;
        n.sync(vec![a, b]);
        n.begin_tick();
        let bridge = snapshot([1, 0, 0], Kind::Charger, 0.0);
        n.sync(vec![a, b, bridge]);
        let saved = n.end_tick();
        assert!((saved.iter().map(|s| s.buffer_share).sum::<f64>() - 30.0).abs() < 1e-9);
        // Removing a real node deducts its 1/N share, like GridEnergyStorage.
        a.saved_buffer = 10.0;
        b.saved_buffer = 10.0;
        n.sync(vec![a, b]);
        let saved = n.end_tick();
        assert!((saved.iter().map(|s| s.buffer_share).sum::<f64>() - 20.0).abs() < 1e-9);
    }
    #[test]
    fn unloaded_reload_restores_only_its_saved_share() {
        let mut n = Network::default();
        let mut a = snapshot([0, 0, 0], Kind::Charger, 0.0);
        a.saved_buffer = 17.0;
        n.sync(vec![a]);
        let saved = n.end_tick();
        n.sync(vec![]);
        a.saved_buffer = saved[0].buffer_share;
        n.sync(vec![a]);
        assert_eq!(n.extract(a.pos, 100.0, 1.0, true), 17.0);
    }
    #[test]
    fn external_nbt_energy_change_is_applied_once_and_stable_ticks_do_not_reset_it() {
        let mut n = Network::default();
        let a = snapshot([0, 0, 0], Kind::EnergyCell, 1000.0);
        n.sync(vec![a]);
        n.extract(a.pos, 100.0, 1.0, true);
        n.sync(vec![a]); // post-tick: NBT still has the old value until persist
        let saved = n.end_tick();
        let mut a = a;
        a.stored = saved[0].stored.unwrap();
        n.sync(vec![a]);
        assert_eq!(n.extract(a.pos, 1000.0, 1.0, true), 900.0);
        a.stored = 777.0;
        n.sync(vec![a]);
        assert_eq!(n.extract(a.pos, 1000.0, 1.0, true), 777.0);
    }
    #[test]
    fn replacing_a_block_destroys_old_energy_node_and_uses_new_kind() {
        let mut n = Network::default();
        let a = snapshot([0, 0, 0], Kind::CreativeEnergyCell, 0.0);
        n.sync(vec![a]);
        assert_eq!(n.extract(a.pos, 1e6, 1.0, true), 1e6);
        n.sync(vec![snapshot(a.pos, Kind::EnergyCell, 10.0)]);
        assert_eq!(n.extract(a.pos, 100.0, 1.0, true), 10.0);
    }
    #[test]
    fn drives_consume_real_idle_energy_and_require_channels() {
        let mut n = Network::default();
        let a = snapshot([0, 0, 0], Kind::Drive, 0.0);
        let source = snapshot([0, 0, 1], Kind::EnergyCell, 1000.0);
        n.sync(vec![a, source]);
        n.set_idle(a.pos, 2.5).unwrap();
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(n.is_online(a.pos));
        // One ad-hoc channel across two nodes costs another 2/128 AE/tick.
        assert_eq!(
            n.extract(a.pos, 1000.0, 1.0, false),
            1000.0 - 40.0 * (2.5 + 2.0 / 128.0)
        );
        let mut drives = vec![snapshot([0, 0, 1], Kind::CreativeEnergyCell, 0.0)];
        for x in 0..9 {
            drives.push(snapshot([x, 0, 0], Kind::Drive, 0.0));
        }
        n.sync(drives.clone());
        for _ in 0..40 {
            n.begin_tick();
            n.end_tick();
        }
        assert!(n.is_powered(a.pos));
        assert!(!n.is_online(a.pos));
        drives.pop();
        n.sync(drives);
        n.begin_tick();
        n.end_tick();
        assert!(n.is_online(a.pos));
    }
}
