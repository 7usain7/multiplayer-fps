pub mod maze;

pub use maze::{MazeMap, TileType};

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
}
