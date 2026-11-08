fn ae2_automation_changed(game: &mut crate::game::Game<'_>, pos: (i32, i32, i32)) {
    if game.native_block([pos.0, pos.1, pos.2]) == "ae2:inscriber"
        && let Some(horizon_client::containers::BlockEntity::Mod(be)) =
            game.world.block_entities.get_mut(&pos)
        && be.id == "ae2:inscriber"
    {
        crate::game::ae2_inscriber_glue::persist(be);
    }
    if game.native_block([pos.0, pos.1, pos.2]) == "ae2:charger"
        && let Some(horizon_client::containers::BlockEntity::Mod(be)) =
            game.world.block_entities.get_mut(&pos)
        && be.id == "ae2:charger"
    {
        crate::game::ae2_charger_inventory::persist(be);
    }
    if matches!(game.world.block_entities.get(&pos), Some(horizon_client::containers::BlockEntity::Mod(be)) if be.id == "ae2:drive")
    {
        crate::game::ae2_grid_glue::flush_drives(game);
    }
}
