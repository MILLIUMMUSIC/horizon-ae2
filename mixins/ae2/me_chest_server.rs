// SPDX-License-Identifier: LGPL-3.0-or-later
fn ae2_me_chest_server(
    game: &mut crate::game::Game<'_>,
    host: &mut crate::game::lan::LanHost,
    conn: u32,
    packet: &horizon_protocol::packets::play::Serverbound,
) {
    let horizon_protocol::packets::play::Serverbound::ContainerClick(click) = packet else {
        return;
    };
    crate::game::ae2_me_chest_glue::server_click(game, host, conn, click);
}
