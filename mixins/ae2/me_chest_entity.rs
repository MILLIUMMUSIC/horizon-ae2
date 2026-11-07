// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_me_chest_entity(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::ae2_me_chest_glue::entity(block, entity);
}
