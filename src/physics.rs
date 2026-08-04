use crate::maze::MazeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    pub distance: f32,
    pub hit_x: f32,
    pub hit_y: f32,
    pub grid_x: usize,
    pub grid_y: usize,
    pub is_vertical_side: bool,
}

/// Check and resolve player movement collision against wall tiles in the map.
/// Uses sliding response so player glides along walls smoothly.
pub fn move_and_slide(
    map: &MazeMap,
    current_x: f32,
    current_y: f32,
    dx: f32,
    dy: f32,
    radius: f32,
) -> (f32, f32) {
    // Try X movement first
    let mut new_x = current_x + dx;
    if check_wall_collision(map, new_x, current_y, radius) {
        new_x = current_x; // Collision on X axis, cancel X movement
    }

    // Try Y movement next
    let mut new_y = current_y + dy;
    if check_wall_collision(map, new_x, new_y, radius) {
        new_y = current_y; // Collision on Y axis, cancel Y movement
    }

    (new_x, new_y)
}

/// Check if a circular bounding box at (x, y) with given radius intersects any wall tile.
pub fn check_wall_collision(map: &MazeMap, x: f32, y: f32, radius: f32) -> bool {
    let min_gx = ((x - radius) / map.tile_size).floor() as isize;
    let max_gx = ((x + radius) / map.tile_size).floor() as isize;
    let min_gy = ((y - radius) / map.tile_size).floor() as isize;
    let max_gy = ((y + radius) / map.tile_size).floor() as isize;

    for gy in min_gy..=max_gy {
        for gx in min_gx..=max_gx {
            if gx < 0 || gy < 0 || gx as usize >= map.width || gy as usize >= map.height {
                return true; // Out of bounds is treated as wall
            }
            if map.is_wall(gx as usize, gy as usize) {
                // Check circle vs AABB (grid cell box)
                let wall_left = gx as f32 * map.tile_size;
                let wall_right = (gx as f32 + 1.0) * map.tile_size;
                let wall_top = gy as f32 * map.tile_size;
                let wall_bottom = (gy as f32 + 1.0) * map.tile_size;

                let closest_x = x.clamp(wall_left, wall_right);
                let closest_y = y.clamp(wall_top, wall_bottom);

                let dist_x = x - closest_x;
                let dist_y = y - closest_y;
                let dist_sq = dist_x * dist_x + dist_y * dist_y;

                if dist_sq < radius * radius {
                    return true;
                }
            }
        }
    }
    false
}

/// Cast a ray from (ox, oy) in direction `angle_rad` using DDA algorithm until hitting a wall.
pub fn cast_ray(map: &MazeMap, ox: f32, oy: f32, angle_rad: f32, max_dist: f32) -> Option<RayHit> {
    let ray_dir_x = angle_rad.cos();
    let ray_dir_y = angle_rad.sin();

    let mut map_x = (ox / map.tile_size).floor() as isize;
    let mut map_y = (oy / map.tile_size).floor() as isize;

    let delta_dist_x = if ray_dir_x.abs() < 1e-6 {
        1e30
    } else {
        (1.0 / ray_dir_x).abs()
    };
    let delta_dist_y = if ray_dir_y.abs() < 1e-6 {
        1e30
    } else {
        (1.0 / ray_dir_y).abs()
    };

    let step_x: isize;
    let mut side_dist_x: f32;
    if ray_dir_x < 0.0 {
        step_x = -1;
        side_dist_x = (ox / map.tile_size - map_x as f32) * delta_dist_x;
    } else {
        step_x = 1;
        side_dist_x = (map_x as f32 + 1.0 - ox / map.tile_size) * delta_dist_x;
    }

    let step_y: isize;
    let mut side_dist_y: f32;
    if ray_dir_y < 0.0 {
        step_y = -1;
        side_dist_y = (oy / map.tile_size - map_y as f32) * delta_dist_y;
    } else {
        step_y = 1;
        side_dist_y = (map_y as f32 + 1.0 - oy / map.tile_size) * delta_dist_y;
    }

    let mut hit = false;
    let mut side = false; // false = X side (vertical wall face), true = Y side (horizontal wall face)

    let mut total_dist = 0.0;
    while !hit && total_dist < max_dist {
        if side_dist_x < side_dist_y {
            total_dist = side_dist_x;
            side_dist_x += delta_dist_x;
            map_x += step_x;
            side = false;
        } else {
            total_dist = side_dist_y;
            side_dist_y += delta_dist_y;
            map_y += step_y;
            side = true;
        }

        if map_x < 0 || map_y < 0 || map_x as usize >= map.width || map_y as usize >= map.height {
            break;
        }

        if map.is_wall(map_x as usize, map_y as usize) {
            hit = true;
        }
    }

    if hit {
        let distance = total_dist * map.tile_size;
        let hit_x = ox + ray_dir_x * distance;
        let hit_y = oy + ray_dir_y * distance;
        Some(RayHit {
            distance,
            hit_x,
            hit_y,
            grid_x: map_x as usize,
            grid_y: map_y as usize,
            is_vertical_side: !side,
        })
    } else {
        None
    }
}
