use macroquad::prelude::*;
use crate::maze::MazeMap;
use crate::physics::cast_ray;
use crate::protocol::{PlayerState, ProjectileState};

pub struct DirectionalTextures {
    pub front: Texture2D,
    pub back: Texture2D,
    pub left: Texture2D,
    pub right: Texture2D,
}

pub struct FirstPersonRenderer {
    pub fov_rad: f32,
    pub max_render_dist: f32,
}

impl Default for FirstPersonRenderer {
    fn default() -> Self {
        Self {
            fov_rad: 90.0f32.to_radians(), // 90 degrees FOV
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

    /// Normalize angle into [-PI, PI] range
    fn normalize_angle(angle: f32) -> f32 {
        let mut a = angle;
        while a > std::f32::consts::PI {
            a -= 2.0 * std::f32::consts::PI;
        }
        while a < -std::f32::consts::PI {
            a += 2.0 * std::f32::consts::PI;
        }
        a
    }

    /// Render 3D first-person perspective of maze walls, floor, ceiling, opponent players, and projectiles.
    pub fn draw(
        &self,
        map: &MazeMap,
        pos_x: f32,
        pos_y: f32,
        yaw: f32,
        screen_w: f32,
        screen_h: f32,
        other_players: &[PlayerState],
        projectiles: &[ProjectileState],
        opponent_textures: Option<&DirectionalTextures>,
    ) {
        // 1. Draw Ceiling and Floor backgrounds
        let half_h = screen_h / 2.0;
        draw_rectangle(0.0, 0.0, screen_w, half_h, Color::new(0.05, 0.05, 0.08, 1.0)); // Dark ceiling
        draw_rectangle(0.0, half_h, screen_w, half_h, Color::new(0.12, 0.12, 0.15, 1.0)); // Floor

        // 2. Raycast across columns and record Z-Buffer distances
        let num_columns = (screen_w / 2.0) as usize; // 2 pixels per column for high FPS
        let col_width = screen_w / num_columns as f32;
        let half_fov = self.fov_rad / 2.0;

        let mut z_buffer = vec![f32::INFINITY; num_columns];

        for col in 0..num_columns {
            let screen_ratio = (col as f32 / num_columns as f32) * 2.0 - 1.0;
            let ray_angle = yaw + (screen_ratio * half_fov).atan();

            if let Some(hit) = cast_ray(map, pos_x, pos_y, ray_angle, self.max_render_dist) {
                let angle_diff = Self::normalize_angle(ray_angle - yaw);
                let perp_dist = (hit.distance * angle_diff.cos()).max(0.01);
                z_buffer[col] = perp_dist;

                let wall_h = (screen_h / perp_dist).min(screen_h * 2.0);
                let wall_top = half_h - wall_h / 2.0;

                let dist_factor: f32 = (hit.distance / self.max_render_dist).clamp(0.0, 1.0);
                let shade = (1.0 - dist_factor).powf(1.2);

                let is_exit = map.get_tile(hit.grid_x, hit.grid_y) == Some(crate::maze::TileType::Exit);

                let base_color = if is_exit {
                    if hit.is_vertical_side {
                        Color::new(1.0 * shade, 0.85 * shade, 0.1 * shade, 1.0)
                    } else {
                        Color::new(0.85 * shade, 0.7 * shade, 0.05 * shade, 1.0)
                    }
                } else if hit.is_vertical_side {
                    Color::new(0.0 * shade, 0.7 * shade, 0.9 * shade, 1.0)
                } else {
                    Color::new(0.0 * shade, 0.5 * shade, 0.7 * shade, 1.0)
                };

                let col_x = col as f32 * col_width;
                draw_rectangle(col_x, wall_top, col_width, wall_h, base_color);

                // Wireframe wall borders
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

        // 3. Render 3D Opponent Player Sprites (with Z-Buffer Occlusion & Directional View)
        for player in other_players {
            if !player.is_alive {
                continue;
            }

            let dx = player.x - pos_x;
            let dy = player.y - pos_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 0.2 || dist > self.max_render_dist {
                continue;
            }

            let angle_to_player = Self::normalize_angle(dy.atan2(dx) - yaw);
            if angle_to_player.abs() > half_fov * 1.5 {
                continue; // Outside FOV view angle
            }

            // Determine directional texture based on opponent facing angle relative to observer
            let angle_to_camera = (pos_y - player.y).atan2(pos_x - player.x);
            let relative_angle = Self::normalize_angle(player.yaw - angle_to_camera);

            let active_tex = opponent_textures.map(|t| {
                let abs_angle = relative_angle.abs();
                if abs_angle < std::f32::consts::FRAC_PI_4 {
                    &t.front
                } else if abs_angle > 3.0 * std::f32::consts::FRAC_PI_4 {
                    &t.back
                } else if relative_angle > 0.0 {
                    &t.right
                } else {
                    &t.left
                }
            });

            // Project opponent onto 2D screen coordinates
            let screen_ratio = angle_to_player.tan() / half_fov.tan();
            let sprite_center_x = (screen_w / 2.0) + (screen_ratio * (screen_w / 2.0));
            let sprite_h = (screen_h / dist).clamp(10.0, screen_h * 1.5);
            let sprite_w = sprite_h * 0.8;
            let sprite_top = half_h - sprite_h / 2.0;
            let sprite_left = sprite_center_x - sprite_w / 2.0;

            // Draw column by column respecting Z-Buffer depth
            let start_col = ((sprite_left / col_width).floor() as i32).max(0) as usize;
            let end_col = (((sprite_left + sprite_w) / col_width).ceil() as i32)
                .max(0)
                .min(num_columns as i32 - 1) as usize;

            for col in start_col..=end_col {
                if col < z_buffer.len() && dist < z_buffer[col] {
                    let col_x = col as f32 * col_width;
                    let draw_w = col_width;

                    if let Some(tex) = active_tex {
                        let tex_w = tex.width();
                        let tex_h = tex.height();
                        if sprite_w > 0.0 {
                            let u = (col_x - sprite_left) / sprite_w;
                            let src_rect = Rect::new(u * tex_w, 0.0, (col_width / sprite_w) * tex_w, tex_h);
                            
                            let dist_factor: f32 = (dist / self.max_render_dist).clamp(0.0, 1.0);
                            let shade = (1.0 - dist_factor).powf(0.8).max(0.2);

                            draw_texture_ex(
                                tex,
                                col_x,
                                sprite_top,
                                Color::new(shade, shade, shade, 1.0),
                                DrawTextureParams {
                                    dest_size: Some(vec2(draw_w, sprite_h)),
                                    source: Some(src_rect),
                                    ..Default::default()
                                },
                            );
                        }
                    } else {
                        // Fallback Red body sprite
                        draw_rectangle(
                            col_x,
                            sprite_top,
                            draw_w,
                            sprite_h,
                            Color::new(0.9, 0.15, 0.15, 1.0),
                        );
                    }
                }
            }
        }

        // 4. Render 3D Active Projectiles (Glowing Orbs)
        for proj in projectiles {
            let dx = proj.x - pos_x;
            let dy = proj.y - pos_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 0.1 || dist > self.max_render_dist {
                continue;
            }

            let angle_to_proj = Self::normalize_angle(dy.atan2(dx) - yaw);
            if angle_to_proj.abs() > half_fov * 1.5 {
                continue;
            }

            let screen_ratio = angle_to_proj.tan() / half_fov.tan();
            let proj_center_x = (screen_w / 2.0) + (screen_ratio * (screen_w / 2.0));
            let proj_size = (screen_h / dist * 0.15).clamp(4.0, 40.0);
            let _proj_top = half_h - proj_size / 2.0;

            let col_idx = (proj_center_x / col_width).floor() as i32;
            if col_idx >= 0 && (col_idx as usize) < z_buffer.len() {
                if dist < z_buffer[col_idx as usize] {
                    draw_circle(
                        proj_center_x,
                        half_h,
                        proj_size * 0.5,
                        Color::new(1.0, 0.9, 0.1, 1.0),
                    );
                    draw_circle(
                        proj_center_x,
                        half_h,
                        proj_size * 0.25,
                        Color::new(1.0, 1.0, 1.0, 1.0),
                    );
                }
            }
        }
    }
}
