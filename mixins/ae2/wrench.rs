// SPDX-License-Identifier: LGPL-3.0-or-later
// Original WrenchHook.onPlayerUseBlock rotation: a main-hand item in the
// common wrench tag turns an AE2 orientable block around the clicked face.
// Shift-disassembly, permissions and the network tool's own UI remain open;
// this module deliberately performs no approximate substitute for them.
use super::{Game, ae2_glue, ae2_orientation};
use winit::keyboard::KeyCode;

const CHARGER: &str = "ae2:charger";
const CRANK: &str = "ae2:crank";
const DRIVE: &str = "ae2:drive";
const NETWORK_TOOL: &str = "ae2:network_tool";
const WRENCH_TAG: &str = "c:tools/wrench";

fn face_name(face: [i32; 3]) -> Option<&'static str> {
    Some(match face {
        [0, -1, 0] => "down",
        [0, 1, 0] => "up",
        [0, 0, -1] => "north",
        [0, 0, 1] => "south",
        [-1, 0, 0] => "west",
        [1, 0, 0] => "east",
        _ => return None,
    })
}

/// Whether the held main-hand stack can turn an AE2 block. The network tool is
/// in the common tag but is explicitly excluded by the source.
fn wrenching(game: &Game<'_>) -> bool {
    // Source `isInAlternateUseMode`: shift is the disassembly mode, not rotation.
    if game.spectator() || game.hand != horizon_client::first_person::Hand::Main {
        return false;
    }
    if game.keys.contains(&KeyCode::ShiftLeft) || game.keys.contains(&KeyCode::ShiftRight) {
        return false;
    }
    let Some(held) = game.held() else {
        return false;
    };
    if &*held.item == NETWORK_TOOL {
        return false;
    }
    game.items()
        .item_tags
        .get(WRENCH_TAG)
        .is_some_and(|tag| tag.contains(&*held.item))
}

/// Returns true when the click was a source orientation turn and must not reach
/// the block's own use. A turn that would not change the state returns false,
/// matching the source's `newState != state` fallthrough.
pub(super) fn rotate(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3]) -> bool {
    if !wrenching(game) {
        return false;
    }
    let block = game.native_block(pos);
    let Some(clicked) = face_name(face) else {
        return false;
    };
    let Some(facing) = game.native_block_property(pos, "facing") else {
        return false;
    };
    if block == CRANK {
        let Some(new_facing) = ae2_orientation::rotated(&facing, clicked) else {
            return false;
        };
        if new_facing == facing || ae2_glue::crankable_attached(game, pos, new_facing).is_none() {
            return false;
        }
        return game.native_set_block_property(pos, "facing", new_facing);
    }
    if !matches!(block.as_str(), CHARGER | DRIVE) {
        return false;
    }
    let Some(spin) = game
        .native_block_property(pos, "spin")
        .and_then(|value| value.parse::<u8>().ok())
    else {
        return false;
    };
    let Some((new_facing, new_spin)) = ae2_orientation::rotate_full(&facing, spin, clicked) else {
        return false;
    };
    if (new_facing, new_spin) == (facing.as_str(), spin) {
        return false;
    }
    set_full(game, pos, new_facing, new_spin)
}

pub(super) fn set_full(game: &mut Game<'_>, pos: [i32; 3], facing: &str, spin: u8) -> bool {
    let old = game.world.block(pos[0], pos[1], pos[2]);
    let spin = spin.to_string();
    let Some(state) = game.with_props(old, &[("facing", facing), ("spin", &spin)]) else {
        return false;
    };
    let dimension = game.dim.id().to_owned();
    game.native_set_block(&dimension, pos, state, 3)
}
