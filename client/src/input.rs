use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

/// Read keyboard input and queue direction changes on the player's snake
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut snake_query: Query<&mut Snake, With<PlayerControlled>>,
) {
    let Ok(mut snake) = snake_query.single_mut() else {
        return;
    };

    if !snake.alive {
        return;
    }

    let new_dir = if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW)
    {
        Some(Direction::Up)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        Some(Direction::Down)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        Some(Direction::Left)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        Some(Direction::Right)
    } else {
        None
    };

    if let Some(dir) = new_dir {
        snake.set_direction(dir);
    }
}

/// Simple AI: avoid walls and other snakes, chase food
pub fn ai_tick(
    mut ai_query: Query<(&mut Snake, &SnakeId), With<AiControlled>>,
    non_ai_snakes: Query<(&Snake, &SnakeId), Without<AiControlled>>,
    food_query: Query<&Food>,
) {
    // Collect segments from all snakes for collision avoidance
    // First collect from non-AI snakes (separate query)
    let mut snake_data: Vec<(SnakeId, Vec<GridPos>)> = non_ai_snakes
        .iter()
        .filter(|(s, _)| s.alive)
        .map(|(s, id)| (*id, s.segments.iter().copied().collect()))
        .collect();

    // Also collect from AI snakes (same query, readonly access via iter)
    for (s, id) in ai_query.iter() {
        if s.alive {
            snake_data.push((*id, s.segments.iter().copied().collect()));
        }
    }

    let foods: Vec<GridPos> = food_query.iter().map(|f| f.pos).collect();

    for (mut snake, my_id) in &mut ai_query {
        if !snake.alive {
            continue;
        }

        let head = snake.head();

        // Score each direction
        let mut best_dir = snake.direction;
        let mut best_score = i32::MIN;

        for dir in Direction::ALL {
            // Can't reverse
            if dir == snake.direction.opposite() {
                continue;
            }

            let delta = dir.delta();
            let next = GridPos::new(head.x + delta.x, head.y + delta.y);

            // Wall = death
            if !next.in_bounds() {
                continue;
            }

            // Check collision with any snake body
            let mut blocked = false;
            for (sid, segments) in &snake_data {
                if *sid == *my_id {
                    // Self: skip head (index 0), check body
                    if segments.iter().skip(1).any(|s| *s == next) {
                        blocked = true;
                        break;
                    }
                } else {
                    // Other snakes: any segment is danger
                    if segments.iter().any(|s| *s == next) {
                        blocked = true;
                        break;
                    }
                }
            }
            if blocked {
                continue;
            }

            // Score: prefer closer to food, prefer center-ish
            let mut score = 0i32;

            // Distance to nearest food (closer = better)
            if let Some(nearest_food) = foods.iter().min_by_key(|f| f.distance(next)) {
                score += 100 - nearest_food.distance(next);
            }

            // Prefer staying away from walls
            let wall_dist = next.x.min(next.y).min(GRID_WIDTH - 1 - next.x).min(GRID_HEIGHT - 1 - next.y);
            score += wall_dist * 2;

            // Slight preference for keeping current direction (less jittery)
            if dir == snake.direction {
                score += 5;
            }

            if score > best_score {
                best_score = score;
                best_dir = dir;
            }
        }

        snake.set_direction(best_dir);
    }
}
