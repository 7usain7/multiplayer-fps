use std::thread;
use std::time::{Duration, Instant};
use multiplayer_fps::maze::get_level;
use multiplayer_fps::protocol::ClientPacket;
use multiplayer_fps::{
    version, ServerGameLoop, ServerNetworkSocket, SessionManager, StateBroadcaster,
};

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
    let mut game_loop = ServerGameLoop::default();

    println!("Authoritative 60Hz Game Loop active on {}", bind_addr);

    let tick_duration = Duration::from_secs_f32(1.0 / 60.0);
    let mut last_tick_time = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_tick_time).as_secs_f32();
        last_tick_time = now;

        // 1. Process all incoming UDP client packets
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
                ClientPacket::Input { .. } => {
                    game_loop.process_input(&mut session_manager, src_addr, packet, &map, dt);
                }
                ClientPacket::Ping { timestamp } => {
                    let pong = multiplayer_fps::ServerPacket::Pong { timestamp };
                    let _ = server_socket.send_to(src_addr, &pong);
                }
                ClientPacket::Disconnect => {
                    // Session disconnect handled in step 3.5
                }
            }
        }

        // 2. Advance server tick
        let tick = game_loop.advance_tick();

        // 3. Broadcast state update snapshot over UDP to all connected clients
        StateBroadcaster::broadcast(&server_socket, &session_manager, tick, Vec::new());

        // 4. Maintain 60Hz tick timing
        let elapsed = now.elapsed();
        if elapsed < tick_duration {
            thread::sleep(tick_duration - elapsed);
        }
    }
}
