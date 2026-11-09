// SPDX-License-Identifier: LGPL-3.0-or-later
// MeteoritePlacer/MeteoriteBlockPutter callback order from pinned AE2 19.2.18.
use super::ae2_meteorite_fallout::{self as fallout, CopyBiome, CraterType, FalloutMode};
pub type Pos = [i32; 3];

// Preflight every requested state before writes or RNG draws. Missing content
// must refuse the probe rather than silently produce an incomplete meteor.
pub const STATE_IDS: &[&str] = &[
    "minecraft:air",
    "minecraft:stone",
    "minecraft:lava",
    "minecraft:obsidian",
    "minecraft:water",
    "minecraft:snow_block",
    "minecraft:ice",
    "minecraft:terracotta",
    "minecraft:sand",
    "minecraft:dirt",
    "minecraft:cobblestone",
    "minecraft:gravel",
    "minecraft:grass_block",
    "minecraft:glass",
    "minecraft:snow",
    "minecraft:magma_block",
    "ae2:sky_stone_block",
    "ae2:quartz_block",
    "ae2:damaged_budding_quartz",
    "ae2:chipped_budding_quartz",
    "ae2:flawed_budding_quartz",
    "ae2:flawless_budding_quartz",
    "ae2:small_quartz_bud",
    "ae2:medium_quartz_bud",
    "ae2:large_quartz_bud",
    "ae2:mysterious_cube",
];

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub min: Pos,
    pub max: Pos,
}
impl Bounds {
    pub fn contains(self, p: Pos) -> bool {
        (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
    }
    pub fn center(self) -> Pos {
        std::array::from_fn(|i| {
            self.min[i].wrapping_add(self.max[i].wrapping_sub(self.min[i]).wrapping_add(1) / 2)
        })
    }
    fn clamp(self, axis: usize, n: i32) -> i32 {
        if n < self.min[axis] {
            self.min[axis]
        } else if n > self.max[axis] {
            self.max[axis]
        } else {
            n
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub pos: Pos,
    pub radius: f32,
    pub crater: CraterType,
    pub fallout: FalloutMode,
    pub pure: bool,
    pub lake: bool,
    pub flawless_only: bool,
    pub spawn_presses: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Properties {
    pub air: bool,
    pub solid: bool,
    pub replaceable: bool,
    pub fluid_empty: bool,
    pub replaceable_tag: bool,
}
/// State values must preserve full block-state identity, including copied
/// orientation/water/variant properties. Placement and level RNG may alias;
/// their draw order must still be preserved by the binding.
pub trait World {
    type State: Clone + PartialEq;
    fn block(&mut self, pos: Pos) -> Self::State;
    fn state(&self, id: &'static str) -> Self::State;
    fn bud_up(&self, id: &'static str) -> Self::State;
    fn is_block(&self, state: &Self::State, id: &'static str) -> bool;
    fn properties(&self, state: &Self::State) -> Properties;
    fn set(&mut self, pos: Pos, state: Self::State);
    fn max_build_height(&self) -> i32;
    fn sea_level(&self) -> i32;
    fn copy_biome(&mut self, pos: Pos) -> CopyBiome;
    fn discard_items(&mut self, bounds: [f64; 6]);
    fn schedule_water(&mut self, pos: Pos);
    fn next_int(&mut self, bound: i32) -> i32;
    fn next_float(&mut self) -> f32;
    fn next_double(&mut self) -> f64;
    fn level_float(&mut self) -> f32;
}

const QUARTZ: [&str; 5] = [
    "ae2:quartz_block",
    "ae2:damaged_budding_quartz",
    "ae2:chipped_budding_quartz",
    "ae2:flawed_budding_quartz",
    "ae2:flawless_budding_quartz",
];
const BUDS: [&str; 3] = [
    "ae2:small_quartz_bud",
    "ae2:medium_quartz_bud",
    "ae2:large_quartz_bud",
];
const SKY: &str = "ae2:sky_stone_block";
const AIR: &str = "minecraft:air";

struct Placer<'a, W: World> {
    world: &'a mut W,
    settings: Settings,
    bounds: Bounds,
    radius: f64,
    squared: f64,
    crater: f64,
    specified: &'static str,
}
impl<W: World> Placer<'_, W> {
    fn put(&mut self, pos: Pos, state: W::State) -> bool {
        let old = self.world.block(pos);
        if self.world.is_block(&old, "minecraft:bedrock") || old == state {
            return false;
        }
        self.world.set(pos, state);
        true
    }
    fn put_id(&mut self, pos: Pos, id: &'static str) {
        let state = self.world.state(id);
        self.put(pos, state);
    }
    fn bowl(&self, x: i32, z: i32) -> f64 {
        let dx = f64::from(x.wrapping_sub(self.settings.pos[0]));
        let dz = f64::from(z.wrapping_sub(self.settings.pos[2]));
        let h = f64::from(self.settings.pos[1]) - self.radius
            + 1.0
            + f64::from(fallout::adjust_crater(self.settings.fallout));
        h + (dx * dx + dz * dz) * 0.02
    }
    fn place_crater(&mut self) {
        let [x, y, z] = self.settings.pos;
        let filler = fallout::filler(self.settings.crater).expect("crater phase excludes NONE");
        for j in y.wrapping_sub(5)..=self.world.max_build_height() {
            for i in self.bounds.min[0]..=self.bounds.max[0] {
                for k in self.bounds.min[2]..=self.bounds.max[2] {
                    if f64::from(j) > self.bowl(i, k) {
                        let pos = [i, j, k];
                        let current = self.world.block(pos);
                        if self.settings.crater != CraterType::Normal
                            && j < y
                            && self.world.properties(&current).solid
                        {
                            self.put_id(pos, filler);
                        } else {
                            self.put_id(pos, AIR);
                        }
                    }
                }
            }
        }
        self.world.discard_items([
            f64::from(self.bounds.clamp(0, x.wrapping_sub(30))),
            f64::from(y.wrapping_sub(5)),
            f64::from(self.bounds.clamp(2, z.wrapping_sub(30))),
            f64::from(self.bounds.clamp(0, x.wrapping_add(30))),
            f64::from(y.wrapping_add(30)),
            f64::from(self.bounds.clamp(2, z.wrapping_add(30))),
        ]);
    }
    fn sky_stone(&mut self) {
        let [x, y, z] = self.settings.pos;
        for i in self.bounds.clamp(0, x.wrapping_sub(8))..=self.bounds.clamp(0, x.wrapping_add(8)) {
            for j in y.wrapping_sub(8)..y.wrapping_add(8) {
                for k in self.bounds.clamp(2, z.wrapping_sub(8))
                    ..=self.bounds.clamp(2, z.wrapping_add(8))
                {
                    let (dx, dy, dz) = (i.wrapping_sub(x), j.wrapping_sub(y), k.wrapping_sub(z));
                    let distance = f64::from(dx.wrapping_mul(dx)) * 0.7
                        + f64::from(dy.wrapping_mul(dy)) * if j > y { 1.4 } else { 0.8 }
                        + f64::from(dz.wrapping_mul(dz)) * 0.7;
                    if distance < self.squared {
                        if dx.wrapping_abs() <= 1
                            && dy.wrapping_abs() <= 1
                            && dz.wrapping_abs() <= 1
                        {
                            if dy == -1 {
                                let quartz = if self.settings.flawless_only {
                                    &QUARTZ[4..]
                                } else {
                                    &QUARTZ[..]
                                };
                                let index = self.world.next_int(quartz.len() as i32) as usize;
                                self.put_id([i, j, k], quartz[index]);
                                if index != 0
                                    && (dx != 0 || dz != 0)
                                    && self.world.next_float() <= 0.7
                                {
                                    let bud = BUDS[self.world.next_int(3) as usize];
                                    let state = self.world.bud_up(bud);
                                    self.put([i, j.wrapping_add(1), k], state);
                                }
                            }
                        } else {
                            self.put_id([i, j, k], SKY);
                        }
                    }
                }
            }
        }
        if self.bounds.contains(self.settings.pos) && self.settings.spawn_presses {
            self.put_id(self.settings.pos, "ae2:mysterious_cube");
        }
    }
    fn fall(&mut self, pos: Pos) {
        let roll = self.world.next_float();
        if let Some(block) = fallout::fall_block(self.settings.fallout, self.specified, roll) {
            self.put_id(pos, block);
        }
    }
    fn inset(&mut self, pos: Pos) {
        let roll = self.world.next_float();
        if let Some(block) = fallout::inset_block(self.settings.fallout, self.specified, roll) {
            self.put_id(pos, block);
        }
    }
    fn decay(&mut self) {
        let [x, y, z] = self.settings.pos;
        let mut random_shit = 0.0;
        let filler = fallout::filler(self.settings.crater).expect("decay phase excludes NONE");
        for i in self.bounds.clamp(0, x.wrapping_sub(30))..=self.bounds.clamp(0, x.wrapping_add(30))
        {
            for k in
                self.bounds.clamp(2, z.wrapping_sub(30))..=self.bounds.clamp(2, z.wrapping_add(30))
            {
                for j in y.wrapping_sub(9)..y.wrapping_add(30) {
                    let pos = [i, j, k];
                    let up = [i, j.wrapping_add(1), k];
                    let down = [i, j.wrapping_sub(1), k];
                    let state = self.world.block(pos);
                    let block = self.world.block(pos);
                    if self.settings.pure && self.world.is_block(&block, filler) {
                        continue;
                    }
                    if self.world.properties(&state).replaceable {
                        let above = self.world.block(up);
                        if !self.world.properties(&above).air {
                            // This source path calls Level.setBlock directly, preserving
                            // the complete state above; it is not MeteoriteBlockPutter.
                            let state_up = self.world.block(up);
                            self.world.set(pos, state_up);
                        } else if random_shit < 100.0 * self.crater {
                            let (dx, dy, dz) = (
                                f64::from(i.wrapping_sub(x)),
                                f64::from(j.wrapping_sub(y)),
                                f64::from(k.wrapping_sub(z)),
                            );
                            let dist = dx * dx + dy * dy + dz * dz;
                            let below = self.world.block(down);
                            if !self.world.properties(&below).replaceable {
                                let extra = self.world.next_double() * 0.6;
                                let height =
                                    self.crater * (extra + 0.2) - (dist - self.crater * 1.7).abs();
                                if !self.world.properties(&below).air
                                    && height > 0.0
                                    && self.world.next_double() > 0.6
                                {
                                    random_shit += 1.0;
                                    self.fall(pos);
                                }
                            }
                        }
                    } else {
                        let above = self.world.block(up);
                        if self.world.properties(&above).air && self.world.next_double() > 0.4 {
                            let (dx, dy, dz) = (
                                f64::from(i.wrapping_sub(x)),
                                f64::from(j.wrapping_sub(y)),
                                f64::from(k.wrapping_sub(z)),
                            );
                            let dr2 = dx * dx + dy * dy + dz * dz;
                            if !(dx.abs() <= 1.0 && dy.abs() <= 1.0 && dz.abs() <= 1.0)
                                && dr2 < self.crater * 1.6
                            {
                                self.inset(pos);
                            }
                        }
                    }
                }
            }
        }
    }
    fn enclose(&mut self, pos: Pos) -> bool {
        let state = self.world.block(pos);
        let props = self.world.properties(&state);
        if self.world.is_block(&state, AIR)
            || (props.fluid_empty && (props.replaceable || props.replaceable_tag))
        {
            if self.settings.crater == CraterType::Lava && self.world.level_float() < 0.075f32 {
                self.put_id(pos, "minecraft:magma_block");
            } else {
                self.fall(pos);
            }
            false
        } else {
            true
        }
    }
    fn lake(&mut self) {
        let max_y = self.world.sea_level().wrapping_sub(1);
        for i in self.bounds.min[0]..=self.bounds.max[0] {
            for k in self.bounds.min[2]..=self.bounds.max[2] {
                for j in self.settings.pos[1].wrapping_sub(5)..=max_y {
                    let threshold = self.bowl(i, k);
                    if f64::from(j) > threshold {
                        let pos = [i, j, k];
                        let state = self.world.block(pos);
                        if self.world.is_block(&state, AIR) {
                            self.put_id(pos, "minecraft:water");
                            if j == max_y {
                                self.world.schedule_water(pos);
                            }
                        }
                    } else if f64::from(
                        max_y
                            .wrapping_add(max_y.wrapping_sub(j).wrapping_mul(2))
                            .wrapping_add(2),
                    ) > threshold
                    {
                        let mut pos = [i, j, k];
                        for _ in 0..20 {
                            if self.enclose(pos) {
                                break;
                            }
                            pos[1] = pos[1].wrapping_sub(1);
                        }
                    }
                }
            }
        }
    }
}

pub fn place<W: World>(world: &mut W, settings: Settings, bounds: Bounds) {
    let radius = f64::from(settings.radius);
    let squared = radius * radius;
    let real_crater = radius * 2.0 + 5.0;
    let specified = if matches!(
        settings.fallout,
        FalloutMode::Sand | FalloutMode::Terracotta | FalloutMode::IceSnow
    ) {
        fallout::copy_block(world.copy_biome(bounds.center()))
    } else {
        "minecraft:cobblestone"
    };
    let mut placer = Placer {
        world,
        settings,
        bounds,
        radius,
        squared,
        crater: real_crater * real_crater,
        specified,
    };
    if settings.crater != CraterType::None {
        placer.place_crater();
    }
    placer.sky_stone();
    if settings.crater != CraterType::None {
        placer.decay();
    }
    if settings.lake {
        placer.lake();
    }
}
