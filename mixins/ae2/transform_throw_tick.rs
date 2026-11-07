// SPDX-License-Identifier: LGPL-3.0-or-later
// Item-entity tick, before ItemEntity lifetime discard.
fn ae2_fluix_before_item_tick(game: &mut crate::game::Game<'_>) {
    crate::game::ae2_transform_throw_world::before_item_tick(game);
}
