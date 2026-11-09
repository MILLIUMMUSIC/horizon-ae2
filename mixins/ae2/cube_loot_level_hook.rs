fn ae2_cube_tag_loot(
    items: &crate::items::Items,
    registry: &horizon_world::registry::BlockRegistry,
    state: horizon_world::registry::BlockStateId,
    held: Option<&crate::items::ItemStack>,
    dynamic: &[crate::items::ItemStack],
    random: &mut impl horizon_core::random::RandomSource,
) -> core::ops::ControlFlow<Vec<ItemStack>> {
    match crate::items::ae2_cube_loot::drops(items, registry, state, held, dynamic, random, false) {
        Some(drops) => core::ops::ControlFlow::Break(drops),
        None => core::ops::ControlFlow::Continue(()),
    }
}
