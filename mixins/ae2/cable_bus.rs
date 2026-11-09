// SPDX-License-Identifier: LGPL-3.0-or-later
// CableBusContainer's persisted center/side slots and GridNode's gn data.
// Registry/facade codecs and unsupported side-part implementations remain open.
use super::ae2_cable::Cable;
use super::ae2_grid::Direction;
use super::ae2_network::{Kind, Network, NodeKey, Pos, Snapshot};
use super::ae2_part_storage::PartStorage;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::collections::BTreeMap;

pub const BLOCK: &str = "ae2:cable_bus";
pub const SIDES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
pub const FACADES: [&str; 6] = [
    "facadeDown",
    "facadeUp",
    "facadeNorth",
    "facadeSouth",
    "facadeWest",
    "facadeEast",
];

#[derive(Clone, Debug)]
pub struct Part {
    pub id: String,
    pub data: Compound,
}
impl Part {
    pub fn reporting(&self) -> Option<super::ae2_reporting::Reporting> {
        use super::ae2_reporting::{Number, Reporting};
        if self.id != "ae2:terminal" {
            return None;
        }
        let number = match self.data.get(&key("spin")) {
            Some(Tag::Byte(v)) => Some(Number::Byte(*v)),
            Some(Tag::Short(v)) => Some(Number::Short(*v)),
            Some(Tag::Int(v)) => Some(Number::Int(*v)),
            Some(Tag::Long(v)) => Some(Number::Long(*v)),
            Some(Tag::Float(v)) => Some(Number::Float(f32::from_bits(*v))),
            Some(Tag::Double(v)) => Some(Number::Double(f64::from_bits(*v))),
            _ => None,
        };
        let mut reporting = Reporting::default();
        reporting.read_spin(number);
        Some(reporting)
    }
    /// Only the source spin byte is saved here. Opacity and synchronized
    /// power/channel flags are transient; preserve all other raw part fields.
    pub fn save_reporting(&mut self, reporting: &super::ae2_reporting::Reporting) -> bool {
        if self.id != "ae2:terminal" {
            return false;
        }
        let value = Tag::Byte(reporting.spin());
        let changed = self.data.get(&key("spin")) != Some(&value);
        self.data.insert(key("spin"), value);
        changed
    }
}
#[derive(Clone, Debug)]
pub struct Saved {
    pub parts: PartStorage<Part, Part, Tag>,
    pub redstone: i32,
}
fn key(value: &str) -> JavaString {
    JavaString::from(value)
}
fn part(value: Option<&Tag>) -> Result<Option<Part>, &'static str> {
    let Some(Tag::Compound(data)) = value else {
        return Ok(None);
    };
    let Some(Tag::String(id)) = data.get(&key("id")) else {
        return Err("part id missing or not a string");
    };
    let id = id.to_string().map_err(|_| "part id is not Unicode")?;
    // Full ResourceLocation and IPartItem registry recovery is not yet ported.
    if id.is_empty() {
        return Err("empty persisted part id");
    }
    Ok(Some(Part {
        id,
        data: data.clone(),
    }))
}
fn node(part: &Part) -> Option<&Compound> {
    match part.data.get(&key("gn")) {
        Some(Tag::Compound(node)) => Some(node),
        _ => None,
    }
}
fn buffer(part: &Part) -> Result<f64, &'static str> {
    let value = match node(part).and_then(|n| n.get(&key("e"))) {
        Some(Tag::Double(value)) => f64::from_bits(*value),
        _ => 0.0,
    };
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err("unsupported node energy")
    }
}
/// GridNode.areTagsEqualIgnoringPlayerId treats absent, empty and p-only alike.
fn same_node_data(a: Option<&Compound>, b: Option<&Compound>) -> bool {
    super::ae2_node_data::same_ignoring_owner(a, b, &key("p"))
}
impl Saved {
    pub fn read(host: &Compound) -> Result<Self, &'static str> {
        let mut parts = PartStorage::default();
        parts.set_center(part(host.get(&key("cable")))?);
        for (i, side) in Direction::ALL.into_iter().enumerate() {
            if let Some(value) = part(host.get(&key(SIDES[i])))? {
                parts.set_part(side, Some(value));
            }
            // Keep raw facade BlockState codec payloads, never manufacture a
            // facade from a block name. Actual codec/shape support is pending.
            if let Some(tag) = host.get(&key(FACADES[i])) {
                parts.set_facade(side, Some(tag.clone()));
            }
        }
        let redstone = match host.get(&key("hasRedstone")) {
            None => 2,
            Some(Tag::Int(v)) => *v,
            _ => return Err("unsupported YesNo tag type"),
        };
        if !(0..=2).contains(&redstone) {
            return Err("invalid YesNo ordinal");
        }
        Ok(Self { parts, redstone })
    }
    pub fn center(&self) -> Result<Option<Cable>, &'static str> {
        self.parts
            .center()
            .map(|p| Cable::from_item(&p.id).ok_or("unsupported center part"))
            .transpose()
    }
    pub fn part(&self, side: Option<Direction>) -> Option<&Part> {
        side.map_or_else(|| self.parts.center(), |side| self.parts.part(side))
    }
    /// Primary-node decoding for the next multipart host stage. The active
    /// native adapter remains center-only until part shape/input/render support
    /// is bound; do not equate these snapshots with a functioning terminal UI.
    pub fn primary_snapshots(&self, pos: Pos, blocked: u8) -> Result<Vec<Snapshot>, &'static str> {
        let cable = self.center()?;
        let mut output = Vec::new();
        let mut exposed = 63 & !blocked;
        let mut sides = Vec::new();
        for side in Direction::ALL {
            if let Some(part) = self.parts.part(side) {
                if part.id != "ae2:terminal" {
                    return Err("side-part implementation unavailable");
                }
                // IPart.canBePlacedOn allows CABLE; dense supports DENSE_CABLE.
                if cable.is_some_and(|c| c.kind.is_dense()) {
                    return Err("terminal is incompatible with dense cable");
                }
                exposed &= !(1 << side as u8);
                sides.push(Snapshot::terminal(pos, side, buffer(part)?));
            }
        }
        if let Some(cable) = cable {
            output.push(Snapshot {
                pos,
                side: None,
                kind: Kind::Cable(cable),
                front: Direction::North,
                exposed,
                stored: 0.0,
                saved_buffer: buffer(self.parts.center().unwrap())?,
            });
        }
        // Source addToWorld installs the center, then East through Down.
        output.extend(sides.into_iter().rev());
        Ok(output)
    }
    pub fn center_snapshot(&self, pos: Pos) -> Result<Option<Snapshot>, &'static str> {
        if Direction::ALL
            .into_iter()
            .any(|side| self.parts.part(side).is_some())
        {
            return Err("side-part implementation unavailable");
        }
        Ok(self.primary_snapshots(pos, 0)?.into_iter().next())
    }
    /// Source local boxes for supported primary slots. Facades need their own
    /// codec and context-sensitive shape pass; never discard them here.
    pub fn primary_boxes(
        &self,
        connections: u8,
        dense_neighbours: u8,
        visual: bool,
    ) -> Result<Vec<(Option<Direction>, super::ae2_part_geometry::Bounds)>, &'static str> {
        self.primary_snapshots([0; 3], 0)?;
        if Direction::ALL
            .into_iter()
            .any(|d| self.parts.facade(d).is_some())
        {
            return Err("facade shape implementation unavailable");
        }
        let parts = Direction::ALL.into_iter().fold(0, |mask, side| {
            mask | if self.parts.part(side).is_some() {
                1 << side as u8
            } else {
                0
            }
        });
        Ok(super::ae2_part_geometry::reporting_host_boxes(
            self.center()?
                .map_or(super::ae2_cable::CableType::None, |c| c.kind),
            parts,
            connections,
            dense_neighbours,
            visual,
        ))
    }
}
#[derive(Clone, Debug)]
struct Observed {
    part: Part,
}
#[derive(Default)]
pub struct Hosts {
    observed: BTreeMap<NodeKey, Observed>,
}
impl Hosts {
    /// Source loadPart replaces the part when item identity differs, including
    /// externally edited color IDs. Actual painting will update the existing
    /// node separately; it is not the NBT loadPart path.
    pub fn observe(
        &mut self,
        pos: Pos,
        host: &Compound,
    ) -> Result<(Option<Snapshot>, bool), &'static str> {
        // Keep the active native boundary until multipart geometry/input/menu
        // work is installed. An unknown side must never silently disappear.
        let saved = Saved::read(host)?;
        if Direction::ALL
            .into_iter()
            .any(|side| saved.parts.part(side).is_some())
        {
            return Err("side-part implementation unavailable");
        }
        let (snapshots, replaced) = self.observe_saved(pos, &saved, 0)?;
        Ok((
            snapshots.into_iter().next(),
            replaced.contains(&NodeKey::center(pos)),
        ))
    }
    pub fn observe_primary(
        &mut self,
        pos: Pos,
        host: &Compound,
        blocked: u8,
    ) -> Result<(Vec<Snapshot>, Vec<NodeKey>), &'static str> {
        let saved = Saved::read(host)?;
        self.observe_saved(pos, &saved, blocked)
    }
    fn observe_saved(
        &mut self,
        pos: Pos,
        saved: &Saved,
        blocked: u8,
    ) -> Result<(Vec<Snapshot>, Vec<NodeKey>), &'static str> {
        let snapshots = saved.primary_snapshots(pos, blocked)?;
        let incoming = snapshots
            .iter()
            .map(|snapshot| {
                let k = snapshot.key();
                (k, saved.part(k.side).unwrap().clone())
            })
            .collect::<BTreeMap<_, _>>();
        // Keys are ordered center, East ... Down at each position. Observe
        // only this host's at most seven primary nodes, not every loaded host.
        let old_keys = self
            .observed
            .range(NodeKey::center(pos)..=NodeKey::part(pos, Direction::Down))
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        let replaced = old_keys
            .iter()
            .copied()
            .filter(|key| {
                let old = &self.observed[key];
                incoming.get(key).is_none_or(|new| {
                    new.id != old.part.id || !same_node_data(node(&old.part), node(new))
                })
            })
            .collect();
        for key in old_keys {
            self.observed.remove(&key);
        }
        self.observed.extend(
            incoming
                .into_iter()
                .map(|(key, part)| (key, Observed { part })),
        );
        Ok((snapshots, replaced))
    }
    pub fn primary_snapshots(
        &mut self,
        pos: Pos,
        host: &Compound,
        blocked: u8,
        network: &mut Network,
    ) -> Result<Vec<Snapshot>, &'static str> {
        let (snapshots, replaced) = self.observe_primary(pos, host, blocked)?;
        for key in replaced {
            network.forget_node(key);
        }
        Ok(snapshots)
    }
    pub fn snapshot(
        &mut self,
        pos: Pos,
        host: &Compound,
        network: &mut Network,
    ) -> Result<Option<Snapshot>, &'static str> {
        let (snapshot, replaced) = self.observe(pos, host)?;
        if replaced {
            network.forget_node(pos);
        }
        Ok(snapshot)
    }
    pub fn retain(&mut self, positions: &std::collections::BTreeSet<Pos>) {
        self.observed.retain(|key, _| positions.contains(&key.pos));
    }
    /// Return a new gn only if the live center still equals the observed one.
    /// Preserve host/part/facade fields and unknown service data; this is not a
    /// complete Grid.saveNodeData implementation. p is always an Int on save.
    pub fn persist(&mut self, pos: Pos, host: &mut Compound, cable: Cable, share: f64) -> bool {
        self.persist_node(NodeKey::center(pos), host, Kind::Cable(cable), share)
    }
    pub fn persist_node(
        &mut self,
        address: NodeKey,
        host: &mut Compound,
        expected_kind: Kind,
        share: f64,
    ) -> bool {
        let field = address.side.map_or("cable", |side| SIDES[side as usize]);
        if !share.is_finite() || share < 0.0 {
            return false;
        }
        let Some(old) = self.observed.get_mut(&address) else {
            return false;
        };
        let Ok(Some(current)) = part(host.get(&key(field))) else {
            return false;
        };
        let kind = match address.side {
            None => Cable::from_item(&current.id).map(Kind::Cable),
            Some(_) if current.id == "ae2:terminal" => Some(Kind::Terminal),
            _ => None,
        };
        if kind != Some(expected_kind) || current.id != old.part.id || current.data != old.part.data
        {
            return false;
        }
        let mut data = current.data;
        let mut gn = node(&old.part).cloned().unwrap_or_default();
        let owner = match gn.get(&key("p")) {
            Some(Tag::Int(v)) => *v,
            _ => -1,
        };
        gn.insert(key("p"), Tag::Int(owner));
        if share > 0.0 {
            gn.insert(key("e"), Tag::Double(share.to_bits()));
        } else {
            gn.remove(&key("e"));
        }
        data.insert(key("gn"), Tag::Compound(gn));
        let changed = data != old.part.data;
        old.part.data = data.clone();
        if changed {
            host.insert(key(field), Tag::Compound(data));
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::super::ae2_cable::{CableType, Color};
    use super::*;
    fn host(color: Color, energy: f64) -> Compound {
        let cable = Cable {
            kind: CableType::Smart,
            color,
        };
        Compound::from([(
            key("cable"),
            Tag::compound([
                ("id", Tag::String(cable.item().unwrap().as_str().into())),
                (
                    "gn",
                    Tag::compound([("p", Tag::Int(12)), ("e", Tag::Double(energy.to_bits()))]),
                ),
            ]),
        )])
    }
    fn center(host: &mut Compound) -> &mut Compound {
        let Tag::Compound(v) = host.get_mut(&key("cable")).unwrap() else {
            unreachable!()
        };
        v
    }
    fn terminal(owner: i32, energy: f64) -> Tag {
        Tag::compound([
            ("id", Tag::String("ae2:terminal".into())),
            ("spin", Tag::Byte(2)),
            (
                "gn",
                Tag::compound([("p", Tag::Int(owner)), ("e", Tag::Double(energy.to_bits()))]),
            ),
            ("otherService", Tag::Long(91)),
        ])
    }

    #[test]
    fn supported_primary_shapes_keep_sides_and_refuse_unimplemented_facades() {
        let mut data = host(Color::Transparent, 0.0);
        for side in SIDES {
            data.insert(key(side), terminal(1, 0.0));
        }
        let saved = Saved::read(&data).unwrap();
        let boxes = saved.primary_boxes(0, 0, true).unwrap();
        // Six reporting parts (two boxes each), cable core and six internal
        // cable connectors ending at AEBasePart's source length of 3 pixels.
        assert_eq!(boxes.len(), 19);
        assert_eq!(boxes[0].0, Some(Direction::Down));
        assert_eq!(boxes[12].0, None);
        assert_eq!(
            boxes[12].1.0,
            [
                5. / 16.,
                5. / 16.,
                5. / 16.,
                11. / 16.,
                11. / 16.,
                11. / 16.
            ]
        );
        assert_eq!(boxes[13].1.0[1], 3. / 16.);
        data.insert(key(FACADES[0]), Tag::Int(7));
        assert!(
            Saved::read(&data)
                .unwrap()
                .primary_boxes(0, 0, true)
                .is_err()
        );
        assert!(Saved::read(&data).unwrap().center_snapshot([0; 3]).is_err());
    }

    #[test]
    fn reporting_spin_codec_keeps_signed_source_values_and_only_saves_spin() {
        let mut data = host(Color::Transparent, 0.0);
        let Tag::Compound(mut part) = terminal(9, 3.0) else {
            unreachable!()
        };
        part.insert(key("spin"), Tag::Float((-0.1f32).to_bits()));
        part.insert(key("unknown"), Tag::Int(123));
        data.insert(key("north"), Tag::Compound(part));
        let mut saved = Saved::read(&data).unwrap();
        let mut part = saved.parts.part(Direction::North).unwrap().clone();
        let mut state = part.reporting().unwrap();
        assert_eq!(state.spin(), -1);
        let original_gn = part.data.get(&key("gn")).cloned();
        state.wrench_turn(false);
        assert!(part.save_reporting(&state));
        assert_eq!(part.data.get(&key("spin")), Some(&Tag::Byte(0)));
        assert_eq!(part.data.get(&key("gn")), original_gn.as_ref());
        assert_eq!(part.data.get(&key("unknown")), Some(&Tag::Int(123)));
        assert!(!part.data.contains_key(&key("opacity")));
        saved.parts.set_part(Direction::North, Some(part));
        assert_eq!(saved.parts.center().unwrap().id, "ae2:fluix_smart_cable");
    }
    #[test]
    fn primary_node_decoding_keeps_each_side_and_removes_occupied_or_blocked_exposure() {
        let pos = [1, 2, 3];
        for mask in 0u8..64 {
            let mut h = host(Color::Red, 20.0);
            for side in Direction::ALL {
                if mask & (1 << side as u8) != 0 {
                    h.insert(
                        key(SIDES[side as usize]),
                        terminal(side as i32, side as u8 as f64 + 1.0),
                    );
                }
            }
            let saved = Saved::read(&h).unwrap();
            for blocked in 0u8..64 {
                let snapshots = saved.primary_snapshots(pos, blocked).unwrap();
                assert_eq!(snapshots.len(), 1 + mask.count_ones() as usize);
                assert_eq!(snapshots[0].key(), NodeKey::center(pos));
                assert_eq!(snapshots[0].exposed, 63 & !mask & !blocked);
                let mut seen = std::collections::BTreeSet::new();
                for snapshot in &snapshots[1..] {
                    let side = snapshot.side.unwrap();
                    assert_eq!(snapshot.kind, Kind::Terminal);
                    assert_eq!(snapshot.exposed, 0);
                    assert_eq!(snapshot.saved_buffer, side as u8 as f64 + 1.0);
                    assert!(seen.insert(snapshot.key()));
                }
            }
            assert_eq!(saved.center_snapshot(pos).is_err(), mask != 0);
        }
    }
    #[test]
    fn side_only_hosts_stay_separate_and_unknown_or_dense_parts_refuse_without_mutation() {
        let pos = [1, 2, 3];
        let mut h = Compound::new();
        h.insert(key("north"), terminal(1, 4.0));
        h.insert(key("south"), terminal(2, 6.0));
        let mut hosts = Hosts::default();
        let mut network = Network::default();
        let snapshots = hosts.primary_snapshots(pos, &h, 0, &mut network).unwrap();
        assert_eq!(snapshots.len(), 2);
        network.sync(snapshots);
        assert_ne!(
            network.node_service((pos, Direction::North)).unwrap().0,
            network.node_service((pos, Direction::South)).unwrap().0
        );
        let before = h.clone();
        assert!(Saved::read(&h).unwrap().center_snapshot(pos).is_err());
        assert_eq!(h, before);
        h.insert(
            key("west"),
            Tag::compound([("id", Tag::String("addon:unknown_part".into()))]),
        );
        assert!(Saved::read(&h).unwrap().primary_snapshots(pos, 0).is_err());
        let mut h = host(Color::Red, 0.0);
        h.insert(key("north"), terminal(1, 0.0));
        center(&mut h).insert(
            key("id"),
            Tag::String(
                Cable {
                    kind: CableType::DenseSmart,
                    color: Color::Red,
                }
                .item()
                .unwrap()
                .as_str()
                .into(),
            ),
        );
        assert!(Saved::read(&h).unwrap().primary_snapshots(pos, 0).is_err());
    }
    #[test]
    fn node_data_replacement_for_one_side_does_not_reset_center_or_siblings() {
        let pos = [1, 2, 3];
        let mut h = host(Color::Red, 20.0);
        h.insert(key("north"), terminal(1, 4.0));
        h.insert(key("south"), terminal(2, 6.0));
        let mut hosts = Hosts::default();
        let mut network = Network::default();
        let snapshots = hosts.primary_snapshots(pos, &h, 0, &mut network).unwrap();
        network.sync(snapshots);
        let keys = [
            NodeKey::center(pos),
            NodeKey::part(pos, Direction::North),
            NodeKey::part(pos, Direction::South),
        ];
        let ids = keys.map(|key| network.node_service(key).unwrap().1);
        let Tag::Compound(north) = h.get_mut(&key("north")).unwrap() else {
            unreachable!()
        };
        let Tag::Compound(gn) = north.get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("p"), Tag::Int(99));
        let snapshots = hosts.primary_snapshots(pos, &h, 0, &mut network).unwrap();
        network.sync(snapshots);
        assert_eq!(keys.map(|key| network.node_service(key).unwrap().1), ids);
        let Tag::Compound(north) = h.get_mut(&key("north")).unwrap() else {
            unreachable!()
        };
        let Tag::Compound(gn) = north.get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("e"), Tag::Double(3.0f64.to_bits()));
        let snapshots = hosts.primary_snapshots(pos, &h, 0, &mut network).unwrap();
        network.sync(snapshots);
        let after = keys.map(|key| network.node_service(key).unwrap().1);
        assert_eq!(ids[0], after[0]);
        assert_ne!(ids[1], after[1]);
        assert_eq!(ids[2], after[2]);
        h.remove(&key("cable"));
        let snapshots = hosts.primary_snapshots(pos, &h, 0, &mut network).unwrap();
        network.sync(snapshots);
        assert_eq!(network.node_service(keys[0]), None);
        assert_eq!(network.node_service(keys[2]).unwrap().1, ids[2]);
    }
    #[test]
    fn side_energy_save_targets_only_its_still_observed_gn_and_preserves_raw_data() {
        let pos = [1, 2, 3];
        let mut h = host(Color::Red, 20.0);
        h.insert(key("north"), terminal(1, 4.0));
        h.insert(key("south"), terminal(2, 6.0));
        h.insert(
            key("proxy"),
            Tag::compound([("e", Tag::Double(999.0f64.to_bits()))]),
        );
        h.insert(key("facadeEast"), Tag::String("opaque-codec-marker".into()));
        let before = h.clone();
        let mut hosts = Hosts::default();
        hosts.observe_primary(pos, &h, 0).unwrap();
        let k = NodeKey::part(pos, Direction::North);
        assert!(hosts.persist_node(k, &mut h, Kind::Terminal, 1.5));
        for field in ["cable", "south", "proxy", "facadeEast"] {
            assert_eq!(h.get(&key(field)), before.get(&key(field)));
        }
        let saved = Saved::read(&h).unwrap();
        let north = saved.part(Some(Direction::North)).unwrap();
        assert_eq!(buffer(north).unwrap(), 1.5);
        assert_eq!(north.data.get(&key("spin")), Some(&Tag::Byte(2)));
        assert_eq!(north.data.get(&key("otherService")), Some(&Tag::Long(91)));
        assert!(!hosts.persist_node(k, &mut h, Kind::Terminal, 1.5));
        let Tag::Compound(north) = h.get_mut(&key("north")).unwrap() else {
            unreachable!()
        };
        north.insert(key("spin"), Tag::Byte(1));
        let newer = h.clone();
        assert!(!hosts.persist_node(k, &mut h, Kind::Terminal, 0.0));
        assert_eq!(h, newer);
        hosts.observe_primary(pos, &h, 0).unwrap();
        assert!(hosts.persist_node(k, &mut h, Kind::Terminal, 0.0));
        assert!(
            node(
                Saved::read(&h)
                    .unwrap()
                    .part(Some(Direction::North))
                    .unwrap()
            )
            .unwrap()
            .get(&key("e"))
            .is_none()
        );
    }
    #[test]
    fn observing_one_host_retains_all_neighboring_host_slots_and_their_snapshots() {
        let mut hosts = Hosts::default();
        let mut h = host(Color::Red, 20.0);
        h.insert(key("north"), terminal(1, 4.0));
        for x in -128..128 {
            hosts.observe_primary([x, 2, 3], &h, 0).unwrap();
        }
        let before = hosts
            .observed
            .iter()
            .filter(|(key, _)| key.pos != [0, 2, 3])
            .map(|(key, value)| (*key, value.part.data.clone()))
            .collect::<BTreeMap<_, _>>();
        h.remove(&key("north"));
        let (_, replaced) = hosts.observe_primary([0, 2, 3], &h, 0).unwrap();
        assert_eq!(replaced, vec![NodeKey::part([0, 2, 3], Direction::North)]);
        let after = hosts
            .observed
            .iter()
            .filter(|(key, _)| key.pos != [0, 2, 3])
            .map(|(key, value)| (*key, value.part.data.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(before, after);
        assert_eq!(hosts.observed.len(), 511);
    }
    #[test]
    fn all_source_centers_read_only_gn_not_full_block_proxy_and_keep_facades_and_side_data() {
        for kind in CableType::VALID {
            for color in Color::ALL {
                let cable = Cable { kind, color };
                let mut h = host(color, 42.0);
                center(&mut h).insert(
                    key("id"),
                    Tag::String(cable.item().unwrap().as_str().into()),
                );
                h.insert(
                    key("proxy"),
                    Tag::compound([("e", Tag::Double(999.0_f64.to_bits()))]),
                );
                h.insert(
                    key("facadeEast"),
                    Tag::compound([("Name", Tag::String("minecraft:stone".into()))]),
                );
                let saved = Saved::read(&h).unwrap();
                let s = saved.center_snapshot([1, 2, 3]).unwrap().unwrap();
                assert_eq!(s.kind, Kind::Cable(cable));
                assert_eq!(s.saved_buffer, 42.0);
                assert_eq!(saved.redstone, 2);
                assert!(saved.parts.facade(Direction::East).is_some());
                h.insert(
                    key("north"),
                    Tag::compound([
                        ("id", Tag::String("ae2:terminal".into())),
                        ("custom", Tag::Long(123)),
                    ]),
                );
                let saved = Saved::read(&h).unwrap();
                assert!(saved.center_snapshot([1, 2, 3]).is_err());
                assert!(
                    saved
                        .parts
                        .part(Direction::North)
                        .unwrap()
                        .data
                        .contains_key(&key("custom"))
                );
            }
        }
    }
    #[test]
    fn owner_only_load_preserves_identity_but_external_node_data_or_item_replacement_does_not() {
        let mut hosts = Hosts::default();
        let mut h = host(Color::Red, 20.0);
        let p = [1, 2, 3];
        assert!(!hosts.observe(p, &h).unwrap().1);
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("p"), Tag::Int(99));
        assert!(!hosts.observe(p, &h).unwrap().1);
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("e"), Tag::Double(30.0_f64.to_bits()));
        assert!(hosts.observe(p, &h).unwrap().1);
        center(&mut h).insert(key("id"), Tag::String("ae2:blue_smart_cable".into()));
        assert!(hosts.observe(p, &h).unwrap().1);
        assert!(same_node_data(
            None,
            Some(&Compound::from([(key("p"), Tag::Int(1))]))
        ));
    }
    #[test]
    fn saves_update_only_center_gn_and_never_overwrite_a_new_external_part_snapshot() {
        let mut hosts = Hosts::default();
        let mut h = host(Color::Red, 20.0);
        let p = [0, 0, 0];
        let cable = Cable {
            kind: CableType::Smart,
            color: Color::Red,
        };
        h.insert(key("unknownHost"), Tag::Long(71));
        center(&mut h).insert(key("customName"), Tag::String("saved".into()));
        hosts.observe(p, &h).unwrap();
        assert!(hosts.persist(p, &mut h, cable, 5.0));
        assert!(!hosts.observe(p, &h).unwrap().1);
        assert!(!hosts.persist(p, &mut h, cable, 5.0));
        assert_eq!(
            Saved::read(&h)
                .unwrap()
                .center_snapshot(p)
                .unwrap()
                .unwrap()
                .saved_buffer,
            5.0
        );
        assert_eq!(h.get(&key("unknownHost")), Some(&Tag::Long(71)));
        assert_eq!(
            center(&mut h).get(&key("customName")),
            Some(&Tag::String("saved".into()))
        );
        center(&mut h).insert(key("external"), Tag::Int(4));
        let external = h.clone();
        assert!(!hosts.persist(p, &mut h, cable, 0.0));
        assert_eq!(h, external);
        hosts.observe(p, &h).unwrap();
        assert!(hosts.persist(p, &mut h, cable, 0.0));
        assert_eq!(
            Saved::read(&h)
                .unwrap()
                .center_snapshot(p)
                .unwrap()
                .unwrap()
                .saved_buffer,
            0.0
        );
    }
    #[test]
    fn typed_nbt_restart_keeps_host_part_facade_and_service_data_with_exact_buffer() {
        use horizon_world::nbt::{self, Limits};
        let pos = [3, 4, 5];
        let mut h = host(Color::Cyan, 23.5);
        h.insert(
            key("opaqueHost"),
            Tag::LongArray(vec![i64::MIN, 0, i64::MAX]),
        );
        h.insert(
            key("facadeEast"),
            Tag::compound([("Name", Tag::String("minecraft:stone".into()))]),
        );
        center(&mut h).insert(key("opaquePart"), Tag::ByteArray(vec![0, 127, 255]));
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(
            key("opaqueService"),
            Tag::IntArray(vec![i32::MIN, 42, i32::MAX]),
        );
        let mut hosts = Hosts::default();
        let cable = Saved::read(&h).unwrap().center().unwrap().unwrap();
        hosts.observe(pos, &h).unwrap();
        assert!(hosts.persist(pos, &mut h, cable, 7.25));
        let tag = Tag::Compound(h.clone());
        let bytes = nbt::write_network(&tag, Limits::saved()).unwrap();
        let (loaded, used) = nbt::read_network(&bytes, Limits::saved()).unwrap();
        assert_eq!(used, bytes.len());
        assert_eq!(loaded, tag);
        let Tag::Compound(loaded) = loaded else {
            unreachable!()
        };
        let mut restarted = Hosts::default();
        let (snapshot, replaced) = restarted.observe(pos, &loaded).unwrap();
        assert!(!replaced);
        assert_eq!(snapshot.unwrap().saved_buffer, 7.25);
        assert_eq!(loaded.get(&key("opaqueHost")), h.get(&key("opaqueHost")));
        assert!(
            Saved::read(&loaded)
                .unwrap()
                .parts
                .facade(Direction::East)
                .is_some()
        );
    }
    #[test]
    fn malformed_or_unsupported_host_refusal_preserves_the_entire_original_nbt() {
        let mut hosts = Hosts::default();
        for energy in [f64::NAN, f64::INFINITY, -1.0] {
            let h = host(Color::Red, energy);
            let original = h.clone();
            assert!(hosts.observe([0, 0, 0], &h).is_err());
            assert_eq!(h, original);
        }
        let mut h = host(Color::Red, 5.0);
        h.insert(key("hasRedstone"), Tag::Int(3));
        assert!(hosts.observe([0, 0, 0], &h).is_err());
        h.insert(key("hasRedstone"), Tag::Byte(1));
        assert!(hosts.observe([0, 0, 0], &h).is_err());
        h.remove(&key("hasRedstone"));
        center(&mut h).insert(key("id"), Tag::String("other:missing_part".into()));
        let original = h.clone();
        assert!(hosts.observe([0, 0, 0], &h).is_err());
        assert_eq!(h, original);
    }
    #[test]
    fn live_graph_buffer_is_not_refilled_by_our_save_and_external_gn_reload_replaces_the_node() {
        let pos = [1, 1, 1];
        let mut h = host(Color::Red, 100.0);
        let mut hosts = Hosts::default();
        let mut network = Network::default();
        let snapshots = hosts
            .snapshot(pos, &h, &mut network)
            .unwrap()
            .into_iter()
            .collect();
        network.sync(snapshots);
        let first = network.node_service(pos).unwrap().1;
        // The source's grid-local buffer is 25 AE per node, not item storage.
        assert_eq!(network.extract(pos, 1000.0, 1.0, false), 25.0);
        assert_eq!(network.extract(pos, 10.0, 1.0, true), 10.0);
        for _ in 0..3 {
            let state = network.end_tick().remove(0);
            let Kind::Cable(cable) = state.kind else {
                unreachable!()
            };
            hosts.persist(pos, &mut h, cable, state.buffer_share);
            let snapshots = hosts
                .snapshot(pos, &h, &mut network)
                .unwrap()
                .into_iter()
                .collect();
            network.sync(snapshots);
            assert_eq!(network.node_service(pos).unwrap().1, first);
            assert_eq!(network.extract(pos, 1000.0, 1.0, false), 15.0);
        }
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("p"), Tag::Int(99));
        let snapshots = hosts
            .snapshot(pos, &h, &mut network)
            .unwrap()
            .into_iter()
            .collect();
        network.sync(snapshots);
        assert_eq!(network.node_service(pos).unwrap().1, first);
        assert_eq!(network.extract(pos, 1000.0, 1.0, false), 15.0);
        let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
            unreachable!()
        };
        gn.insert(key("e"), Tag::Double(55.0_f64.to_bits()));
        let snapshots = hosts
            .snapshot(pos, &h, &mut network)
            .unwrap()
            .into_iter()
            .collect();
        network.sync(snapshots);
        assert_ne!(network.node_service(pos).unwrap().1, first);
        assert_eq!(network.extract(pos, 1000.0, 1.0, false), 25.0);
    }
    #[test]
    fn only_double_typed_saved_node_energy_is_restored_as_in_energy_service_add_node() {
        let mut h = host(Color::Red, 5.0);
        for value in [
            Tag::Byte(5),
            Tag::Short(5),
            Tag::Int(5),
            Tag::Long(5),
            Tag::Float(5.0_f32.to_bits()),
            Tag::String("5".into()),
        ] {
            let Tag::Compound(gn) = center(&mut h).get_mut(&key("gn")).unwrap() else {
                unreachable!()
            };
            gn.insert(key("e"), value);
            assert_eq!(
                Saved::read(&h)
                    .unwrap()
                    .center_snapshot([0, 0, 0])
                    .unwrap()
                    .unwrap()
                    .saved_buffer,
                0.0
            );
        }
    }
}
