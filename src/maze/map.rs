use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Empty,
    Wall,
    Spawn,
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileType>, // Row-major order: index = y * width + x
    pub tile_size: f32,       // World units per grid cell (e.g., 1.0)
}

impl MazeMap {
    pub fn new(width: usize, height: usize, default_tile: TileType) -> Self {
        Self {
            width,
            height,
            tiles: vec![default_tile; width * height],
            tile_size: 1.0,
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<TileType> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: TileType) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    pub fn is_wall(&self, x: usize, y: usize) -> bool {
        match self.get_tile(x, y) {
            Some(TileType::Wall) | None => true, // Out of bounds treated as wall
            _ => false,
        }
    }

    pub fn is_solid_at_world(&self, world_x: f32, world_y: f32) -> bool {
        let (gx, gy) = self.world_to_grid(world_x, world_y);
        if gx < 0 || gy < 0 || gx as usize >= self.width || gy as usize >= self.height {
            return true; // Out of bounds is solid
        }
        self.is_wall(gx as usize, gy as usize)
    }

    pub fn world_to_grid(&self, world_x: f32, world_y: f32) -> (isize, isize) {
        (
            (world_x / self.tile_size).floor() as isize,
            (world_y / self.tile_size).floor() as isize,
        )
    }

    pub fn grid_to_world_center(&self, x: usize, y: usize) -> (f32, f32) {
        (
            (x as f32 + 0.5) * self.tile_size,
            (y as f32 + 0.5) * self.tile_size,
        )
    }

    pub fn find_spawn_positions(&self) -> Vec<(f32, f32)> {
        let mut spawns = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_tile(x, y) == Some(TileType::Spawn) {
                    spawns.push(self.grid_to_world_center(x, y));
                }
            }
        }
        if spawns.is_empty() {
            // Fallback to any empty tile if no explicit Spawn tile is set
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.get_tile(x, y) == Some(TileType::Empty) {
                        spawns.push(self.grid_to_world_center(x, y));
                    }
                }
            }
        }
        spawns
    }
}
