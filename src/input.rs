use macroquad::prelude::*;
use crate::maze::MazeMap;
use crate::physics::move_and_slide;
use crate::protocol::ClientPacket;

pub struct ClientInputController {
    pub sequence: u64,
    pub move_speed: f32,
    pub turn_speed: f32,
    pub player_radius: f32,
}

impl Default for ClientInputController {
    fn default() -> Self {
        Self {
            sequence: 0,
            move_speed: 2.5,
            turn_speed: 2.2,
            player_radius: 0.3,
        }
    }
}

pub struct InputUpdateResult {
    pub new_x: f32,
    pub new_y: f32,
    pub new_yaw: f32,
    pub packet: ClientPacket,
    pub shot_fired: bool,
}

impl ClientInputController {
    /// Sample user input for the current frame, apply local wall-sliding prediction,
    /// and generate a serialized ClientPacket::Input payload.
    pub fn update(
        &mut self,
        map: &MazeMap,
        current_x: f32,
        current_y: f32,
        mut yaw: f32,
        dt: f32,
    ) -> InputUpdateResult {
        self.sequence += 1;

        // 1. Turning (Keyboard Left/Right or A/D)
        let mut turn_dir = 0.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            turn_dir -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            turn_dir += 1.0;
        }
        yaw += turn_dir * self.turn_speed * dt;

        // 2. Forward / Backward Movement (Keyboard Up/Down or W/S)
        let mut forward_dir = 0.0;
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            forward_dir += 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            forward_dir -= 1.0;
        }

        let mut input_dx = 0.0;
        let mut input_dy = 0.0;
        let mut new_x = current_x;
        let mut new_y = current_y;

        if forward_dir != 0.0 {
            let move_vx = yaw.cos() * forward_dir * self.move_speed;
            let move_vy = yaw.sin() * forward_dir * self.move_speed;

            input_dx = move_vx * dt;
            input_dy = move_vy * dt;

            let (next_x, next_y) =
                move_and_slide(map, current_x, current_y, input_dx, input_dy, self.player_radius);
            new_x = next_x;
            new_y = next_y;
        }

        // 3. Shooting (Spacebar or Left Mouse Click)
        let shoot = is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left);

        let packet = ClientPacket::Input {
            sequence: self.sequence,
            move_x: input_dx,
            move_y: input_dy,
            yaw,
            shoot,
        };

        InputUpdateResult {
            new_x,
            new_y,
            new_yaw: yaw,
            packet,
            shot_fired: shoot,
        }
    }
}
