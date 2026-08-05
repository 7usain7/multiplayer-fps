use macroquad::prelude::*;
use crate::protocol::PlayerState;

pub struct HudRenderer;

impl HudRenderer {
    /// Render on-screen HUD overlay including real-time FPS counter, health, scoreboard, and player info.
    pub fn draw(
        fps: i32,
        username: &str,
        server_addr_str: &str,
        level_num: u8,
        health: u32,
        max_health: u32,
        respawn_timer: f32,
        my_score: u32,
        opponents: &[PlayerState],
        screen_w: f32,
        screen_h: f32,
    ) {
        let center_x = screen_w / 2.0;

        // 1. Draw Real-Time FPS Counter (Top Right Corner)
        let fps_text = format!("FPS: {}", fps);
        let fps_color = if fps >= 50 {
            Color::new(0.0, 1.0, 0.4, 0.9) // Green if >= 50 FPS
        } else if fps >= 30 {
            Color::new(1.0, 0.8, 0.0, 0.9) // Yellow if 30-49 FPS
        } else {
            Color::new(1.0, 0.2, 0.2, 0.9) // Red if low FPS
        };

        let fps_font_size = 20.0;
        let fps_w = measure_text(&fps_text, None, fps_font_size as u16, 1.0).width;
        let fps_x = screen_w - fps_w - 20.0;
        let fps_y = 30.0;

        // Background pill for FPS text
        draw_rectangle(
            fps_x - 8.0,
            fps_y - 18.0,
            fps_w + 16.0,
            24.0,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        draw_rectangle_lines(
            fps_x - 8.0,
            fps_y - 18.0,
            fps_w + 16.0,
            24.0,
            1.0,
            fps_color,
        );
        draw_text(&fps_text, fps_x, fps_y, fps_font_size, fps_color);

        // 2. Draw Player & Level Banner (Top Center)
        let banner_text = format!("{} | Server: {} | Level {}", username, server_addr_str, level_num);
        let banner_font_size = 18.0;
        let banner_dim = measure_text(&banner_text, None, banner_font_size as u16, 1.0);
        let banner_x = center_x - banner_dim.width / 2.0;
        let banner_y = 30.0;

        draw_rectangle(
            banner_x - 10.0,
            banner_y - 18.0,
            banner_dim.width + 20.0,
            24.0,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        draw_text(&banner_text, banner_x, banner_y, banner_font_size, WHITE);

        // 3. Draw Top-Right Scoreboard Panel (Below FPS Pill)
        let sb_x = screen_w - 170.0;
        let mut sb_y = 65.0;

        let mut score_list: Vec<(&str, u32, bool)> = Vec::new();
        score_list.push((username, my_score, true));
        for p in opponents {
            score_list.push((&p.username, p.score, false));
        }
        score_list.sort_by(|a, b| b.1.cmp(&a.1)); // Highest score first

        let sb_w = 150.0;
        let row_h = 18.0;
        let header_h = 22.0;
        let sb_h = header_h + score_list.len() as f32 * row_h + 8.0;

        // Dark background box with gold border
        draw_rectangle(sb_x - 8.0, sb_y - 16.0, sb_w + 8.0, sb_h, Color::new(0.0, 0.0, 0.0, 0.7));
        draw_rectangle_lines(sb_x - 8.0, sb_y - 16.0, sb_w + 8.0, sb_h, 1.0, Color::new(1.0, 0.8, 0.0, 0.8));

        // Header
        draw_text("SCOREBOARD", sb_x, sb_y, 14.0, Color::new(1.0, 0.8, 0.0, 1.0));
        sb_y += row_h;

        // Player score entries
        for (name, score, is_self) in score_list {
            let color = if is_self {
                Color::new(0.0, 1.0, 0.5, 1.0) // Bright cyan-green for local player
            } else {
                Color::new(0.9, 0.9, 0.9, 0.9)
            };
            let text = format!("{}: {} pts", name, score);
            draw_text(&text, sb_x, sb_y, 13.0, color);
            sb_y += row_h;
        }

        // 4. Draw Health Bar (Bottom Left Corner)
        let health_bar_w = 160.0;
        let health_bar_h = 16.0;
        let health_x = 20.0;
        let health_y = screen_h - 20.0;

        let hp_text = format!("HP: {}/{}", health, max_health);
        let font_size = 16.0;
        let hp_text_dim = measure_text(&hp_text, None, font_size as u16, 1.0);

        let total_content_w = health_bar_w + 12.0 + hp_text_dim.width;
        let padding_x = 10.0;
        let padding_y = 8.0;

        let box_x = health_x - padding_x;
        let box_y = health_y - health_bar_h - padding_y + 2.0;
        let box_w = total_content_w + padding_x * 2.0;
        let box_h = health_bar_h + padding_y * 2.0;

        let hp_ratio = (health as f32 / max_health as f32).clamp(0.0, 1.0);
        let hp_fill_w = health_bar_w * hp_ratio;

        let hp_color = if hp_ratio > 0.5 {
            Color::new(0.0, 0.9, 0.4, 0.9)
        } else if hp_ratio > 0.25 {
            Color::new(1.0, 0.7, 0.0, 0.9)
        } else {
            Color::new(1.0, 0.2, 0.2, 0.9)
        };

        // Dynamically sized health background box
        draw_rectangle(box_x, box_y, box_w, box_h, Color::new(0.0, 0.0, 0.0, 0.65));
        draw_rectangle_lines(box_x, box_y, box_w, box_h, 1.0, Color::new(0.0, 0.8, 1.0, 0.6));

        // Fill health bar
        let bar_draw_y = health_y - health_bar_h;
        draw_rectangle(health_x, bar_draw_y, hp_fill_w, health_bar_h, hp_color);
        draw_rectangle_lines(
            health_x,
            bar_draw_y,
            health_bar_w,
            health_bar_h,
            1.5,
            WHITE,
        );

        // Draw HP text next to health bar
        draw_text(
            &hp_text,
            health_x + health_bar_w + 10.0,
            health_y - 2.0,
            font_size,
            WHITE,
        );

        // 5. Draw Precision Center Crosshair (Screen Center)
        if health > 0 {
            let center_y = screen_h / 2.0;
            let ch_size = 6.0; // Line length
            let ch_gap = 4.0;  // Gap around center dot
            let ch_color = Color::new(0.0, 1.0, 0.5, 0.85); // Neon green crosshair

            // Center dot
            draw_circle(center_x, center_y, 1.5, Color::new(1.0, 1.0, 1.0, 0.9));

            // Top reticle line
            draw_line(center_x, center_y - ch_gap - ch_size, center_x, center_y - ch_gap, 1.5, ch_color);
            // Bottom reticle line
            draw_line(center_x, center_y + ch_gap, center_x, center_y + ch_gap + ch_size, 1.5, ch_color);
            // Left reticle line
            draw_line(center_x - ch_gap - ch_size, center_y, center_x - ch_gap, center_y, 1.5, ch_color);
            // Right reticle line
            draw_line(center_x + ch_gap, center_y, center_x + ch_gap + ch_size, center_y, 1.5, ch_color);
        }

        // 6. Draw Center Respawn Countdown Overlay when health == 0
        if health == 0 {
            // Dark red semi-transparent vignette/panel
            draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.2, 0.0, 0.0, 0.5));

            let elim_text = "YOU WERE ELIMINATED!";
            let elim_font_size = 36.0;
            let elim_dim = measure_text(elim_text, None, elim_font_size as u16, 1.0);
            draw_text(
                elim_text,
                center_x - elim_dim.width / 2.0,
                screen_h / 2.0 - 20.0,
                elim_font_size,
                Color::new(1.0, 0.25, 0.25, 1.0),
            );

            let count_num = (respawn_timer.ceil() as u32).clamp(1, 3);
            let respawn_text = format!("Respawn in {}...", count_num);
            let respawn_font_size = 28.0;
            let respawn_dim = measure_text(&respawn_text, None, respawn_font_size as u16, 1.0);
            draw_text(
                &respawn_text,
                center_x - respawn_dim.width / 2.0,
                screen_h / 2.0 + 30.0,
                respawn_font_size,
                Color::new(1.0, 0.9, 0.2, 1.0),
            );
        }
    }
}
