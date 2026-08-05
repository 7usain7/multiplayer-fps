use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use macroquad::prelude::*;
use multiplayer_fps::input::ClientInputController;
use multiplayer_fps::maze::get_level;
use multiplayer_fps::protocol::{
    deserialize_packet, serialize_packet, ClientPacket, PlayerState, ProjectileState,
    ServerPacket,
};
use multiplayer_fps::render::{FirstPersonRenderer, HudRenderer, MiniMapRenderer};
use multiplayer_fps::version;

pub struct ClientConfig {
    pub server_addr: SocketAddr,
    pub username: String,
}

fn window_conf() -> Conf {
    Conf {
        window_title: format!("Maze War Multiplayer FPS v{}", version()),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        fullscreen: false,
        sample_count: 4,
        ..Default::default()
    }
}

pub fn prompt_connection_info() -> ClientConfig {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    print!("Enter IP Address [default: 127.0.0.1:34254]: ");
    handle.flush().unwrap();

    let mut ip_input = String::new();
    io::stdin().read_line(&mut ip_input).unwrap();
    let ip_str = ip_input.trim();

    let server_addr: SocketAddr = if ip_str.is_empty() {
        "127.0.0.1:34254".parse().unwrap()
    } else if let Ok(addr) = ip_str.parse() {
        addr
    } else if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
        SocketAddr::new(ip, 34254)
    } else {
        println!("Invalid IP address format. Falling back to 127.0.0.1:34254");
        "127.0.0.1:34254".parse().unwrap()
    };

    print!("Enter Name: ");
    handle.flush().unwrap();

    let mut name_input = String::new();
    io::stdin().read_line(&mut name_input).unwrap();
    let username = name_input.trim();
    let username = if username.is_empty() {
        "Player".to_string()
    } else {
        username.to_string()
    };

    println!("Starting...");

    ClientConfig {
        server_addr,
        username,
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let config = prompt_connection_info();
    println!(
        "Connecting to UDP server at {} as username '{}'",
        config.server_addr, config.username
    );

    // Initialize local UDP Socket
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client UDP socket");
    let _ = socket.set_nonblocking(true);

    // Send Connect handshake packet to server
    let connect_packet = ClientPacket::Connect {
        username: config.username.clone(),
    };
    if let Ok(bytes) = serialize_packet(&connect_packet) {
        let _ = socket.send_to(&bytes, config.server_addr);
    }

    let mut current_level: u8 = 1;
    let mut map = get_level(current_level);
    let fp_renderer = FirstPersonRenderer::default();
    let minimap_renderer = MiniMapRenderer::default();
    let mut input_controller = ClientInputController::default();

    let spawns = map.find_spawn_positions();
    let (mut pos_x, mut pos_y) = spawns.first().copied().unwrap_or((1.5, 1.5));
    let mut yaw: f32 = 0.0;
    let mut my_player_id: Option<u32> = None;

    let mut target_opponents: Vec<PlayerState> = Vec::new();
    let mut smoothed_opponents: Vec<PlayerState> = Vec::new();
    let mut active_projectiles: Vec<ProjectileState> = Vec::new();
    let mut health: u32 = 100;
    let max_health: u32 = 100;
    let mut recv_buf = [0u8; 4096];

    // Load 4-directional eyeball opponent textures (Maze War style)
    let opponent_textures = async {
        let front = load_texture("assets/Eye.png").await.ok()?;
        let back = load_texture("assets/EyeBack.png").await.ok()?;
        let left = load_texture("assets/EyeRight.png").await.ok()?;
        let right = load_texture("assets/EyeLeft.png").await.ok()?;

        front.set_filter(FilterMode::Nearest);
        back.set_filter(FilterMode::Nearest);
        left.set_filter(FilterMode::Nearest);
        right.set_filter(FilterMode::Nearest);

        Some(multiplayer_fps::render::DirectionalTextures {
            front,
            back,
            left,
            right,
        })
    }.await;

    loop {
        let dt = get_frame_time();

        // 1. Read incoming UDP packets from server
        loop {
            match socket.recv_from(&mut recv_buf) {
                Ok((amt, src_addr)) => {
                    if src_addr == config.server_addr {
                        if let Ok(server_packet) = deserialize_packet::<ServerPacket>(&recv_buf[..amt])
                        {
                            match server_packet {
                                ServerPacket::Welcome {
                                    player_id,
                                    map: server_map,
                                    level,
                                } => {
                                    my_player_id = Some(player_id);
                                    map = server_map;
                                    current_level = level;

                                    // Set initial position to server spawn point
                                    let server_spawns = map.find_spawn_positions();
                                    let spawn_idx = (player_id as usize) % server_spawns.len().max(1);
                                    if let Some(&(sx, sy)) = server_spawns.get(spawn_idx) {
                                        pos_x = sx;
                                        pos_y = sy;
                                    }
                                    println!("Received Welcome! Assigned Player ID: {} at ({:.1}, {:.1})", player_id, pos_x, pos_y);
                                }
                                ServerPacket::StateUpdate {
                                    players,
                                    projectiles,
                                    ..
                                } => {
                                    let mut opponents = Vec::new();
                                    for p in players {
                                        if Some(p.id) == my_player_id {
                                            let dist_sq = (pos_x - p.x).powi(2) + (pos_y - p.y).powi(2);
                                            if dist_sq > 0.8 {
                                                pos_x = p.x;
                                                pos_y = p.y;
                                            }
                                            health = p.health;
                                        } else {
                                            opponents.push(p);
                                        }
                                    }
                                    target_opponents = opponents;
                                    active_projectiles = projectiles;
                                }
                                ServerPacket::PlayerLeft { id } => {
                                    target_opponents.retain(|p| p.id != id);
                                    smoothed_opponents.retain(|p| p.id != id);
                                }
                                ServerPacket::LevelChange { new_level, map: new_map } => {
                                    current_level = new_level;
                                    map = new_map;
                                    let server_spawns = map.find_spawn_positions();
                                    if let Some(&(sx, sy)) = server_spawns.first() {
                                        pos_x = sx;
                                        pos_y = sy;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(_) => break,
            }
        }

        // 2. Smoothly interpolate opponent player positions
        smoothed_opponents.retain(|s| target_opponents.iter().any(|t| t.id == s.id));
        for target in &target_opponents {
            if let Some(smoothed) = smoothed_opponents.iter_mut().find(|s| s.id == target.id) {
                smoothed.x = smoothed.x * 0.80 + target.x * 0.20;
                smoothed.y = smoothed.y * 0.80 + target.y * 0.20;
                smoothed.yaw = target.yaw;
                smoothed.health = target.health;
                smoothed.score = target.score;
                smoothed.is_alive = target.is_alive;
            } else {
                smoothed_opponents.push(target.clone());
            }
        }

        // 3. Sample input and transmit input packet to server over UDP
        let input_res = input_controller.update(&map, pos_x, pos_y, yaw, dt);
        pos_x = input_res.new_x;
        pos_y = input_res.new_y;
        yaw = input_res.new_yaw;

        if let Ok(input_bytes) = serialize_packet(&input_res.packet) {
            let _ = socket.send_to(&input_bytes, config.server_addr);
        }

        clear_background(BLACK);

        // 4. Render 3D First-Person View (including 3D opponents and projectiles)
        fp_renderer.draw(
            &map,
            pos_x,
            pos_y,
            yaw,
            screen_width(),
            screen_height(),
            &smoothed_opponents,
            &active_projectiles,
            opponent_textures.as_ref(),
        );

        // 5. Render 2D Mini-Map Overlay (including connected opponents & active projectiles)
        minimap_renderer.draw(
            &map,
            pos_x,
            pos_y,
            yaw,
            fp_renderer.fov_rad,
            &smoothed_opponents,
            &active_projectiles,
        );

        // 6. Render On-Screen HUD & Real-Time FPS Counter
        let fps = get_fps();
        let server_str = config.server_addr.to_string();
        HudRenderer::draw(
            fps,
            &config.username,
            &server_str,
            current_level,
            health,
            max_health,
            screen_width(),
            screen_height(),
        );

        next_frame().await;
    }
}
