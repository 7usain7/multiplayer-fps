use macroquad::prelude::*;

pub struct HudRenderer;

impl HudRenderer {
    /// Render on-screen HUD overlay including real-time FPS counter, health, and player info.
    pub fn draw(
        fps: i32,
        username: &str,
        server_addr_str: &str,
        level_num: u8,
        health: u32,
        max_health: u32,
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

        // 3. Draw Health Bar (Bottom Left Corner)
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
    }
}
