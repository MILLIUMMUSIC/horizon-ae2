// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Ludovic Marie and the Horizon contributors.
// Adapted pinned Game::tick_items body; AE2 additions are marked below.
// See LICENSES/Horizon-MIT. Original file SHA-256 is in compatibility.json.
// The source-body audit removes only the marked additions and verifies the
// remaining body against the pinned engine. Original checkouts are unchanged.
use super::{Game, ae2_transform_throw_world as transform, pickups};
use horizon_client::items::ItemStack;
use horizon_core::{geometry::Vec3, random::RandomSource};

pub(super) fn tick(game: &mut Game<'_>) {
    // AE2_BEGIN initial
    let recipes = transform::recipes(game);
    let mut clocks = transform::begin(game);
    // AE2_END initial
    let mut items = std::mem::take(&mut game.dropped);
    // ServerLevel.tick: only entities in entity-ticking chunks tick.
    let ticking = game.entity_ticking_area();
    let mut removed = vec![false; items.len()];
    let mut cells = horizon_client::entities::ItemCells::new(&items);
    let catalog = world_items!(game);
    let max_stack = |stack: &ItemStack| catalog.get(&stack.item).map_or(64, |i| i.max_stack).max(1);
    // A LAN or server guest only shows the host's items.
    let merging = game.remote.is_none();
    for i in 0..items.len() {
        if removed[i] {
            continue;
        }
        let item = &mut items[i];
        if !ticking(
            item.position.x.floor() as i32,
            item.position.z.floor() as i32,
        ) {
            continue;
        }
        // ItemEntity.tick's noPhysics (servers only): out of a block
        // placed on it, towards the closest open side (#1252).
        let no_physics = merging && item.inside_block(&game.world, game.collision);
        if no_physics {
            let speed = game.world.random.next_float() * 0.2 + 0.1;
            item.move_towards_closest_space(&game.world, game.res.facts(), speed);
        }
        let alive = item.tick(
            &game.world,
            game.pack.assets.registry,
            game.res.facts(),
            game.collision,
            no_physics,
            &game.res.motion().fluids,
        );
        cells.moved(i, item.previous, item.position);
        // ItemEntity.tick: merge every 2 ticks while moving between
        // blocks, every 40 otherwise, before the lifetime check.
        let block = |v: Vec3| (v.x.floor(), v.y.floor(), v.z.floor());
        let rate = if block(item.previous) == block(item.position) {
            40
        } else {
            2
        };
        if merging
            && (alive || item.age == 6000 && item.position.y > -128.0)
            && item.ticks.is_multiple_of(rate)
        {
            horizon_client::entities::merge_with_neighbours(
                &mut items,
                &mut removed,
                &mut cells,
                i,
                &max_stack,
            );
        }
        let item = &mut items[i];
        if !alive || removed[i] {
            removed[i] = true;
            cells.remove(i, item.position);
            continue;
        }
        let environment = item.environment_tick(
            &game.world,
            game.res.facts(),
            game.pack.assets.registry,
            world_items!(game),
        );
        // shortcut: age is the fresh-drop clock; use entity ticks for saved/frozen-age phase parity.
        if environment.should_play_lava_hurt_sound(item.age) {
            let pitch = 2.0 + game.world.random.next_float() * 0.4;
            game.audio.play(
                "minecraft:entity.generic.burn",
                Some([item.position.x, item.position.y, item.position.z]),
                0.4,
                pitch,
            );
        }
        if !environment.alive {
            removed[i] = true;
            cells.remove(i, item.position);
        }
        // AE2_BEGIN entity-return
        if environment.alive {
            transform::after_tick(
                &mut game.world,
                game.pack.assets.registry,
                &mut items,
                &mut removed,
                &mut cells,
                i,
                &recipes,
                &mut clocks,
                |stack, position, velocity| {
                    game.next_entity += 1;
                    let bob = game.particles.random().next_float() * std::f32::consts::PI * 2.0;
                    transform::fresh(stack, position, velocity, game.next_entity, bob)
                },
            );
        }
        // AE2_END entity-return
    }
    // AE2_BEGIN outputs
    transform::finish(game, clocks, &items, &removed);
    // AE2_END outputs
    let mut removed = removed.into_iter();
    items.retain(|_| !removed.next().unwrap_or(false));
    let Some(p) = game.player.as_ref() else {
        game.dropped = items;
        return;
    };
    if p.health.dead_or_dying() || game.spectator() {
        game.dropped = items;
        return;
    }
    let w = f64::from(p.dimensions.width) / 2.0;
    let (min, max) = (
        [
            p.position.x - w - 1.0,
            p.position.y - 0.5,
            p.position.z - w - 1.0,
        ],
        [
            p.position.x + w + 1.0,
            p.position.y + f64::from(p.dimensions.height) + 0.5,
            p.position.z + w + 1.0,
        ],
    );
    let feet = [p.position.x, p.position.y, p.position.z];
    let own = game.level.ensure_player_uuid();
    let mut kept = Vec::with_capacity(items.len());
    for mut item in items {
        let b = item.bounds();
        let touching = b.min_position().x < max[0]
            && b.max_position().x > min[0]
            && b.min_position().y < max[1]
            && b.max_position().y > min[1]
            && b.min_position().z < max[2]
            && b.max_position().z > min[2];
        // ItemEntity.playerTouch: no pickup delay, and the target (if any) is this player.
        if touching && item.pickup_delay == 0 && item.target.is_none_or(|t| t == own) {
            if game.native_pickup_refused(&item.stack) {
                item.pickup_delay = 10;
                kept.push(item);
                continue;
            }
            let before = item.stack.count;
            let kind = item.stack.item.clone();
            let thrower = item.thrower;
            match game.give(item.stack.clone()) {
                None => {
                    game.stats.add("picked_up", &kind, before as i32);
                    if let Some(thrower) = thrower {
                        game.trigger_thrown_picked_up_by_player(&item.stack, thrower);
                    }
                    game.add_pickup(
                        pickups::Taken::Item(Box::new(item)),
                        pickups::Collector::Player,
                    );
                    let pitch = ((game.world.random.next_float() - game.world.random.next_float())
                        * 0.7
                        + 1.0)
                        * 2.0;
                    game.audio
                        .play("minecraft:entity.item.pickup", Some(feet), 0.2, pitch);
                    continue;
                }
                Some(rest) => {
                    if rest.count < before {
                        game.stats
                            .add("picked_up", &kind, (before - rest.count) as i32);
                        if let Some(thrower) = thrower {
                            game.trigger_thrown_picked_up_by_player(&rest, thrower);
                        }
                        game.add_pickup(
                            pickups::Taken::Item(Box::new(item.clone())),
                            pickups::Collector::Player,
                        );
                        // handleTakeItemEntity: the random pitch of a full pickup.
                        let pitch = ((game.world.random.next_float()
                            - game.world.random.next_float())
                            * 0.7
                            + 1.0)
                            * 2.0;
                        game.audio
                            .play("minecraft:entity.item.pickup", Some(feet), 0.2, pitch);
                    }
                    item.stack = rest;
                }
            }
        }
        kept.push(item);
    }
    game.dropped = kept;
}
