// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_right_click(game: &mut crate::game::Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if game
        .held()
        .is_some_and(|stack| crate::game::ae2_cable::Cable::from_item(&stack.item).is_some())
    {
        let block = game.native_block(pos);
        if block == "ae2:cable_bus"
            || matches!(
                block.as_str(),
                "ae2:controller"
                    | "ae2:energy_cell"
                    | "ae2:dense_energy_cell"
                    | "ae2:creative_energy_cell"
            )
            || game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft)
        {
            return;
        }
    }
    // An owned chest would be canceled in ae2_glue::right_click before
    // use_hand reaches open_menu. Leave the click alone so the chest menu opens.
    if game.native_block(pos) == "ae2:chest" {
        return;
    }
    crate::game::ae2_glue::right_click(game, pos, canceled);
}
