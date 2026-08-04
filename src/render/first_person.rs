use macroquad::prelude::*;
use crate::maze::MazeMap;
use crate::physics::cast_ray;

pub struct FirstPersonRenderer {
    pub fov_rad: f32,
    pub max_render_dist: f32,
}

impl Default for FirstPersonRenderer {
    fn default() -> Self {
        Self {
            fov_rad: 90.0f32.to_radians(), // Wider 80 degrees Field of View
            max_render_dist: 20.0,
        }
    }
}

impl FirstPersonRenderer {
    pub fn new(fov_deg: f32) -> Self {
        Self {
            fov_rad: fov_deg.to_radians(),
            max_render_dist: 20.0,
        }
    }

    /// Render 3D first-person perspective of the maze walls, ceiling, and floor.
    pub fn draw(
        &self,
        map: &MazeMap,
        pos_x: f32,
        pos_y: f32,
        yaw: f32,
        screen_w: f32,
        screen_h: f32,
    ) {
        // 1. Draw Ceiling and Floor backgrounds
        let half_h = screen_h / 2.0;
        draw_rectangle(0.0, 0.0, screen_w, half_h, Color::new(0.05, 0.05, 0.08, 1.0)); // Dark ceiling
        draw_rectangle(0.0, half_h, screen_w, half_h, Color::new(0.12, 0.12, 0.15, 1.0)); // Floor

        // 2. Raycast across columns
        let num_columns = (screen_w / 2.0) as usize; // Step 2 pixels per column for high performance
        let col_width = screen_w / num_columns as f32;

        let half_fov = self.fov_rad / 2.0;

        for col in 0..num_columns {
            // Angle offset across screen FOV
            let screen_ratio = (col as f32 / num_columns as f32) * 2.0 - 1.0;
            let ray_angle = yaw + (screen_ratio * half_fov).atan();

            if let Some(hit) = cast_ray(map, pos_x, pos_y, ray_angle, self.max_render_dist) {
                // Correct fisheye distortion
                let angle_diff = ray_angle - yaw;
                let perp_dist = (hit.distance * angle_diff.cos()).max(0.01);

                // Calculate wall projection height
                let wall_h = (screen_h / perp_dist).min(screen_h * 2.0);
                let wall_top = half_h - wall_h / 2.0;

                // Color & shading based on distance and wall side
                let dist_factor: f32 = (hit.distance / self.max_render_dist).clamp(0.0, 1.0);
                let shade = (1.0 - dist_factor).powf(1.2);

                let base_color = if hit.is_vertical_side {
                    Color::new(0.0 * shade, 0.7 * shade, 0.9 * shade, 1.0) // Cyan vertical walls
                } else {
                    Color::new(0.0 * shade, 0.5 * shade, 0.7 * shade, 1.0) // Slightly darker horizontal walls
                };

                let col_x = col as f32 * col_width;
                draw_rectangle(col_x, wall_top, col_width, wall_h, base_color);

                // Draw crisp wireframe edge outlining retro Maze War style
                draw_rectangle_lines(
                    col_x,
                    wall_top,
                    col_width,
                    wall_h,
                    1.0,
                    Color::new(0.0, 0.9 * shade, 1.0 * shade, 0.4),
                );
            }
        }
    }
}
