// SPDX-License-Identifier: LGPL-3.0-or-later
// In-world fluix throw. Runs at the start of item-entity ticking, before a
// finished lifetime removes the entity. A missing recipe circumstance is the
// minecraft:water fluid tag, matched on the fluid at the trigger's box center.

use super::Game;
use super::ae2_transform_throw::{self as rules, ThrownItem};
use horizon_client::items::ItemStack;
use horizon_core::random::RandomSource;
use horizon_world::fluid::{FluidKind, FluidState};
use std::collections::BTreeMap;
use std::sync::Mutex;

static CLOCKS: Mutex<BTreeMap<u32, u32>> = Mutex::new(BTreeMap::new());

pub(super) fn before_item_tick(game: &mut Game<'_>) {
    if game.remote.is_some() {
        return;
    }
    let live: Vec<u32> = game.dropped.iter().map(|entity| entity.id).collect();
    let mut clocks = CLOCKS.lock().expect("fluix throw clocks");
    clocks.retain(|id, _| live.contains(id));
    let mut ready = Vec::new();
    for entity in &game.dropped {
        if !game.world.loaded(
            entity.position.x.floor() as i32,
            entity.position.z.floor() as i32,
        ) {
            continue;
        }
        let ticks = clocks.entry(entity.id).or_insert(0);
        if water_at(game, entity) {
            *ticks = ticks.saturating_add(1);
        } else {
            *ticks = 0;
        }
        if *ticks > rules::FLUID_TICKS_BEFORE_TRY
            && entity.stack.count > 0
            && entity.stack.item.as_ref() == rules::CHARGED_CERTUS
        {
            ready.push(entity.id);
        }
    }
    drop(clocks);

    for id in ready {
        let Some(index) = game.dropped.iter().position(|entity| entity.id == id) else {
            continue;
        };
        if !transform_at(game, index)
            && let Ok(mut clocks) = CLOCKS.lock()
        {
            if let Some(ticks) = clocks.get_mut(&id) {
                *ticks = 0;
            }
        }
    }
}

fn water_at(game: &Game<'_>, entity: &horizon_client::entities::ItemEntity) -> bool {
    let bounds = entity.bounds();
    let sample_y = (bounds.min_position().y + bounds.max_position().y) / 2.0;
    let pos = [
        entity.position.x.floor() as i32,
        sample_y.floor() as i32,
        entity.position.z.floor() as i32,
    ];
    let state = game.world.block(pos[0], pos[1], pos[2]);
    game.pack
        .assets
        .registry
        .state(state)
        .and_then(|block| FluidState::from_block(block).ok())
        .is_some_and(|fluid| matches!(fluid.kind(), FluidKind::Water | FluidKind::FlowingWater))
}

fn transform_at(game: &mut Game<'_>, index: usize) -> bool {
    let origin = game.dropped[index].position;
    let near: Vec<usize> = (0..game.dropped.len())
        .filter(|&other| {
            game.dropped[other].stack.count > 0 && intersects_region(origin, &game.dropped[other])
        })
        .collect();
    let Some(trigger) = near.iter().position(|&other| other == index) else {
        return false;
    };
    let wet = water_at(game, &game.dropped[index]);
    let names: Vec<String> = near
        .iter()
        .map(|&other| game.dropped[other].stack.item.to_string())
        .collect();
    let mut thrown: Vec<ThrownItem<'_>> = near
        .iter()
        .enumerate()
        .map(|(slot, &other)| ThrownItem {
            item: &names[slot],
            count: game.dropped[other].stack.count,
            alive: true,
        })
        .collect();
    let tags: &[&str] = if wet { &[rules::WATER_TAG] } else { &[] };
    let Some(product) = rules::try_fluix_throw(&mut thrown, trigger, tags) else {
        return false;
    };
    for (slot, item) in thrown.iter().enumerate() {
        game.dropped[near[slot]].stack.count = item.count;
    }
    let mut dead: Vec<usize> = near
        .iter()
        .copied()
        .filter(|&other| game.dropped[other].stack.count == 0)
        .collect();
    dead.sort_unstable();
    dead.dedup();
    for other in dead.into_iter().rev() {
        game.dropped.remove(other);
    }
    let random = &mut game.world.random;
    let x = origin.x.floor() + 0.25 + random.next_double() * 0.5;
    let y = origin.y.floor() + 0.25 + random.next_double() * 0.5;
    let z = origin.z.floor() + 0.25 + random.next_double() * 0.5;
    let velocity = [
        random.next_double() * 0.25 - 0.125,
        random.next_double() * 0.25 - 0.125,
        random.next_double() * 0.25 - 0.125,
    ];
    game.spawn_item(
        ItemStack::new(product.item, product.count),
        [x, y, z],
        velocity,
        10,
    );
    true
}

fn intersects_region(
    origin: horizon_core::geometry::Vec3,
    entity: &horizon_client::entities::ItemEntity,
) -> bool {
    let bounds = entity.bounds();
    let min = bounds.min_position();
    let max = bounds.max_position();
    min.x < origin.x + 1.0
        && max.x > origin.x - 1.0
        && min.y < origin.y + 1.0
        && max.y > origin.y - 1.0
        && min.z < origin.z + 1.0
        && max.z > origin.z - 1.0
}
