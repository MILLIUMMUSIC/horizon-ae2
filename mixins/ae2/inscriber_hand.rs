fn ae2_inscriber_hand(
    game: &mut crate::game::Game<'_>,
    hit: Option<&crate::game::Hit>,
) -> core::ops::ControlFlow<Result<bool>> {
    let Some(hit) = hit else {
        return core::ops::ControlFlow::Continue(());
    };
    if !game
        .held()
        .is_some_and(|stack| &*stack.item == "ae2:inscriber")
    {
        return core::ops::ControlFlow::Continue(());
    }
    let Some(_scope) = crate::game::ae2_inscriber_placement::begin(game, hit) else {
        return core::ops::ControlFlow::Continue(());
    };
    core::ops::ControlFlow::Break(game.use_hand(Some(hit)))
}
