// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::{
    ae2_cable::{Cable, CableType, Color},
    ae2_grid::{Direction, Flags},
    ae2_network::{Kind, Network, NodeKey, Snapshot},
};
use std::{
    collections::BTreeSet,
    io::{self, BufRead, BufWriter, Write},
};
fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let values = line
            .split_whitespace()
            .map(|s| s.parse::<u8>().unwrap())
            .collect::<Vec<_>>();
        let (kind, parts, blocked, neighbors) = (values[0], values[1], values[2], values[3]);
        let p = [0, 0, 0];
        let mut snapshots = vec![];
        if kind != 0 {
            snapshots.push(Snapshot {
                pos: p,
                side: None,
                kind: Kind::Cable(Cable {
                    kind: CableType::from_ordinal(usize::from(kind)).unwrap(),
                    color: Color::Transparent,
                }),
                front: Direction::North,
                exposed: 63 & !blocked,
                stored: 0.0,
                saved_buffer: 0.0,
            });
        }
        for side in Direction::ALL {
            if parts & (1 << side as u8) != 0 {
                snapshots.push(Snapshot::terminal(p, side, 0.0));
            }
            if neighbors & (1 << side as u8) != 0 {
                snapshots.push(Snapshot {
                    pos: side.offset(),
                    side: None,
                    kind: Kind::Drive,
                    front: side,
                    exposed: 63,
                    stored: 0.0,
                    saved_buffer: 0.0,
                });
            }
        }
        let mut n = Network::default();
        n.sync(snapshots);
        let mut ids = BTreeSet::new();
        if let Some((_, id)) = n.node_service(p) {
            ids.insert(id);
        }
        let mut internal = 0;
        let mut external = 0;
        for side in Direction::ALL {
            let key = NodeKey::part(p, side);
            if let Some((_, id)) = n.node_service(key) {
                ids.insert(id);
            }
            if let Some((direction, _)) = n.connection(p, key) {
                assert_eq!(direction, None);
                internal |= 1 << side as u8;
            }
            if let Some((direction, _)) = n.connection(p, side.offset()) {
                assert_eq!(direction, Some(side));
                external |= 1 << side as u8;
            }
        }
        let flags = Kind::Terminal.node_flags();
        let mut bits = 0;
        for bit in 0..7 {
            if flags.has(Flags::from_bits(1 << bit).unwrap()) {
                bits |= 1 << bit;
            }
        }
        write!(
            out,
            "{} {internal} {external} {bits} {}",
            ids.len(),
            (Kind::Terminal.base_idle_power() * 16.0) as i32
        )
        .unwrap();
        let raw = n.cable_stream(p).unwrap_or_default();
        let view = n.cable_visual(p).unwrap_or_default();
        write!(out, " {}", raw.connections).unwrap();
        for channels in raw.channels {
            write!(out, " {channels}").unwrap();
        }
        for channels in view.channels {
            write!(out, " {channels}").unwrap();
        }
        writeln!(out).unwrap();
    }
}
