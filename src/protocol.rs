use serde::{Deserialize, Serialize};
use crate::maze::MazeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerState {
    pub id: u32,
    pub username: String,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub health: u32,
    pub score: u32,
    pub is_alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectileState {
    pub id: u32,
    pub owner_id: u32,
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClientPacket {
    Connect { username: String },
    Input {
        sequence: u64,
        move_x: f32,
        move_y: f32,
        yaw: f32,
        shoot: bool,
    },
    Ping { timestamp: u64 },
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerPacket {
    Welcome {
        player_id: u32,
        map: MazeMap,
        level: u8,
    },
    StateUpdate {
        tick: u64,
        players: Vec<PlayerState>,
        projectiles: Vec<ProjectileState>,
    },
    Pong { timestamp: u64 },
    LevelChange {
        new_level: u8,
        map: MazeMap,
    },
    PlayerJoined {
        id: u32,
        username: String,
    },
    PlayerLeft { id: u32 },
}

pub fn serialize_packet<T: Serialize>(packet: &T) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(packet)
}

pub fn deserialize_packet<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T, bincode::Error> {
    bincode::deserialize(data)
}
