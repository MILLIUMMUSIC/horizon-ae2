fn ae2_meteorite_command(
    game: &mut crate::game::Game<'_>,
    player: &crate::game::native_server::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    crate::game::ae2_meteorite_glue::perform(game, player, command, ran);
}
