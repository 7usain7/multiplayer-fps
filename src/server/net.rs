use std::io;
use std::net::{SocketAddr, UdpSocket};
use crate::protocol::{deserialize_packet, serialize_packet, ClientPacket, ServerPacket};

pub struct ServerNetworkSocket {
    socket: UdpSocket,
    buffer: [u8; 4096],
}

impl ServerNetworkSocket {
    /// Bind a non-blocking UDP socket to the specified port (default "0.0.0.0:34254").
    pub fn bind(bind_addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;

        println!("UDP Server listening on non-blocking socket at {}", socket.local_addr()?);

        Ok(Self {
            socket,
            buffer: [0u8; 4096],
        })
    }

    /// Read all currently pending UDP datagrams without blocking.
    /// Returns a list of `(SocketAddr, ClientPacket)` tuples.
    pub fn poll_packets(&mut self) -> Vec<(SocketAddr, ClientPacket)> {
        let mut packets = Vec::new();

        loop {
            match self.socket.recv_from(&mut self.buffer) {
                Ok((amt, src_addr)) => {
                    match deserialize_packet::<ClientPacket>(&self.buffer[..amt]) {
                        Ok(client_packet) => {
                            packets.push((src_addr, client_packet));
                        }
                        Err(err) => {
                            eprintln!("Failed to deserialize packet from {}: {}", src_addr, err);
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No more packets to read in this tick loop
                    break;
                }
                Err(e) => {
                    eprintln!("UDP receive error: {}", e);
                    break;
                }
            }
        }

        packets
    }

    /// Send a serialized ServerPacket to a specific client address.
    pub fn send_to(&self, target_addr: SocketAddr, packet: &ServerPacket) -> io::Result<usize> {
        match serialize_packet(packet) {
            Ok(bytes) => self.socket.send_to(&bytes, target_addr),
            Err(e) => {
                eprintln!("Failed to serialize ServerPacket: {}", e);
                Err(io::Error::new(io::ErrorKind::InvalidData, e))
            }
        }
    }
}
