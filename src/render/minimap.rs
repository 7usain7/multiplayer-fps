use macroquad::prelude::*;
use crate::maze::{MazeMap, TileType};
use crate::protocol::{PlayerState, ProjectileState};

pub struct MiniMapRenderer {
    pub map_scale: f32, // Pixels per grid cell
    pub offset_x: f32,  // Screen offset X
    pub offset_y: f32,  // Screen offset Y
}

impl Default for MiniMapRenderer {
    fn default() -> Self {
        Self {
            map_scale: 10.0,
            offset_x: 15.0,
            offset_y: 15.0,
        }
    }
}

impl MiniMapRenderer {
    pub fn new(map_scale: f32, offset_x: f32, offset_y: f32) -> Self {
        Self {
            map_scale,
            offset_x,
            offset_y,
        }
    }

    /// Render 2D top-down mini-map HUD showing maze layout, player position + tight vision cone,
    /// opponent player positions, and active projectiles.
    pub fn draw(
        &self,
        map: &MazeMap,
        self_x: f32,
        self_y: f32,
        self_yaw: f32,
        fov_rad: f32,
        other_players: &[PlayerState],
        projectiles: &[ProjectileState],
    ) {
        let map_w_px = map.width as f32 * self.map_scale;
        let map_h_px = map.height as f32 * self.map_scale;

        // 1. Draw semi-transparent background panel and border
        draw_rectangle(
            self.offset_x - 4.0,
            self.offset_y - 4.0,
            map_w_px + 8.0,
            map_h_px + 8.0,
            Color::new(0.0, 0.0, 0.0, 0.75),
        );
        draw_rectangle_lines(
            self.offset_x - 4.0,
            self.offset_y - 4.0,
            map_w_px + 8.0,
            map_h_px + 8.0,
            1.5,
            Color::new(0.0, 0.8, 1.0, 0.8),
        );

        // 2. Draw Maze Grid Tiles
        for y in 0..map.height {
            for x in 0..map.width {
                let px = self.offset_x + x as f32 * self.map_scale;
                let py = self.offset_y + y as f32 * self.map_scale;

                match map.get_tile(x, y) {
                    Some(TileType::Wall) => {
                        draw_rectangle(
                            px,
                            py,
                            self.map_scale - 1.0,
                            self.map_scale - 1.0,
                            Color::new(0.1, 0.4, 0.6, 0.85),
                        );
                    }
                    Some(TileType::Exit) => {
                        draw_rectangle(
                            px,
                            py,
                            self.map_scale - 1.0,
                            self.map_scale - 1.0,
                            Color::new(1.0, 0.8, 0.0, 0.6),
                        );
                    }
                    _ => {
                        // Empty corridor
                        draw_rectangle(
                            px,
                            py,
                            self.map_scale - 1.0,
                            self.map_scale - 1.0,
                            Color::new(0.05, 0.08, 0.12, 0.4),
                        );
                    }
                }
            }
        }

        // Helper to convert world position -> mini-map pixel position
        let world_to_map_px = |wx: f32, wy: f32| -> (f32, f32) {
            (
                self.offset_x + (wx / map.tile_size) * self.map_scale,
                self.offset_y + (wy / map.tile_size) * self.map_scale,
            )
        };

        // 3. Draw Active Projectiles (Yellow Dots)
        for proj in projectiles {
            let (pr_x, pr_y) = world_to_map_px(proj.x, proj.y);
            draw_circle(pr_x, pr_y, 2.0, Color::new(1.0, 0.9, 0.2, 1.0));
        }

        // 4. Draw Opponent Players (Red Dots)
        for player in other_players {
            if player.is_alive {
                let (op_x, op_y) = world_to_map_px(player.x, player.y);
                draw_circle(op_x, op_y, 3.5, Color::new(1.0, 0.2, 0.2, 1.0));
            }
        }

        // 5. Draw Tight & Compact FOV Vision Cone
        let (p_px, p_py) = world_to_map_px(self_x, self_y);
        let half_fov = (fov_rad * 0.5) / 2.0; // Tighter half-angle (~20 degrees)
        let cone_len = 1.2 * self.map_scale;  // Compact length (12 pixels)

        let left_angle = self_yaw - half_fov;
        let right_angle = self_yaw + half_fov;

        let left_x = p_px + left_angle.cos() * cone_len;
        let left_y = p_py + left_angle.sin() * cone_len;
        let right_x = p_px + right_angle.cos() * cone_len;
        let right_y = p_py + right_angle.sin() * cone_len;

        draw_triangle(
            Vec2::new(p_px, p_py),
            Vec2::new(left_x, left_y),
            Vec2::new(right_x, right_y),
            Color::new(0.0, 1.0, 0.5, 0.3),
        );
        draw_line(p_px, p_py, left_x, left_y, 1.0, Color::new(0.0, 1.0, 0.5, 0.7));
        draw_line(p_px, p_py, right_x, right_y, 1.0, Color::new(0.0, 1.0, 0.5, 0.7));

        // 6. Draw Self Player Circle
        draw_circle(p_px, p_py, 3.5, Color::new(0.0, 1.0, 0.3, 1.0));
    }
}
