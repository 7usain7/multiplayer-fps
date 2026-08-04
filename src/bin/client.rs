use std::io::{self, Write};
use std::net::SocketAddr;
use macroquad::prelude::*;
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

    // 1. Prompt for IP address
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

    // 2. Prompt for Username
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
        "Client connected configuration -> Server: {}, Username: {}",
        config.server_addr, config.username
    );

    // Basic GUI loop placeholder for Step 2.1
    loop {
        clear_background(BLACK);

        draw_text(
            &format!("Connecting to {} as {}...", config.server_addr, config.username),
            20.0,
            40.0,
            24.0,
            WHITE,
        );

        next_frame().await;
    }
}
