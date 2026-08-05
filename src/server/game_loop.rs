use std::net::SocketAddr;
use crate::maze::MazeMap;
use crate::physics::move_and_slide;
use crate::protocol::ClientPacket;
use crate::server::session::SessionManager;

pub struct ServerGameLoop {
    pub tick: u64,
    pub tick_rate_hz: f32,
    pub player_radius: f32,
}

impl Default for ServerGameLoop {
    fn default() -> Self {
        Self {
            tick: 0,
            tick_rate_hz: 60.0,
            player_radius: 0.3,
        }
    }
}

impl ServerGameLoop {
    pub fn new(tick_rate_hz: f32) -> Self {
        Self {
            tick: 0,
            tick_rate_hz,
            player_radius: 0.3,
        }
    }

    /// Process a single server tick (60Hz), applying authoritative movement
    /// and wall collision updates for all active client input packets.
    pub fn process_input(
        &self,
        sessions: &mut SessionManager,
        addr: SocketAddr,
        packet: ClientPacket,
        map: &MazeMap,
        dt: f32,
    ) {
        if let ClientPacket::Input {
            move_x,
            move_y,
            yaw,
            shoot: _,
            ..
        } = packet
        {
            if let Some(session) = sessions.get_session_mut(&addr) {
                if !session.is_alive {
                    return;
                }

                session.yaw = yaw;

                if move_x != 0.0 || move_y != 0.0 {
                    let dx = move_x * dt;
                    let dy = move_y * dt;

                    let (new_x, new_y) =
                        move_and_slide(map, session.x, session.y, dx, dy, self.player_radius);

                    session.x = new_x;
                    session.y = new_y;
                }
            }
        }
    }

    pub fn advance_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }
}
