// CertusQuartzClusterBlock.getDrops: refuse a missing THIS_ENTITY before the
// native evaluator allocates/draws the loot sequence. Player contexts keep the
// original carried table, actual passed tool and normal sequence machinery.
fn ae2_quartz_loot_context(
    registry: &horizon_world::registry::BlockRegistry,
    state: horizon_world::registry::BlockStateId,
    has_entity: bool,
) -> core::ops::ControlFlow<Vec<ItemStack>> {
    if !has_entity
        && registry
            .state(state)
            .is_some_and(|block| crate::items::ae2_quartz::is_cluster(&block.name))
    {
        return core::ops::ControlFlow::Break(vec![]);
    }
    core::ops::ControlFlow::Continue(())
}
