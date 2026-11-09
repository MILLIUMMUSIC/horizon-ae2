// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::{
    ae2_meteorite_fallout::{CopyBiome, CraterType, FalloutMode},
    ae2_meteorite_placer::{self as meteorite_placer, Bounds, Pos, Properties, Settings, World},
};
use std::{
    collections::BTreeMap,
    fmt::Write,
    io::{self, BufRead, BufWriter, Write as IoWrite},
};
struct Random(u64);
impl Random {
    fn new(seed: i64) -> Self {
        Self((seed as u64 ^ 0x5deece66d) & ((1 << 48) - 1))
    }
    fn bits(&mut self, n: u32) -> u32 {
        self.0 = self.0.wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
        (self.0 >> (48 - n)) as u32
    }
    fn int(&mut self, bound: i32) -> i32 {
        if bound & (bound - 1) == 0 {
            return ((i64::from(bound) * i64::from(self.bits(31))) >> 31) as i32;
        }
        loop {
            let bits = self.bits(31) as i32;
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }
    fn float(&mut self) -> f32 {
        self.bits(24) as f32 / (1u32 << 24) as f32
    }
    fn double(&mut self) -> f64 {
        let a = u64::from(self.bits(26));
        let b = u64::from(self.bits(27));
        ((a << 27) + b) as f64 / (1u64 << 53) as f64
    }
}
struct Fixture {
    max: i32,
    sea: i32,
    floor: i32,
    terrain: i32,
    mask: i32,
    program: Random,
    level: Random,
    alias: bool,
    blocks: BTreeMap<Pos, String>,
    trace: String,
}
fn base(state: &str) -> &str {
    state.split('[').next().unwrap()
}
fn default_state(id: &str) -> String {
    if id == "minecraft:snow" {
        format!("{id}[layers=1]")
    } else {
        id.into()
    }
}
fn point(p: Pos) -> String {
    format!("{},{},{}", p[0], p[1], p[2])
}
impl Fixture {
    fn initial(&self, [x, y, z]: Pos) -> String {
        if y < -16 || y >= self.max {
            return "minecraft:air".into();
        }
        let n = x
            .wrapping_mul(31)
            .wrapping_add(z.wrapping_mul(17))
            .wrapping_add(self.terrain)
            .rem_euclid(23);
        let ground = self.floor + n % 5 - 2;
        if y <= ground {
            if n % 11 == 0 && y >= ground - 2 {
                return "minecraft:bedrock".into();
            }
            return if y == ground {
                "minecraft:dirt"
            } else {
                "minecraft:stone"
            }
            .into();
        }
        if y == ground + 1 {
            match n % 7 {
                0 => return "minecraft:short_grass".into(),
                1 => return "minecraft:snow[layers=1]".into(),
                2 => return "minecraft:cave_air".into(),
                3 => return "minecraft:void_air".into(),
                4 => return "test:tagged".into(),
                _ => {}
            }
        }
        if y == ground + 2 && n % 3 == 0 {
            return "minecraft:oak_stairs[facing=east,waterlogged=true]".into();
        }
        if y < self.sea && n % 4 == 0 {
            return "minecraft:water".into();
        }
        "minecraft:air".into()
    }
    fn finish(mut self) -> String {
        self.trace.push_str("END:");
        for (p, state) in self.blocks {
            write!(self.trace, "{}:{state};", point(p)).unwrap();
        }
        self.trace
    }
}
impl World for Fixture {
    type State = String;
    fn block(&mut self, p: Pos) -> String {
        self.blocks
            .get(&p)
            .cloned()
            .unwrap_or_else(|| self.initial(p))
    }
    fn state(&self, id: &'static str) -> String {
        assert!(
            meteorite_placer::STATE_IDS.contains(&id),
            "unregistered output: {id}"
        );
        default_state(id)
    }
    fn bud_up(&self, id: &'static str) -> String {
        assert!(
            meteorite_placer::STATE_IDS.contains(&id),
            "unregistered bud: {id}"
        );
        format!("{id}[facing=up]")
    }
    fn is_block(&self, state: &String, id: &'static str) -> bool {
        base(state) == id
    }
    fn properties(&self, state: &String) -> Properties {
        let id = base(state);
        let air = matches!(
            id,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        );
        let liquid = matches!(id, "minecraft:water" | "minecraft:lava");
        let plant = matches!(id, "minecraft:short_grass" | "minecraft:snow");
        Properties {
            air,
            solid: !air
                && !liquid
                && !plant
                && !matches!(
                    id,
                    "test:tagged"
                        | "ae2:small_quartz_bud"
                        | "ae2:medium_quartz_bud"
                        | "ae2:large_quartz_bud"
                ),
            replaceable: air || liquid || plant,
            fluid_empty: !liquid && !state.contains("waterlogged=true"),
            replaceable_tag: matches!(id, "test:tagged" | "minecraft:short_grass"),
        }
    }
    fn set(&mut self, p: Pos, state: String) {
        write!(self.trace, "S:{}:{state}:3;", point(p)).unwrap();
        if p[1] >= -16 && p[1] < self.max {
            self.blocks.insert(p, state);
        }
    }
    fn max_build_height(&self) -> i32 {
        self.max
    }
    fn sea_level(&self) -> i32 {
        self.sea
    }
    fn copy_biome(&mut self, p: Pos) -> CopyBiome {
        let mask = self.mask
            ^ p[0]
                .wrapping_add(p[1].wrapping_mul(2))
                .wrapping_add(p[2].wrapping_mul(3))
                .rem_euclid(64);
        write!(self.trace, "B:{}:{mask};", point(p)).unwrap();
        CopyBiome {
            badlands: mask & 1 != 0,
            snowy: mask & 2 != 0,
            beach: mask & 4 != 0,
            sandy: mask & 8 != 0,
            plains: mask & 16 != 0,
            forest: mask & 32 != 0,
        }
    }
    fn discard_items(&mut self, b: [f64; 6]) {
        self.trace.push('E');
        for n in b {
            write!(self.trace, ":{:x}", n.to_bits()).unwrap();
        }
        self.trace.push_str(";X;");
    }
    fn schedule_water(&mut self, p: Pos) {
        write!(self.trace, "W:{}:0;", point(p)).unwrap();
    }
    fn next_int(&mut self, bound: i32) -> i32 {
        let n = self.program.int(bound);
        write!(self.trace, "PI:{bound}:{n};").unwrap();
        n
    }
    fn next_float(&mut self) -> f32 {
        let n = self.program.float();
        write!(self.trace, "PF:{:x};", n.to_bits()).unwrap();
        n
    }
    fn next_double(&mut self) -> f64 {
        let n = self.program.double();
        write!(self.trace, "PD:{:x};", n.to_bits()).unwrap();
        n
    }
    fn level_float(&mut self) -> f32 {
        let n = if self.alias {
            self.program.float()
        } else {
            self.level.float()
        };
        write!(self.trace, "LF:{:x};", n.to_bits()).unwrap();
        n
    }
}
fn main() {
    let mut out = BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let s: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| s[i].parse::<i32>().unwrap();
        let settings = Settings {
            pos: [n(0), n(1), n(2)],
            radius: f32::from_bits(u32::from_str_radix(s[3], 16).unwrap()),
            crater: [
                CraterType::None,
                CraterType::Normal,
                CraterType::Lava,
                CraterType::Obsidian,
                CraterType::Water,
                CraterType::Snow,
                CraterType::Ice,
            ][n(4) as usize],
            fallout: [
                FalloutMode::None,
                FalloutMode::Default,
                FalloutMode::Sand,
                FalloutMode::Terracotta,
                FalloutMode::IceSnow,
            ][n(5) as usize],
            pure: n(6) != 0,
            lake: n(7) != 0,
            flawless_only: n(8) != 0,
            spawn_presses: n(9) != 0,
        };
        let bounds = Bounds {
            min: [n(10), n(11), n(12)],
            max: [n(13), n(14), n(15)],
        };
        let mut world = Fixture {
            max: n(16),
            sea: n(17),
            floor: n(18),
            terrain: n(19),
            mask: n(20),
            program: Random::new(s[21].parse().unwrap()),
            level: Random::new(s[22].parse().unwrap()),
            alias: n(23) != 0,
            blocks: BTreeMap::new(),
            trace: String::new(),
        };
        meteorite_placer::place(&mut world, settings, bounds);
        writeln!(out, "{}", world.finish()).unwrap();
    }
}
