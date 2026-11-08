// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::ae2_transform_throw::{claims, fluid_tick};
use std::{fmt::Write, io::Read};
fn main() {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).unwrap();
    let mut words = text.split_whitespace();
    let mut next = || words.next().unwrap().parse::<i64>().unwrap();
    let cases = next();
    let mut out = String::new();
    for _ in 0..cases {
        let mut clock = next() as i32;
        let trigger = next() as usize;
        let len = next() as usize;
        let mut kinds = Vec::new();
        let mut counts = Vec::new();
        let mut alive = Vec::new();
        for _ in 0..len {
            kinds.push(next() as u32);
            counts.push(next() as u32);
            alive.push(next() != 0);
        }
        let mut recipes = Vec::new();
        for _ in 0..next() {
            let n = next();
            recipes.push((0..n).map(|_| next() as u32).collect::<Vec<_>>());
        }
        let mut products = 0;
        let mut draws = 0;
        for _ in 0..next() {
            let wet = next() != 0;
            let eligible = counts[trigger] > 0
                && recipes.iter().any(|r| {
                    r.first()
                        .is_some_and(|mask| mask & (1 << kinds[trigger]) != 0)
                });
            fluid_tick(
                &mut clock,
                !alive[trigger],
                eligible,
                wet && eligible,
                || {
                    let available = counts
                        .iter()
                        .zip(&alive)
                        .map(|(n, live)| if *live { *n } else { 0 })
                        .collect::<Vec<_>>();
                    for recipe in &recipes {
                        let matches = recipe
                            .iter()
                            .map(|mask| kinds.iter().map(|kind| mask & (1 << kind) != 0).collect())
                            .collect::<Vec<_>>();
                        if let Some(claimed) = claims(&available, &matches, trigger) {
                            for i in 0..len {
                                if claimed[i] > 0 {
                                    counts[i] -= claimed[i];
                                    if counts[i] == 0 {
                                        alive[i] = false;
                                    }
                                }
                            }
                            products += 1;
                            draws += 6;
                            return true;
                        }
                    }
                    false
                },
            );
            write!(out, "{clock} {products} {draws}").unwrap();
            for i in 0..len {
                write!(out, " {} {}", counts[i], u8::from(alive[i])).unwrap();
            }
            out.push('\n');
        }
    }
    print!("{out}");
}
