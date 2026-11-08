fn ae2_automation_insert(
    game: &mut crate::game::Game<'_>,
    view: &crate::game::machines::View,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
) -> core::ops::ControlFlow<Option<ItemStack>> {
    let Some((pos, index)) = view.locate(slot) else {
        return core::ops::ControlFlow::Continue(());
    };
    if game.native_block([pos.0, pos.1, pos.2]) == "ae2:inscriber" {
        crate::game::ae2_grid_glue::sync_inscribers(game);
        let limit = match game.world.block_entities.get(&pos) {
            Some(horizon_client::containers::BlockEntity::Mod(be))
                if be.id == "ae2:inscriber"
                    && crate::game::ae2_inscriber_glue::allow_insert(game, be, index, stack) =>
            {
                crate::game::ae2_inscriber_glue::input_capacity(be).min(
                    game.items()
                        .get(&stack.item)
                        .map_or(64, |i| i.max_stack)
                        .max(1),
                )
            }
            _ => return core::ops::ControlFlow::Break(Some(stack.clone())),
        };
        let Some(target) = game.slot_mut_at(view, slot) else {
            return core::ops::ControlFlow::Break(Some(stack.clone()));
        };
        let (rest, changed) =
            crate::game::ae2_inscriber_glue::insert_stack(target, stack.clone(), limit);
        if changed {
            game.view_changed(view);
        }
        return core::ops::ControlFlow::Break(rest);
    }
    if game.native_block([pos.0, pos.1, pos.2]) == "ae2:charger" {
        crate::game::ae2_grid_glue::sync_chargers(game);
        let usable = matches!(game.world.block_entities.get(&pos),
            Some(horizon_client::containers::BlockEntity::Mod(be))
                if be.id == "ae2:charger" && crate::game::ae2_charger_inventory::usable(be));
        let valid =
            index == 0 && usable && crate::game::ae2_charger_automation::allow_insert(game, stack);
        if !valid {
            return core::ops::ControlFlow::Break(Some(stack.clone()));
        }
        let Some(target) = game.slot_mut_at(view, slot) else {
            return core::ops::ControlFlow::Break(Some(stack.clone()));
        };
        let (rest, changed) = crate::game::ae2_automation::insert_one(target, stack.clone(), true);
        if changed {
            game.view_changed(view);
        }
        return core::ops::ControlFlow::Break(rest);
    }
    if !matches!(game.world.block_entities.get(&pos), Some(horizon_client::containers::BlockEntity::Mod(be)) if be.id == "ae2:drive")
    {
        return core::ops::ControlFlow::Continue(());
    }
    if index >= 10 {
        return core::ops::ControlFlow::Break(Some(stack.clone()));
    }
    let Some(target) = game.slot_mut_at(view, slot) else {
        return core::ops::ControlFlow::Break(Some(stack.clone()));
    };
    let (rest, changed) = crate::game::ae2_automation::insert(target, stack.clone());
    if changed {
        game.view_changed(view);
    }
    core::ops::ControlFlow::Break(rest)
}
