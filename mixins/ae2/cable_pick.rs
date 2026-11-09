fn ae2_cable_pick(
    game: &crate::game::Game<'_>,
    pos: [i32; 3],
    include_data: bool,
) -> core::ops::ControlFlow<Option<ItemStack>> {
    if game.native_block(pos) != "ae2:cable_bus" {
        return core::ops::ControlFlow::Continue(());
    }
    let picked = (|| {
        let hit = game.pick().filter(|hit| hit.pos == pos)?;
        if !horizon_client_render::ae2_cable_view::selects_center(
            &game.world,
            &game.pack.assets.registry,
            pos,
            hit.point,
        ) {
            return None;
        }
        let BlockEntity::Mod(be) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))? else {
            return None;
        };
        if be.id != "ae2:cable_bus"
            || crate::game::ae2_cable_bus::SIDES
                .iter()
                .chain(crate::game::ae2_cable_bus::FACADES.iter())
                .any(|name| {
                    be.nbt
                        .contains_key(&horizon_world::nbt::JavaString::from(*name))
                })
        {
            return None;
        }
        let saved = crate::game::ae2_cable_bus::Saved::read(&be.nbt).ok()?;
        let cable = saved.center().ok()??;
        let mut stack = ItemStack::new(&cable.item()?, 1);
        // CableBusBlock.getCloneItemStack returns the bare part item. The
        // server's optional creative Ctrl data is a separate vanilla step.
        if include_data && game.creative {
            let entity = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))?;
            let patch = crate::game::ae2_cable_input::pick_components(pos, entity);
            if !patch.is_empty() {
                (stack.damage, stack.extra) =
                    item_data::decode_components(Some(&Tag::Compound(patch)));
            }
        }
        Some(stack)
    })();
    core::ops::ControlFlow::Break(picked)
}
