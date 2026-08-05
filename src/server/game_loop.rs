use std::collections::HashMap;
use std::net::SocketAddr;
use crate::maze::MazeMap;
use crate::physics::move_and_slide;
use crate::protocol::{ClientPacket, ProjectileState};
use crate::server::session::SessionManager;

#[derive(Debug, Clone)]
pub struct ServerProjectile {
    pub id: u32,
    pub owner_id: u32,
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub speed: f32,
    pub lifetime: f32,
}

impl ServerProjectile {
    pub fn to_state(&self) -> ProjectileState {
        ProjectileState {
            id: self.id,
            owner_id: self.owner_id,
            x: self.x,
            y: self.y,
            dir_x: self.dir_x,
            dir_y: self.dir_y,
        }
    }
}

pub struct ServerGameLoop {
    pub tick: u64,
    pub tick_rate_hz: f32,
    pub player_radius: f32,
    pub projectiles: Vec<ServerProjectile>,
    pub next_projectile_id: u32,
    pub last_shot_ticks: HashMap<u32, u64>,
}

impl Default for ServerGameLoop {
    fn default() -> Self {
        Self {
            tick: 0,
            tick_rate_hz: 60.0,
            player_radius: 0.3,
            projectiles: Vec::new(),
            next_projectile_id: 1,
            last_shot_ticks: HashMap::new(),
        }
    }
}

impl ServerGameLoop {
    pub fn new(tick_rate_hz: f32) -> Self {
        Self {
            tick: 0,
            tick_rate_hz,
            player_radius: 0.3,
            projectiles: Vec::new(),
            next_projectile_id: 1,
            last_shot_ticks: HashMap::new(),
        }
    }

    /// Process a single server tick (60Hz), applying authoritative movement,
    /// wall collision updates, and weapon firing for active client input packets.
    pub fn process_input(
        &mut self,
        sessions: &mut SessionManager,
        addr: SocketAddr,
        packet: ClientPacket,
        map: &MazeMap,
        _dt: f32,
    ) {
        if let ClientPacket::Input {
            move_x,
            move_y,
            yaw,
            shoot,
            ..
        } = packet
        {
            let (player_id, px, py, is_alive) = match sessions.get_session_mut(&addr) {
                Some(session) => {
                    if !session.is_alive {
                        return;
                    }
                    session.yaw = yaw;

                    if move_x != 0.0 || move_y != 0.0 {
                        let len_sq = move_x * move_x + move_y * move_y;
                        let max_displacement = 0.5;
                        let (valid_dx, valid_dy) = if len_sq > max_displacement * max_displacement {
                            let scale = max_displacement / len_sq.sqrt();
                            (move_x * scale, move_y * scale)
                        } else {
                            (move_x, move_y)
                        };

                        let (new_x, new_y) = move_and_slide(
                            map,
                            session.x,
                            session.y,
                            valid_dx,
                            valid_dy,
                            self.player_radius,
                        );

                        session.x = new_x;
                        session.y = new_y;
                    }
                    (session.id, session.x, session.y, session.is_alive)
                }
                None => return,
            };

            // Handle shooting with weapon cooldown (~12 ticks / 200ms)
            if shoot && is_alive {
                let last_shot = self.last_shot_ticks.get(&player_id).copied().unwrap_or(0);
                if self.tick >= last_shot + 12 {
                    self.last_shot_ticks.insert(player_id, self.tick);

                    let proj_id = self.next_projectile_id;
                    self.next_projectile_id += 1;

                    let dir_x = yaw.cos();
                    let dir_y = yaw.sin();

                    // Offset projectile slightly forward from player center
                    let start_x = px + dir_x * (self.player_radius + 0.1);
                    let start_y = py + dir_y * (self.player_radius + 0.1);

                    self.projectiles.push(ServerProjectile {
                        id: proj_id,
                        owner_id: player_id,
                        x: start_x,
                        y: start_y,
                        dir_x,
                        dir_y,
                        speed: 10.0,
                        lifetime: 3.0,
                    });
                }
            }
        }
    }

    /// Advance active projectiles, checking wall collisions and lifetime decay.
    pub fn update_projectiles(&mut self, map: &MazeMap, dt: f32) {
        let mut active = Vec::new();

        for mut proj in self.projectiles.drain(..) {
            proj.lifetime -= dt;
            if proj.lifetime <= 0.0 {
                continue;
            }

            let next_x = proj.x + proj.dir_x * proj.speed * dt;
            let next_y = proj.y + proj.dir_y * proj.speed * dt;

            let grid_x = (next_x / map.tile_size).floor() as i32;
            let grid_y = (next_y / map.tile_size).floor() as i32;

            if grid_x < 0
                || grid_y < 0
                || grid_x >= map.width as i32
                || grid_y >= map.height as i32
                || map.is_wall(grid_x as usize, grid_y as usize)
            {
                // Destroy projectile on wall impact or map boundary
                continue;
            }

            proj.x = next_x;
            proj.y = next_y;
            active.push(proj);
        }

        self.projectiles = active;
    }

    pub fn active_projectile_states(&self) -> Vec<ProjectileState> {
        self.projectiles.iter().map(|p| p.to_state()).collect()
    }

    pub fn advance_tick(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }
}
