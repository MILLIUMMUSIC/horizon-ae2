fn ae2_meteorite_register(
    game: &crate::game::Game<'_>,
    commands: &mut Vec<crate::game::native_server::NativeCommand>,
) {
    crate::game::ae2_meteorite_glue::register(game, commands);
}
