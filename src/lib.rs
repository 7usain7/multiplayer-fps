pub mod input;
pub mod maze;
pub mod physics;
pub mod protocol;
pub mod render;
pub mod server;

pub use input::ClientInputController;
pub use maze::{get_level, parse_ascii_map, MazeMap, TileType};
pub use physics::{cast_ray, check_wall_collision, move_and_slide, RayHit};
pub use protocol::{
    deserialize_packet, serialize_packet, ClientPacket, PlayerState, ProjectileState,
    ServerPacket,
};
pub use render::{FirstPersonRenderer, HudRenderer, MiniMapRenderer};
pub use server::{PlayerSession, ServerNetworkSocket, SessionManager};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_map_creation_and_conversion() {
        let mut map = MazeMap::new(5, 5, TileType::Wall);
        map.set_tile(1, 1, TileType::Empty);
        map.set_tile(2, 2, TileType::Spawn);

        assert!(map.is_wall(0, 0));
        assert!(!map.is_wall(1, 1));
        assert_eq!(map.get_tile(2, 2), Some(TileType::Spawn));

        let (world_x, world_y) = map.grid_to_world_center(1, 1);
        assert_eq!((world_x, world_y), (1.5, 1.5));

        let (gx, gy) = map.world_to_grid(1.5, 1.5);
        assert_eq!((gx, gy), (1, 1));

        let spawns = map.find_spawn_positions();
        assert_eq!(spawns.len(), 1);
        assert_eq!(spawns[0], (2.5, 2.5));
    }

    #[test]
    fn test_packet_serialization() {
        let connect_packet = ClientPacket::Connect {
            username: "TestPlayer".to_string(),
        };
        let encoded = serialize_packet(&connect_packet).expect("Failed to serialize ClientPacket");
        let decoded: ClientPacket =
            deserialize_packet(&encoded).expect("Failed to deserialize ClientPacket");
        assert_eq!(connect_packet, decoded);

        let map = MazeMap::new(4, 4, TileType::Empty);
        let welcome_packet = ServerPacket::Welcome {
            player_id: 42,
            map,
            level: 1,
        };
        let encoded_server =
            serialize_packet(&welcome_packet).expect("Failed to serialize ServerPacket");
        let decoded_server: ServerPacket =
            deserialize_packet(&encoded_server).expect("Failed to deserialize ServerPacket");
        assert_eq!(welcome_packet, decoded_server);
    }

    #[test]
    fn test_physics_collision_and_sliding() {
        let mut map = MazeMap::new(5, 5, TileType::Wall);
        map.set_tile(1, 1, TileType::Empty);
        map.set_tile(2, 1, TileType::Empty);

        let player_r = 0.3;
        assert!(!check_wall_collision(&map, 1.5, 1.5, player_r));

        let (new_x, new_y) = move_and_slide(&map, 1.5, 1.5, 0.0, -0.4, player_r);
        assert_eq!(new_y, 1.5);
        assert_eq!(new_x, 1.5);

        let (new_x2, new_y2) = move_and_slide(&map, 1.5, 1.5, 0.4, 0.0, player_r);
        assert_eq!(new_x2, 1.9);
        assert_eq!(new_y2, 1.5);
    }

    #[test]
    fn test_raycasting() {
        let mut map = MazeMap::new(5, 5, TileType::Wall);
        map.set_tile(1, 1, TileType::Empty);
        map.set_tile(2, 1, TileType::Empty);

        let hit = cast_ray(&map, 1.5, 1.5, 0.0, 10.0);
        assert!(hit.is_some());
        let ray = hit.unwrap();
        assert_eq!(ray.grid_x, 3);
        assert_eq!(ray.grid_y, 1);
        assert!(ray.distance > 0.0);
    }

    #[test]
    fn test_level_registry() {
        for lvl in 1..=3 {
            let map = get_level(lvl);
            assert!(map.width > 0 && map.height > 0);
            let spawns = map.find_spawn_positions();
            assert!(!spawns.is_empty(), "Level {} must have spawn positions", lvl);
        }
    }

    #[test]
    fn test_udp_socket_binding() {
        if let Ok(server_net) = ServerNetworkSocket::bind("127.0.0.1:0") {
            drop(server_net);
        }
    }
}
