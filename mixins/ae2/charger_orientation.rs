// SPDX-License-Identifier: LGPL-3.0-or-later
// Applies the source full-orientation placement to a Charger or Drive that the
// native item placement already put at `pos`. The engine's dispenser template
// supplies the facing for most looks, but the source thresholds the vertical
// placement at 65 degrees and adds the spin around the facing axis.
use super::{Game, ae2_orientation};
use horizon_client::items::ItemStack;

pub(super) fn placed(game: &mut Game<'_>, _stack: &ItemStack, pos: [i32; 3]) {
    let block = game.native_block(pos);
    if !matches!(block.as_str(), "ae2:charger" | "ae2:drive") {
        return;
    }
    let Some(player) = game.native_player() else {
        return;
    };
    let (facing, spin) = ae2_orientation::placement(player.yaw, player.pitch);
    let _ = super::ae2_wrench::set_full(game, pos, facing, spin);
}
