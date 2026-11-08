// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_fluid_item_phase(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<()> {
    if game.remote.is_some() {
        return std::ops::ControlFlow::Continue(());
    }
    crate::game::ae2_item_tick::tick(game);
    std::ops::ControlFlow::Break(())
}
