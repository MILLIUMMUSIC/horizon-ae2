// SPDX-License-Identifier: LGPL-3.0-or-later
// Retain the pre-placement fluid only for the current native hand operation.
// The ordinary BlockItem/adventure/obstruction path still performs admission.
use super::{Game, Hit};
use horizon_world::fluid::{FluidKind, FluidState};
use std::cell::RefCell;
struct Frame {
    game: usize,
    water: [([i32; 3], bool); 2],
}
thread_local! { static FRAMES: RefCell<Vec<Frame>> = const { RefCell::new(Vec::new()) }; }
pub(super) struct Scope;
impl Drop for Scope {
    fn drop(&mut self) {
        FRAMES.with(|frames| {
            frames.borrow_mut().pop();
        });
    }
}
pub(super) fn begin(game: &Game<'_>, hit: &Hit) -> Option<Scope> {
    let address = std::ptr::from_ref(game).addr();
    FRAMES.with(|frames| {
        let mut frames = frames.borrow_mut();
        if frames.iter().any(|f| f.game == address) {
            return None;
        }
        let adjacent = std::array::from_fn(|i| hit.pos[i].wrapping_add(hit.face[i]));
        let water = [hit.pos, adjacent].map(|pos| {
            let state = game.world.block(pos[0], pos[1], pos[2]);
            let water = game
                .pack
                .assets
                .registry
                .state(state)
                .and_then(|s| FluidState::from_block(s).ok())
                .is_some_and(|f| f.kind() == FluidKind::Water);
            (pos, water)
        });
        frames.push(Frame {
            game: address,
            water,
        });
        Some(Scope)
    })
}
pub(super) fn water(game: &Game<'_>, pos: [i32; 3]) -> Option<bool> {
    let address = std::ptr::from_ref(game).addr();
    FRAMES.with(|frames| {
        frames
            .borrow()
            .last()
            .filter(|f| f.game == address)
            .and_then(|f| {
                f.water
                    .iter()
                    .find(|(p, _)| *p == pos)
                    .map(|(_, water)| *water)
            })
    })
}
