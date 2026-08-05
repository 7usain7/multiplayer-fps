use std::thread;
use std::time::Duration;
use multiplayer_fps::maze::get_level;
use multiplayer_fps::protocol::ClientPacket;
use multiplayer_fps::{version, ServerNetworkSocket, SessionManager};

fn main() {
    println!("Starting Multiplayer FPS Server v{}...", version());

    let bind_addr = "0.0.0.0:34254";
    let mut server_socket = match ServerNetworkSocket::bind(bind_addr) {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("Failed to bind UDP server on {}: {}", bind_addr, e);
            return;
        }
    };

    let current_level: u8 = 1;
    let map = get_level(current_level);
    let mut session_manager = SessionManager::new(16);

    println!("Server loop running on {}. Ready for player handshakes...", bind_addr);

    loop {
        let packets = server_socket.poll_packets();
        for (src_addr, packet) in packets {
            match packet {
                ClientPacket::Connect { username } => {
                    session_manager.handle_connect(
                        &server_socket,
                        src_addr,
                        username,
                        current_level,
                        &map,
                    );
                }
                ClientPacket::Ping { timestamp } => {
                    let pong = multiplayer_fps::ServerPacket::Pong { timestamp };
                    let _ = server_socket.send_to(src_addr, &pong);
                }
                _ => {
                    // Update heartbeat for active sessions
                    if let Some(session) = session_manager.get_session_mut(&src_addr) {
                        session.last_seen = std::time::Instant::now();
                    }
                }
            }
        }

        // Tick loop ~60Hz
        thread::sleep(Duration::from_millis(16));
    }
}
