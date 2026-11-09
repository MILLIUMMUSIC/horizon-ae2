// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_terminal_use(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    canceled: &mut bool,
) {
    let Some(side) = crate::game::ae2_terminal_screen::face_name(face) else {
        return;
    };
    if game.native_block(pos) != crate::game::ae2_cable_bus::BLOCK {
        return;
    }
    let held = game.held().map(|stack| stack.item.to_string());
    let stored = game.native_with_mod_block_entity(pos, |be| {
        if be.id != crate::game::ae2_cable_bus::BLOCK {
            return None;
        }
        let key = horizon_world::nbt::JavaString::from(side);
        match be.nbt.get(&key) {
            Some(horizon_world::nbt::Tag::Compound(part)) => {
                let id = match part.get(&horizon_world::nbt::JavaString::from("id")) {
                    Some(horizon_world::nbt::Tag::String(id)) => Some(id.to_string_lossy()),
                    _ => None,
                };
                let grid_node = matches!(
                    part.get(&horizon_world::nbt::JavaString::from("gn")),
                    Some(horizon_world::nbt::Tag::Compound(_))
                );
                Some((id, grid_node, true))
            }
            Some(_) => Some((None, false, true)),
            None => Some((None, false, false)),
        }
    });
    let Some(Some((id, grid_node, present))) = stored else {
        return;
    };
    if crate::game::ae2_terminal_screen::places(held.as_deref(), side, present) {
        let added = game
            .native_with_mod_block_entity(pos, |be| {
                if be.id != crate::game::ae2_cable_bus::BLOCK {
                    return false;
                }
                let key = horizon_world::nbt::JavaString::from(side);
                if be.nbt.contains_key(&key) {
                    return false;
                }
                let mut part = horizon_world::nbt::Compound::new();
                part.insert(
                    horizon_world::nbt::JavaString::from("id"),
                    horizon_world::nbt::Tag::String("ae2:terminal".into()),
                );
                part.insert(
                    horizon_world::nbt::JavaString::from("gn"),
                    horizon_world::nbt::Tag::compound([("p", horizon_world::nbt::Tag::Int(-1))]),
                );
                be.nbt.insert(key, horizon_world::nbt::Tag::Compound(part));
                true
            })
            .unwrap_or(false);
        if added {
            crate::game::ae2_grid_glue::node_placed(game.dim.id(), pos);
            crate::game::ae2_grid_glue::cables_changed(game);
            crate::game::ae2_terminal_screen::note_placed(game.ticks);
            game.consume_held();
            *canceled = true;
        }
        return;
    }
    let id = id.unwrap_or_default();
    let parts = if present {
        vec![(
            side,
            crate::game::ae2_terminal_screen::Part {
                id: id.as_str(),
                grid_node,
            },
        )]
    } else {
        Vec::new()
    };
    let Some(menu) = crate::game::ae2_terminal_screen::open(held.is_none(), side, &parts) else {
        return;
    };
    if crate::game::ae2_terminal_screen::placed_this_click(game.ticks) {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    crate::game::ae2_terminal_screen::request(player.uuid, menu);
    *canceled = true;
}
