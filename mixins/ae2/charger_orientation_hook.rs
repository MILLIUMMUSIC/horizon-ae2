// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_charger_orientation(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
) {
    crate::game::ae2_charger_orientation::placed(game, stack, pos);
}
