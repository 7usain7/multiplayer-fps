use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use crate::maze::MazeMap;
use crate::protocol::{PlayerState, ServerPacket};
use crate::server::net::ServerNetworkSocket;

#[derive(Debug, Clone)]
pub struct PlayerSession {
    pub id: u32,
    pub username: String,
    pub addr: SocketAddr,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub health: u32,
    pub max_health: u32,
    pub score: u32,
    pub is_alive: bool,
    pub last_seen: Instant,
}

impl PlayerSession {
    pub fn to_state(&self) -> PlayerState {
        PlayerState {
            id: self.id,
            username: self.username.clone(),
            x: self.x,
            y: self.y,
            yaw: self.yaw,
            health: self.health,
            score: self.score,
            is_alive: self.is_alive,
        }
    }
}

pub struct SessionManager {
    next_player_id: u32,
    sessions: HashMap<SocketAddr, PlayerSession>,
    max_clients: usize,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            next_player_id: 1,
            sessions: HashMap::new(),
            max_clients: 16,
        }
    }
}

impl SessionManager {
    pub fn new(max_clients: usize) -> Self {
        Self {
            next_player_id: 1,
            sessions: HashMap::new(),
            max_clients,
        }
    }

    /// Refresh heartbeat timestamp whenever any packet arrives from an active client address.
    pub fn touch_session(&mut self, addr: &SocketAddr) {
        if let Some(session) = self.sessions.get_mut(addr) {
            session.last_seen = Instant::now();
        }
    }

    /// Process a new client connection handshake request.
    pub fn handle_connect(
        &mut self,
        socket: &ServerNetworkSocket,
        addr: SocketAddr,
        username: String,
        current_level: u8,
        map: &MazeMap,
    ) {
        if self.sessions.contains_key(&addr) {
            self.touch_session(&addr);
            return;
        }

        if self.sessions.len() >= self.max_clients {
            println!("Server full! Rejected connection from {}", addr);
            return;
        }

        let player_id = self.next_player_id;
        self.next_player_id += 1;

        let spawns = map.find_spawn_positions();
        let spawn_idx = (player_id as usize) % spawns.len().max(1);
        let (spawn_x, spawn_y) = spawns.get(spawn_idx).copied().unwrap_or((1.5, 1.5));

        let new_session = PlayerSession {
            id: player_id,
            username: username.clone(),
            addr,
            x: spawn_x,
            y: spawn_y,
            yaw: 0.0,
            health: 100,
            max_health: 100,
            score: 0,
            is_alive: true,
            last_seen: Instant::now(),
        };

        self.sessions.insert(addr, new_session);
        println!(
            "Player '{}' (ID: {}) connected from {}",
            username, player_id, addr
        );

        // Send Welcome packet to new player
        let welcome_packet = ServerPacket::Welcome {
            player_id,
            map: map.clone(),
            level: current_level,
        };
        let _ = socket.send_to(addr, &welcome_packet);

        // Notify existing players
        let joined_packet = ServerPacket::PlayerJoined {
            id: player_id,
            username,
        };
        for (&other_addr, _) in self.sessions.iter() {
            if other_addr != addr {
                let _ = socket.send_to(other_addr, &joined_packet);
            }
        }
    }

    /// Remove a player session and notify remaining connected clients.
    pub fn remove_session(&mut self, socket: &ServerNetworkSocket, addr: SocketAddr, reason: &str) {
        if let Some(session) = self.sessions.remove(&addr) {
            println!(
                "Player '{}' (ID: {}) disconnected ({})",
                session.username, session.id, reason
            );

            let left_packet = ServerPacket::PlayerLeft { id: session.id };
            for &other_addr in self.sessions.keys() {
                let _ = socket.send_to(other_addr, &left_packet);
            }
        }
    }

    /// Check all active sessions and drop any client that hasn't sent a packet within `timeout_duration`.
    pub fn clean_timed_out_sessions(
        &mut self,
        socket: &ServerNetworkSocket,
        timeout_duration: Duration,
    ) {
        let now = Instant::now();
        let timed_out_addrs: Vec<SocketAddr> = self
            .sessions
            .iter()
            .filter(|(_, session)| now.duration_since(session.last_seen) > timeout_duration)
            .map(|(&addr, _)| addr)
            .collect();

        for addr in timed_out_addrs {
            self.remove_session(socket, addr, "heartbeat timeout");
        }
    }

    pub fn get_session_mut(&mut self, addr: &SocketAddr) -> Option<&mut PlayerSession> {
        self.sessions.get_mut(addr)
    }

    pub fn sessions(&self) -> &HashMap<SocketAddr, PlayerSession> {
        &self.sessions
    }

    pub fn active_player_states(&self) -> Vec<PlayerState> {
        self.sessions.values().map(|s| s.to_state()).collect()
    }
}
