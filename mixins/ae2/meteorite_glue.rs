// SPDX-License-Identifier: LGPL-3.0-or-later
use super::ae2_meteorite_placer::{self as placer, Pos, Properties, World};
use super::{Game, ae2_meteorite_command as command, ae2_meteorite_fallout::CopyBiome};
use horizon_client::text::Text;
use horizon_core::{
    geometry::{Aabb, Vec3},
    position::BlockPos,
    random::RandomSource,
};
use horizon_world::{fluid::FluidKind, registry::BlockStateId};
use std::collections::HashMap;
struct Native<'g, 'a> {
    game: &'g mut Game<'a>,
    states: HashMap<&'static str, BlockStateId>,
    buds: HashMap<&'static str, BlockStateId>,
    sea: i32,
    writes: usize,
}
impl World for Native<'_, '_> {
    type State = BlockStateId;
    fn block(&mut self, [x, y, z]: Pos) -> BlockStateId {
        self.game.world.block(x, y, z)
    }
    fn state(&self, id: &'static str) -> BlockStateId {
        self.states[id]
    }
    fn bud_up(&self, id: &'static str) -> BlockStateId {
        self.buds[id]
    }
    fn is_block(&self, state: &BlockStateId, id: &'static str) -> bool {
        self.game.res.facts().get(*state).name.as_ref() == id
    }
    fn properties(&self, state: &BlockStateId) -> Properties {
        let facts = self.game.res.facts().get(*state);
        Properties {
            air: self.game.res.behaviours().is_air(*state),
            solid: facts.legacy_solid,
            replaceable: facts.replaceable,
            fluid_empty: self.game.res.behaviours().fluid(*state).kind() == FluidKind::Empty,
            replaceable_tag: self
                .game
                .items()
                .block_in_tag("minecraft:replaceable", &facts.name),
        }
    }
    fn set(&mut self, pos: Pos, state: BlockStateId) {
        self.game
            .native_set_block(self.game.dim.id(), pos, state, 3);
        self.writes += 1;
    }
    fn max_build_height(&self) -> i32 {
        self.game.world.top_y()
    }
    fn sea_level(&self) -> i32 {
        self.sea
    }
    fn copy_biome(&mut self, pos: Pos) -> CopyBiome {
        let [qx, qy, qz] = self
            .game
            .streaming
            .biome_zoom
            .quart_at_block(BlockPos::new(pos[0], pos[1], pos[2]));
        let [x, y, z] = [qx << 2, qy << 2, qz << 2];
        let chunk = &self.game.world.chunks[&(x >> 4, z >> 4)];
        let biome = &self.game.res.biomes().get(chunk.biome(x, y, z)).name;
        let tag = |name| self.game.res.mob_data().biome_has(name, biome);
        CopyBiome {
            badlands: tag("minecraft:is_badlands"),
            snowy: tag("c:is_snowy"),
            beach: tag("minecraft:is_beach"),
            sandy: tag("c:is_sandy"),
            plains: tag("c:is_plains"),
            forest: tag("minecraft:is_forest"),
        }
    }
    fn discard_items(&mut self, bounds: [f64; 6]) {
        let area = Aabb::new(
            Vec3::new(bounds[0], bounds[1], bounds[2]),
            Vec3::new(bounds[3], bounds[4], bounds[5]),
        );
        self.game
            .dropped
            .retain(|item| !item.bounds().intersects(area));
    }
    fn schedule_water(&mut self, pos: Pos) {
        let time = self.game.world.game_time;
        self.game.with_level(|_, _, ticks| {
            ticks.schedule_fluid(
                time,
                BlockPos::new(pos[0], pos[1], pos[2]),
                FluidKind::Water,
                0,
            );
        });
    }
    // This diagnostic caller aliases both streams to the level RNG, as the
    // source MeteoritePlacerItem does. Never eagerly sample unused rolls.
    fn next_int(&mut self, bound: i32) -> i32 {
        self.game
            .world
            .random
            .next_int_bound(bound)
            .expect("positive source bound")
    }
    fn next_float(&mut self) -> f32 {
        self.game.world.random.next_float()
    }
    fn next_double(&mut self) -> f64 {
        self.game.world.random.next_double()
    }
    fn level_float(&mut self) -> f32 {
        self.game.world.random.next_float()
    }
}
pub(super) fn register(game: &Game<'_>, commands: &mut Vec<super::native_server::NativeCommand>) {
    if game.native_on("ae2") {
        commands.push(super::native_server::NativeCommand::op(command::USAGE, 2));
    }
}
pub(super) fn perform(
    game: &mut Game<'_>,
    actor: &super::native_server::NativeServerPlayer,
    line: &str,
    ran: &mut bool,
) {
    let words: Vec<_> = line.split_whitespace().collect();
    if words.first() != Some(&"ae2meteorite") || !game.native_on("ae2") {
        return;
    }
    *ran = true;
    let result = (|| -> Result<usize, String> {
        if actor.permission < 2 || actor.dimension != game.dim.id() {
            return Err("The meteorite probe requires an operator in this dimension".into());
        }
        let request = command::parse(&words[1..]).map_err(str::to_owned)?;
        if request.settings.pos[1] < game.world.bottom_y() + 9
            || request.settings.pos[1] >= game.world.top_y() - 30
            || request.sea_level < game.world.bottom_y()
            || request.sea_level > game.world.top_y()
        {
            return Err(
                "Meteorite center or explicit sea level outside this level's build range".into(),
            );
        }
        let mut states = HashMap::new();
        for &id in placer::STATE_IDS {
            let state = game
                .pack
                .assets
                .registry
                .default_state(id)
                .ok_or_else(|| format!("Missing meteorite state {id}"))?;
            states.insert(id, state);
        }
        let mut buds = HashMap::new();
        for id in [
            "ae2:small_quartz_bud",
            "ae2:medium_quartz_bud",
            "ae2:large_quartz_bud",
        ] {
            let state = game
                .with_props(states[id], &[("facing", "up")])
                .ok_or_else(|| format!("Missing up-facing bud state {id}"))?;
            buds.insert(id, state);
        }
        for x in (request.bounds.min[0] >> 4)..=(request.bounds.max[0] >> 4) {
            for z in (request.bounds.min[2] >> 4)..=(request.bounds.max[2] >> 4) {
                if !game.world.chunks.contains_key(&(x, z)) {
                    return Err(format!("Meteorite slice includes unloaded chunk {x} {z}"));
                }
            }
        }
        let center = request.bounds.center();
        let [qx, _, qz] = game
            .streaming
            .biome_zoom
            .quart_at_block(BlockPos::new(center[0], center[1], center[2]));
        if !game.world.chunks.contains_key(&(qx >> 2, qz >> 2)) {
            return Err("The slice's zoomed center biome is not loaded".into());
        }
        let mut native = Native {
            game,
            states,
            buds,
            sea: request.sea_level,
            writes: 0,
        };
        placer::place(&mut native, request.settings, request.bounds);
        Ok(native.writes)
    })();
    game.native_feedback(Text::from(match result {
        Ok(writes) => format!("AE2 meteorite placement probe finished ({writes} writes)"),
        Err(error) => format!("AE2 meteorite probe refused: {error}"),
    }));
}
