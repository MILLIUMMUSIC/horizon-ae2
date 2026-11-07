// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_me_chest_slot(
    game: &crate::game::Game<'_>,
    slot: usize,
    stack: &horizon_client::items::ItemStack,
) -> core::ops::ControlFlow<(bool, u32)> {
    match crate::game::ae2_me_chest_glue::slot_rule(game, slot, stack) {
        Some(rule) => core::ops::ControlFlow::Break(rule),
        None => core::ops::ControlFlow::Continue(()),
    }
}
