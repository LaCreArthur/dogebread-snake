use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

use super::*;

/// Spawn the grid background
pub fn spawn_grid(mut commands: Commands) {
    commands.spawn(Camera2d);

    let cell_visual = CELL_SIZE - 1.0;
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let pos = GridPos::new(x, y);
            let is_border = x == 0 || y == 0 || x == GRID_WIDTH - 1 || y == GRID_HEIGHT - 1;
            let color = if is_border {
                COLOR_WALL
            } else if (x + y) % 2 == 0 {
                COLOR_GRID_A
            } else {
                COLOR_GRID_B
            };
            commands.spawn((
                Sprite::from_color(color, Vec2::splat(cell_visual)),
                Transform::from_translation(pos.to_world().extend(0.0)),
                GridCell { pos },
            ));
        }
    }
}

/// Update grid cell colors based on current arena bounds, with shrink warning flash
pub fn update_grid_cells(
    bounds: Res<ArenaBounds>,
    time: Res<Time>,
    warning: Res<ShrinkWarning>,
    mut cell_query: Query<(&GridCell, &mut Sprite)>,
) {
    if !bounds.is_changed() && !warning.active {
        return;
    }

    let default = ArenaBounds::default();
    let has_shrunk = bounds.min_x > default.min_x;

    let blink = if warning.active {
        let t = (time.elapsed_secs() * 8.0).sin();
        t * 0.5 + 0.5
    } else {
        0.0
    };

    for (cell, mut sprite) in &mut cell_query {
        let pos = cell.pos;
        let is_outer_border = pos.x == 0 || pos.y == 0 || pos.x == GRID_WIDTH - 1 || pos.y == GRID_HEIGHT - 1;

        if is_outer_border || !bounds.contains(pos) {
            sprite.color = COLOR_WALL;
        } else if has_shrunk && bounds.wall_distance(pos) <= 1 {
            if warning.active {
                sprite.color = lerp_color(COLOR_DANGER, COLOR_DANGER_BRIGHT, blink);
            } else {
                sprite.color = COLOR_DANGER;
            }
        } else if warning.active && bounds.wall_distance(pos) <= 2 {
            let warn_color = lerp_color(
                if (pos.x + pos.y) % 2 == 0 {
                    COLOR_GRID_A
                } else {
                    COLOR_GRID_B
                },
                COLOR_DANGER,
                blink * 0.5,
            );
            sprite.color = warn_color;
        } else if (pos.x + pos.y) % 2 == 0 {
            sprite.color = COLOR_GRID_A;
        } else {
            sprite.color = COLOR_GRID_B;
        }
    }
}

/// Linearly interpolate between two sRGB colors
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}
