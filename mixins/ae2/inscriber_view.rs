fn ae2_inscriber_view(
    containers: &crate::game::machines::Containers<'_>,
    pos: (i32, i32, i32),
) -> core::ops::ControlFlow<Option<View>> {
    let state = containers.world.block(pos.0, pos.1, pos.2);
    if containers.name(state) != "ae2:inscriber" {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(horizon_client::containers::BlockEntity::Mod(be)) =
        containers.world.block_entities.get(&pos)
    else {
        return core::ops::ControlFlow::Break(None);
    };
    if be.id != "ae2:inscriber" {
        return core::ops::ControlFlow::Break(None);
    }
    let facing = containers.prop(state, "facing").unwrap_or("north");
    let spin = containers
        .prop(state, "spin")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let rules = crate::game::ae2_inscriber_glue::container(be, facing, spin);
    core::ops::ControlFlow::Break(Some(View {
        parts: vec![(pos, 4)],
        kind: Kind::Mod,
        container: Some(std::sync::Arc::new(rules)),
    }))
}
