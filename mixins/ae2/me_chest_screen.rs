// SPDX-License-Identifier: LGPL-3.0-or-later
// Layout for a menu that is already opening. `Game::use_hand` calls
// `native_right_click_block` first; when that sets `canceled`, it returns
// before `open_menu`, so this hook never runs and the client stays on playing.
fn ae2_me_chest_screen(
    block: &str,
    screen: &mut Option<crate::game::native_hooks::NativeScreen>,
) {
    crate::game::ae2_me_chest_glue::screen(block, screen);
}
