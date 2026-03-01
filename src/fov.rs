use bevy::prelude::*;

use crate::components::*;

/// Returns true if there is a clear line of sight from (x0,y0) to (x1,y1).
/// Walls at intermediate tiles block LOS, but the target tile itself is visible
/// (you can see the wall face).
fn has_line_of_sight(x0: i32, y0: i32, x1: i32, y1: i32, walls: &[(i32, i32)]) -> bool {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        // If we've reached the target, LOS is clear
        if x == x1 && y == y1 {
            return true;
        }

        // Check if current intermediate tile is a wall (blocks LOS)
        if (x != x0 || y != y0) && walls.contains(&(x, y)) {
            return false;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn update_fog_of_war(
    player_query: Query<&GridPos, With<Player>>,
    wall_query: Query<(&GridPos, &Tags), With<Blocking>>,
    mut fog_map: ResMut<FogMap>,
) {
    let Ok(player_pos) = player_query.single() else {
        return;
    };
    let px = player_pos.0.x;
    let py = player_pos.0.y;

    // Collect wall positions (Stone-tagged blocking entities)
    let walls: Vec<(i32, i32)> = wall_query
        .iter()
        .filter(|(_, tags)| tags.0.contains(&Tag::Stone))
        .map(|(gp, _)| (gp.0.x, gp.0.y))
        .collect();

    // Demote all Visible to Explored
    fog_map.begin_update();

    // Mark player tile visible
    fog_map.mark_visible(px, py);

    // Check each tile within radius 5
    let radius_sq = 25; // 5*5
    for ty in 0..12i32 {
        for tx in 0..12i32 {
            let dx = tx - px;
            let dy = ty - py;
            if dx * dx + dy * dy > radius_sq {
                continue;
            }

            if has_line_of_sight(px, py, tx, ty, &walls) {
                fog_map.mark_visible(tx, ty);
            }
        }
    }
}
