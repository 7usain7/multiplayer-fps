use std::thread;
use std::time::Duration;
use multiplayer_fps::{version, ServerNetworkSocket};

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

    println!("Server main loop active. Listening for UDP clients...");

    // Server loop listening for packets
    loop {
        let packets = server_socket.poll_packets();
        for (src_addr, packet) in packets {
            println!("Received UDP packet from {}: {:?}", src_addr, packet);
        }

        // Tick delay ~60Hz
        thread::sleep(Duration::from_millis(16));
    }
}
