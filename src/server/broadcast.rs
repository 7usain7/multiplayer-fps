use crate::protocol::{ProjectileState, ServerPacket};
use crate::server::net::ServerNetworkSocket;
use crate::server::session::SessionManager;

pub struct StateBroadcaster;

impl StateBroadcaster {
    /// Serialize and broadcast the current game state snapshot (tick, player states, projectiles)
    /// to all connected clients over UDP.
    pub fn broadcast(
        socket: &ServerNetworkSocket,
        sessions: &SessionManager,
        tick: u64,
        projectiles: Vec<ProjectileState>,
    ) {
        if sessions.sessions().is_empty() {
            return;
        }

        let players = sessions.active_player_states();
        let state_update = ServerPacket::StateUpdate {
            tick,
            players,
            projectiles,
        };

        if let Ok(bytes) = crate::protocol::serialize_packet(&state_update) {
            for &client_addr in sessions.sessions().keys() {
                let _ = socket.send_bytes(client_addr, &bytes);
            }
        }
    }
}
