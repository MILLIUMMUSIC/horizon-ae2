fn ae2_inventory_signal_override(
    containers: &crate::game::machines::Containers<'_>,
    view: &crate::game::machines::View,
) -> core::ops::ControlFlow<i32> {
    if let Some((pos, _)) = view.locate(0)
        && let Some(horizon_client::containers::BlockEntity::Mod(be)) =
            containers.world.block_entities.get(&pos)
        && be.id == "ae2:inscriber"
    {
        let saved = match be.nbt.get(&horizon_world::nbt::JavaString::from("inv")) {
            Some(tag) => match super::ae2_inventory_codec::read_slots(tag, 4) {
                Ok(items) => Some(items),
                Err(_) => return core::ops::ControlFlow::Break(0),
            },
            None => None,
        };
        let limit = super::ae2_inscriber_glue::input_capacity(be);
        let signal = super::ae2_inventory_signal::signal_with_limits(
            saved
                .as_deref()
                .unwrap_or(&be.items)
                .iter()
                .take(4)
                .map(|s| {
                    s.as_ref()
                        .filter(|s| s.count > 0 && &*s.item != "minecraft:air")
                        .map_or((0, limit), |s| {
                            (s.count, containers.max_stack(s).min(limit))
                        })
                }),
            4,
        );
        return core::ops::ControlFlow::Break(signal);
    }
    if let Some((pos, _)) = view.locate(0)
        && let Some(horizon_client::containers::BlockEntity::Mod(be)) =
            containers.world.block_entities.get(&pos)
        && let Some(signal) = super::ae2_inventory_signal_glue::signal(&be.id, &be.nbt, &be.items)
    {
        return core::ops::ControlFlow::Break(signal);
    }
    core::ops::ControlFlow::Continue(())
}
