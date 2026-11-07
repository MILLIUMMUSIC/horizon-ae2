// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_me_chest_click(
    game: &mut crate::game::Game<'_>,
    slot: Option<usize>,
    button: i32,
    input: horizon_client::menu::Input,
) {
    crate::game::ae2_me_chest_glue::before_click(game, slot, button, input);
}
