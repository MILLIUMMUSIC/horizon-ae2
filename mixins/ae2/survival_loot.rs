fn ae2_survival_loot(
    game: &mut crate::game::Game<'_>,
    state: horizon_world::registry::BlockStateId,
) -> core::ops::ControlFlow<Vec<ItemStack>> {
    let name = game.res.facts().get(state).name.clone();
    if matches!(
        name.as_ref(),
        "ae2:quartz_cluster"
            | "ae2:small_quartz_bud"
            | "ae2:medium_quartz_bud"
            | "ae2:large_quartz_bud"
    ) {
        let silk = game
            .held()
            .is_some_and(|stack| stack.enchantment_level("minecraft:silk_touch") > 0);
        let fortune = game
            .held()
            .map_or(0, |stack| stack.enchantment_level("minecraft:fortune"));
        // uniform_bonus_count, bonusMultiplier 1: nextInt(fortuneLevel + 1).
        // Fortune 0 is nextInt(1), which is always 0. Silk Touch skips it.
        let roll = if silk || fortune == 0 {
            0
        } else {
            let bound = i32::try_from(fortune).unwrap_or(i32::MAX).saturating_add(1);
            horizon_core::random::RandomSource::next_int_bound(&mut game.world.random, bound)
                .unwrap_or(0) as u32
        };
        let drops = crate::game::ae2_quartz::drops(name.as_ref(), true, silk, roll);
        return core::ops::ControlFlow::Break(
            drops
                .into_iter()
                .map(|drop| horizon_client::items::ItemStack::new(drop.item, drop.count))
                .collect(),
        );
    }
    core::ops::ControlFlow::Continue(())
}
