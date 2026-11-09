// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::{
    ae2_grid::Direction,
    ae2_quad_rotation::{PackedQuad, pack_normal, transform},
};
use std::io::{self, BufRead, BufWriter, Write};
fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let s: Vec<_> = line.split_whitespace().collect();
        let word = |i: usize| u32::from_str_radix(s[i], 16).unwrap();
        if s[0] == "N" {
            writeln!(
                out,
                "{:x}",
                pack_normal(std::array::from_fn(|i| f32::from_bits(word(i + 1))))
            )
            .unwrap();
            continue;
        }
        let facing = s[1].parse::<usize>().unwrap();
        let spin = s[2].parse::<i32>().unwrap();
        let mut quad = PackedQuad {
            header: std::array::from_fn(|i| word(i + 3)),
            vertices: std::array::from_fn(|i| std::array::from_fn(|j| word(6 + i * 8 + j))),
            nominal: Direction::ALL[s[38].parse::<usize>().unwrap()],
            geometry_invalid: s[39] == "1",
        };
        let index = (facing as i32).wrapping_mul(4).wrapping_add(spin);
        let result = transform(&mut quad, Direction::ALL[facing], spin);
        write!(
            out,
            "{} {} {}",
            if result.is_ok() { "OK" } else { "ERR" },
            quad.nominal as u8,
            u8::from(quad.geometry_invalid)
        )
        .unwrap();
        for v in quad.header {
            write!(out, " {v:x}").unwrap();
        }
        for vertex in quad.vertices {
            for (i, mut word) in vertex.into_iter().enumerate() {
                if result.is_ok() && index != 8 && i < 3 && f32::from_bits(word).is_nan() {
                    word = 0x7fc0_0000;
                }
                write!(out, " {word:x}").unwrap();
            }
        }
        writeln!(out).unwrap();
    }
}
