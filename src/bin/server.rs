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

    let mut current_level: u8 = 1;
    let mut map = get_level(current_level);
    let mut session_manager = SessionManager::new(16);
    let mut game_loop = ServerGameLoop::default();

    println!("Authoritative 60Hz Game Loop active on {}", bind_addr);

    let tick_duration = Duration::from_secs_f32(1.0 / 60.0);
    let timeout_duration = Duration::from_secs(10);
    let mut last_tick_time = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_tick_time).as_secs_f32();
        last_tick_time = now;

        // 1. Process all incoming UDP client packets
        let packets = server_socket.poll_packets();
        for (src_addr, packet) in packets {
            // Touch heartbeat for any active packet from client
            session_manager.touch_session(&src_addr);

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
                    session_manager.remove_session(&server_socket, src_addr, "client disconnect");
                }
            }
        }

        // 2. Clean up timed-out player sessions (>10 seconds inactivity)
        session_manager.clean_timed_out_sessions(&server_socket, timeout_duration);

        // 3. Update server projectile positions, wall collisions, and player hit detection
        game_loop.update_projectiles(&mut session_manager, &map, dt);

        // 4. Respawn dead players after 3-second death timer
        session_manager.respawn_dead_players(&map);

        // 5. Advance server tick
        let tick = game_loop.advance_tick();

        // 6. Check for Level Exit condition (Multi-Level Progression: Level 1 -> 2 -> 3 -> 1)
        if let Some(winner_id) = game_loop.check_level_exit(&session_manager, &map) {
            let next_level = if current_level >= 3 { 1 } else { current_level + 1 };
            println!(
                "Player (ID: {}) reached the Level Exit! Advancing level from {} to {}...",
                winner_id, current_level, next_level
            );

            current_level = next_level;
            map = get_level(current_level);
            game_loop.projectiles.clear();

            // Teleport all connected players to spawn positions in the new level
            let spawns = map.find_spawn_positions();
            for session in session_manager.sessions_mut().values_mut() {
                let spawn_idx = (session.id as usize) % spawns.len().max(1);
                let (sx, sy) = spawns.get(spawn_idx).copied().unwrap_or((1.5, 1.5));
                session.x = sx;
                session.y = sy;
                session.health = session.max_health;
                session.is_alive = true;
                session.death_time = None;
            }

            // Broadcast LevelChange packet to all connected clients
            let level_change_packet = multiplayer_fps::protocol::ServerPacket::LevelChange {
                new_level: current_level,
                map: map.clone(),
            };
            for &addr in session_manager.sessions().keys() {
                let _ = server_socket.send_to(addr, &level_change_packet);
            }
        }

        // 7. Broadcast state update snapshot over UDP to all connected clients
        let active_projectiles = game_loop.active_projectile_states();
        StateBroadcaster::broadcast(&server_socket, &session_manager, tick, active_projectiles);

        // 8. Maintain 60Hz tick timing
        let elapsed = now.elapsed();
        if elapsed < tick_duration {
            thread::sleep(tick_duration - elapsed);
        }
    }
}
