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
        _dt: f32,
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
                    // move_x and move_y are actual displacement (dx, dy) from the client.
                    // Prevent speed-hacking by clamping the maximum displacement per packet.
                    let len_sq = move_x * move_x + move_y * move_y;
                    let max_displacement = 0.5; // ~5.0 units/s at 10hz, plenty for 2.5 speed at 60hz (0.04 per frame)
                    
                    let (valid_dx, valid_dy) = if len_sq > max_displacement * max_displacement {
                        let scale = max_displacement / len_sq.sqrt();
                        (move_x * scale, move_y * scale)
                    } else {
                        (move_x, move_y)
                    };

                    let (new_x, new_y) =
                        move_and_slide(map, session.x, session.y, valid_dx, valid_dy, self.player_radius);

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
