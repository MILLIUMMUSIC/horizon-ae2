// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::{
    ae2_grid::Direction,
    ae2_reporting::{Model, Number, Reporting},
};
use std::io::{self, BufRead, BufWriter, Write};
fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let s: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| s[i].parse::<i32>().unwrap();
        match s[0] {
            "P" => {
                let mut p = Reporting::new(n(2) as i8);
                p.on_placement(
                    Direction::ALL[n(1) as usize],
                    f32::from_bits(u32::from_str_radix(s[3], 16).unwrap()),
                );
                writeln!(out, "{}", p.spin()).unwrap();
            }
            "W" => {
                let mut p = Reporting::new(n(1) as i8);
                let allowed = n(3) != 0;
                let changed = allowed && p.wrench_turn(n(2) != 0);
                writeln!(
                    out,
                    "{} {} {} {}",
                    p.spin(),
                    u8::from(allowed),
                    u8::from(changed),
                    u8::from(changed)
                )
                .unwrap();
            }
            "F" => {
                let mut p = Reporting::new(n(2) as i8);
                p.flags.read(n(1) as u8, true, true);
                let changed = p.read_stream([n(3) as u8, n(4) as u8], n(5) != 0, n(6) != 0);
                let model = match p.flags.model() {
                    Model::Off => 0,
                    Model::On => 1,
                    Model::HasChannel => 2,
                };
                let wire = p.write_stream(p.flags);
                writeln!(
                    out,
                    "{} {} {} {} {} {}",
                    u8::from(changed),
                    p.flags.byte(),
                    p.spin(),
                    model,
                    wire[0],
                    wire[1]
                )
                .unwrap();
            }
            "N" => {
                let value = match n(1) {
                    1 => Some(Number::Byte(n(2) as i8)),
                    2 => Some(Number::Short(n(2) as i16)),
                    3 => Some(Number::Int(n(2))),
                    4 => Some(Number::Long(s[2].parse().unwrap())),
                    5 => Some(Number::Float(f32::from_bits(
                        u32::from_str_radix(s[2], 16).unwrap(),
                    ))),
                    6 => Some(Number::Double(f64::from_bits(
                        u64::from_str_radix(s[2], 16).unwrap(),
                    ))),
                    _ => None,
                };
                let mut p = Reporting::default();
                p.read_spin(value);
                writeln!(out, "{}", p.spin()).unwrap();
            }
            "L" => {
                let mut p = Reporting::default();
                let side = Direction::ALL[n(1) as usize];
                let a = n(2);
                let b = n(3);
                let mut queries = 0;
                let off = p.light(false, false, || {
                    queries += 1;
                    a
                });
                let powered = p.light(n(4) != 0, n(5) != 0, || {
                    queries += 1;
                    b
                });
                let changed = p.neighbor_changed([i32::MAX, 2, 3], side, [n(6), n(7), n(8)]);
                let after = p.light(n(4) != 0, n(5) != 0, || {
                    queries += 1;
                    b
                });
                writeln!(
                    out,
                    "{off} {powered} {after} {queries} {}",
                    u8::from(changed)
                )
                .unwrap();
            }
            _ => panic!("unknown reporting operation"),
        }
    }
}
