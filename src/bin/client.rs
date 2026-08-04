use std::io::{self, Write};
use std::net::SocketAddr;
use macroquad::prelude::*;
use multiplayer_fps::input::ClientInputController;
use multiplayer_fps::maze::get_level;
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
        "Connecting to server at {} as username '{}'",
        config.server_addr, config.username
    );

    let current_level: u8 = 1;
    let map = get_level(current_level);
    let fp_renderer = FirstPersonRenderer::default();
    let minimap_renderer = MiniMapRenderer::default();
    let mut input_controller = ClientInputController::default();

    let spawns = map.find_spawn_positions();
    let (mut pos_x, mut pos_y) = spawns.first().copied().unwrap_or((1.5, 1.5));
    let mut yaw: f32 = 0.0;

    let health: u32 = 100;
    let max_health: u32 = 100;

    loop {
        let dt = get_frame_time();

        // Sample input and perform local client prediction
        let input_res = input_controller.update(&map, pos_x, pos_y, yaw, dt);
        pos_x = input_res.new_x;
        pos_y = input_res.new_y;
        yaw = input_res.new_yaw;

        clear_background(BLACK);

        // 1. Draw 3D First-Person View
        fp_renderer.draw(&map, pos_x, pos_y, yaw, screen_width(), screen_height());

        // 2. Draw 2D Mini-Map Overlay
        minimap_renderer.draw(
            &map,
            pos_x,
            pos_y,
            yaw,
            fp_renderer.fov_rad,
            &[],
        );

        // 3. Draw On-Screen HUD & Real-Time FPS Counter
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
