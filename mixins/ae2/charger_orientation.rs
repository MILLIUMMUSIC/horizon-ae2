// SPDX-License-Identifier: LGPL-3.0-or-later
// Applies the source full-orientation placement to a Charger or Drive that the
// native item placement already put at `pos`. The engine's dispenser template
// supplies the facing for most looks, but the source thresholds the vertical
// placement at 65 degrees and adds the spin around the facing axis.
use super::{Game, ae2_orientation};
use horizon_client::items::ItemStack;

pub(super) fn placed(game: &mut Game<'_>, stack: &ItemStack, pos: [i32; 3]) {
    let block = game.native_block(pos);
    if !matches!(
        block.as_str(),
        "ae2:charger" | "ae2:drive" | "ae2:inscriber"
    ) {
        return;
    }
    let Some(player) = game.native_player() else {
        return;
    };
    let (facing, spin) = ae2_orientation::placement(player.yaw, player.pitch);
    let old = game.world.block(pos[0], pos[1], pos[2]);
    let spin = spin.to_string();
    let Some(mut state) = game.with_props(old, &[("facing", facing), ("spin", &spin)]) else {
        return;
    };
    if block == "ae2:inscriber"
        && let Some(water) = super::ae2_inscriber_placement::water(game, pos)
    {
        state = game
            .with_props(
                state,
                &[("waterlogged", if water { "true" } else { "false" })],
            )
            .unwrap_or(state);
    }
    // BlockItem applies its BLOCK_STATE component after placement orientation.
    // Keep that ordering instead of replacing explicit item properties here.
    if let Some(horizon_world::nbt::Tag::Compound(properties)) =
        stack.data().and_then(|d| d.other("minecraft:block_state"))
    {
        for (key, value) in properties {
            if let horizon_world::nbt::Tag::String(value) = value {
                state = game
                    .with_props(
                        state,
                        &[(
                            key.to_string_lossy().as_str(),
                            value.to_string_lossy().as_str(),
                        )],
                    )
                    .unwrap_or(state);
            }
        }
    }
    if state != old {
        game.native_set_block(game.dim.id(), pos, state, 3);
    }
}
