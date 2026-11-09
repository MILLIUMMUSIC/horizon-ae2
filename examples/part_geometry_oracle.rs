// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::{
    ae2_cable::CableType,
    ae2_grid::Direction,
    ae2_part_geometry::{CollisionHelper, reporting_host_boxes, select_primary},
};
use std::io::{self, BufRead, BufWriter, Write};

fn number(value: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(value, 16).unwrap())
}
fn bits(value: f64) -> u64 {
    if value.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        value.to_bits()
    }
}
fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let s: Vec<_> = line.split_whitespace().collect();
        let int = |i: usize| s[i].parse::<usize>().unwrap();
        match s[0] {
            "B" | "A" => {
                let (helper, start) = if s[0] == "B" {
                    (
                        CollisionHelper::new(Direction::ALL.get(int(1)).copied(), int(2) != 0),
                        3,
                    )
                } else {
                    (
                        CollisionHelper::with_axes(
                            [
                                Direction::ALL[int(1)],
                                Direction::ALL[int(2)],
                                Direction::ALL[int(3)],
                            ],
                            int(4) != 0,
                        ),
                        5,
                    )
                };
                let b = helper.add_box(std::array::from_fn(|i| number(s[start + i])));
                let point = std::array::from_fn(|i| number(s[start + 6 + i]));
                for d in helper.axes() {
                    write!(out, "{} ", d as u8).unwrap();
                }
                write!(out, "{}", u8::from(helper.is_bb_collision())).unwrap();
                for v in b.0 {
                    write!(out, " {:x}", bits(v)).unwrap();
                }
                writeln!(out, " {}", u8::from(b.inflate(0.002).contains(point))).unwrap();
            }
            "H" => {
                let kind = CableType::from_ordinal(int(1)).unwrap();
                let (parts, conn, dense, visual) =
                    (int(2) as u8, int(3) as u8, int(4) as u8, int(5) != 0);
                let boxes = reporting_host_boxes(kind, parts, conn, dense, visual);
                write!(out, "{}", boxes.len()).unwrap();
                for (slot, b) in boxes {
                    write!(out, " {}", slot.map_or(6, |d| d as u8)).unwrap();
                    for v in b.0 {
                        write!(out, " {:x}", bits(v)).unwrap();
                    }
                }
                let point = std::array::from_fn(|i| number(s[6 + i]));
                let selected =
                    select_primary(&reporting_host_boxes(kind, parts, conn, dense, true), point)
                        .map_or(-1, |slot| slot.map_or(6, |d| d as i32));
                writeln!(out, " {selected}").unwrap();
            }
            _ => panic!("unknown geometry operation"),
        }
    }
}
