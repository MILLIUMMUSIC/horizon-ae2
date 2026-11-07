fn ae2_meteorite_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use horizon_core::random::RandomSource;
    use crate::game::ae2_meteorite::{
        self as meteorite, ColumnKind, CopyBiome, CraterType, FalloutMode, Site, SkyRoll,
    };
    let mut words = command.split_whitespace();
    if words.next() != Some("ae2meteorite") {
        return;
    }
    *ran = true;
    let Some((x, y, z, radius)) = (|| {
        let x: i32 = words.next()?.parse().ok()?;
        let y: i32 = words.next()?.parse().ok()?;
        let z: i32 = words.next()?.parse().ok()?;
        let radius: f64 = words.next()?.parse().ok()?;
        (radius.is_finite() && radius > 0.0).then_some((x, y, z, radius))
    })() else {
        return;
    };
    let mode = FalloutMode::Default;
    let kind = CraterType::Lava;
    let pure = true;
    let span = 24;
    let site = Site {
        x,
        y,
        z,
        radius,
        min_x: x.saturating_sub(span),
        max_x: x.saturating_add(span),
        min_z: z.saturating_sub(span),
        max_z: z.saturating_add(span),
    };
    let top = game.world.top_y() - 1;
    let max_y = (y.saturating_add(30)).min(top);
    let sea = if game.dim.id() == "minecraft:the_nether" {
        32
    } else {
        63
    };
    let specified = meteorite::copy_block(CopyBiome::default());
    let filler = meteorite::filler(kind);
    let crater_sq = meteorite::crater_squared(radius);
    let budget = meteorite::fall_budget(radius);

    let read = |game: &crate::game::Game<'_>, px: i32, py: i32, pz: i32| {
        let state = game.world.block(px, py, pz);
        let facts = game.res.facts().get(state);
        (
            String::from(facts.name.as_ref()),
            facts.legacy_solid,
            facts.replaceable,
            facts.fluid_level == 0,
        )
    };
    let empty = |name: &str| {
        name == "minecraft:air" || name == "minecraft:cave_air" || name == "minecraft:void_air"
    };
    let write = |game: &mut crate::game::Game<'_>,
                 px: i32,
                 py: i32,
                 pz: i32,
                 block: &str,
                 facing_up: bool| {
        let pos = [px, py, pz];
        if game.native_block(pos) == "minecraft:bedrock" {
            return;
        }
        if !facing_up && game.native_block(pos) == block {
            return;
        }
        let Some(mut state) = game.pack.assets.registry.default_state(block) else {
            return;
        };
        if facing_up {
            if let Some(next) =
                game.with_props(state, &[("facing", "up"), ("waterlogged", "false")])
            {
                state = next;
            }
        }
        game.native_set_block(game.dim.id(), pos, state, 3);
    };

    let crater = meteorite::place_crater(site, mode, kind, max_y, |px, py, pz| {
        read(game, px, py, pz).1
    });
    for block in crater {
        write(game, block.x, block.y, block.z, block.block, false);
    }
    let shell = meteorite::place_sky_stone_rolls(x, y, z, radius, |roll, slot| match roll {
        SkyRoll::Int(bound) => {
            slot.int = game.world.random.next_int_bound(bound).unwrap_or(0);
        }
        SkyRoll::Float => slot.float = game.world.random.next_float(),
    });
    for block in shell {
        write(
            game,
            block.x,
            block.y,
            block.z,
            block.block,
            block.facing_up,
        );
    }

    let mut falls = 0.0;
    for i in site.min_x..=site.max_x {
        for k in site.min_z..=site.max_z {
            let mut j = y.saturating_sub(9);
            let j_end = y.saturating_add(30).min(top);
            while j < j_end {
                let (here, _, replaceable, _) = read(game, i, j, k);
                if pure && here == filler {
                    j += 1;
                    continue;
                }
                let (above, _, _, _) = read(game, i, j + 1, k);
                if replaceable {
                    if !empty(&above) {
                        write(game, i, j, k, &above, false);
                    } else if falls < budget {
                        let dx = f64::from(i.wrapping_sub(x));
                        let dy = f64::from(j.wrapping_sub(y));
                        let dz = f64::from(k.wrapping_sub(z));
                        let dist = dx * dx + dy * dy + dz * dz;
                        let (below, _, below_replaceable, _) = read(game, i, j - 1, k);
                        if !below_replaceable {
                            let extra = game.world.random.next_double() * 0.6;
                            let height = meteorite::fall_height(crater_sq, extra, dist);
                            if !empty(&below)
                                && height > 0.0
                                && game.world.random.next_double() > 0.6
                            {
                                falls += 1.0;
                                if let Some(block) = meteorite::fall_block(
                                    mode,
                                    specified,
                                    game.world.random.next_float(),
                                ) {
                                    write(game, i, j, k, block, false);
                                }
                            }
                        }
                    }
                } else if empty(&above) && game.world.random.next_double() > 0.4 {
                    let dx = i.wrapping_sub(x);
                    let dy = j.wrapping_sub(y);
                    let dz = k.wrapping_sub(z);
                    if meteorite::inset_reaches(dx, dy, dz, crater_sq) {
                        if let Some(block) =
                            meteorite::inset_block(mode, specified, game.world.random.next_float())
                        {
                            write(game, i, j, k, block, false);
                        }
                    }
                }
                j += 1;
            }
        }
    }

    let lake = meteorite::place_lake(site, mode, sea, |px, py, pz| {
        game.native_block([px, py, pz]) == "minecraft:air"
    });
    for block in lake.water {
        write(game, block.x, block.y, block.z, block.block, false);
    }
    for column in lake.slopes {
        let mut py = column.y;
        for _ in 0..20 {
            let (name, _, replaceable, fluid_empty) = read(game, column.x, py, column.z);
            let open = meteorite::column_kind(empty(&name), fluid_empty, replaceable, replaceable);
            if open == ColumnKind::Stop {
                break;
            }
            let magma = game.world.random.next_float();
            let fall = game.world.random.next_float();
            if let Some(block) = meteorite::enclosing_block(kind, magma, mode, specified, fall) {
                write(game, column.x, py, column.z, block, false);
            }
            py -= 1;
        }
    }
}
