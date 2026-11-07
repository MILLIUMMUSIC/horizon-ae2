// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_terminal_menu(game: &mut crate::game::Game<'_>, dimension: &str) {
    let _ = dimension;
    let Some(pending) = crate::game::ae2_terminal_screen::take() else {
        return;
    };
    let title = horizon_client::text::Text::translate(pending.title_key, &[]);
    let _ = game.native_open_menu(pending.uuid, pending.rows, title, Vec::new());
}
