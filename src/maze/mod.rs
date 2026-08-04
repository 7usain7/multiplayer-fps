pub mod levels;
pub mod map;

pub use levels::{get_level, parse_ascii_map};
pub use map::{MazeMap, TileType};
