fn ae2_automation_take(
    game: &mut crate::game::Game<'_>,
    view: &crate::game::machines::View,
    slot: usize,
) -> core::ops::ControlFlow<Option<ItemStack>> {
    let Some((pos, index)) = view.locate(slot) else {
        return core::ops::ControlFlow::Continue(());
    };
    if game.native_block([pos.0, pos.1, pos.2]) == "ae2:inscriber" {
        crate::game::ae2_grid_glue::sync_inscribers(game);
        if !matches!(game.world.block_entities.get(&pos), Some(horizon_client::containers::BlockEntity::Mod(be))
            if be.id == "ae2:inscriber" && crate::game::ae2_inscriber_glue::allow_take(be, index))
        {
            return core::ops::ControlFlow::Break(None);
        }
        return core::ops::ControlFlow::Continue(());
    }
    if game.native_block([pos.0, pos.1, pos.2]) != "ae2:charger" {
        return core::ops::ControlFlow::Continue(());
    }
    crate::game::ae2_grid_glue::sync_chargers(game);
    let allowed = index == 0
        && matches!(game.world.block_entities.get(&pos),
        Some(horizon_client::containers::BlockEntity::Mod(be))
            if be.id == "ae2:charger" && crate::game::ae2_charger_inventory::usable(be)
            && be.items.first().and_then(Option::as_ref)
                .is_some_and(|stack| crate::game::ae2_charger_automation::allow_extract(game, stack)));
    if !allowed {
        return core::ops::ControlFlow::Break(None);
    }
    core::ops::ControlFlow::Continue(())
}
