fn ae2_survival_quartz(
    level: &mut crate::game::GameLevel<'_>,
    ticks: &mut horizon_world::level::Ticks,
    pos: horizon_core::position::BlockPos,
    state: horizon_world::registry::BlockStateId,
    handled: &mut bool,
) {
    use crate::game::ae2_quartz as quartz;
    let name = level.facts.get(state).name.clone();
    let Some(host) = quartz::host(&name) else {
        return;
    };
    *handled = true;
    if horizon_core::random::RandomSource::next_int_bound(
        &mut level.world.random,
        quartz::GROWTH_CHANCE,
    )
    .unwrap_or(0)
        != 0
    {
        return;
    }
    const FACES: [([i32; 3], &str); 6] = [
        ([0, -1, 0], "down"),
        ([0, 1, 0], "up"),
        ([0, 0, -1], "north"),
        ([0, 0, 1], "south"),
        ([-1, 0, 0], "west"),
        ([1, 0, 0], "east"),
    ];
    let index = horizon_core::random::RandomSource::next_int_bound(
        &mut level.world.random,
        FACES.len() as i32,
    )
    .unwrap_or(0) as usize;
    let (offset, direction) = FACES[index];
    let at = pos.offset(offset[0], offset[1], offset[2]);
    let target_state = level.block_state(at);
    let target_name = level.facts.get(target_state).name.clone();
    let facing = level.registry.state(target_state).and_then(|decoded| {
        decoded.properties.get("facing").map(|value| value.clone())
    });
    let water_level = level.registry.state(target_state).and_then(|decoded| {
        decoded.properties.get("level").map(|value| value.clone())
    });
    let water = target_name.as_ref() == "minecraft:water"
        && water_level.as_deref().is_none_or(|value| value == "0");
    let decay = horizon_core::random::RandomSource::next_int_bound(
        &mut level.world.random,
        quartz::DECAY_CHANCE,
    )
    .unwrap_or(1)
        == 0;
    let growth = quartz::grow(
        host,
        quartz::target(target_name.as_ref(), facing.as_deref(), direction, water),
        decay,
    );
    let place = |level: &mut crate::game::GameLevel<'_>,
                 ticks: &mut horizon_world::level::Ticks,
                 pos: horizon_core::position::BlockPos,
                 block: &str,
                 facing: Option<(&str, bool)>| {
        let Some(mut state) = level.registry.default_state(block) else {
            return;
        };
        if let Some((direction, waterlogged)) = facing {
            let water = if waterlogged { "true" } else { "false" };
            if let Some(next) = with_property(level.registry, state, "facing", direction) {
                state = next;
            }
            if let Some(next) = with_property(level.registry, state, "waterlogged", water) {
                state = next;
            }
        }
        let behaviours = level.behaviours;
        let _ = behaviours.set_block(level, ticks, pos, state, flags::UPDATE_ALL);
    };
    if let Some(cluster) = growth.cluster {
        place(level, ticks, at, cluster, Some((direction, water)));
    }
    if let Some(next) = growth.host {
        place(level, ticks, pos, next, None);
    }
}
