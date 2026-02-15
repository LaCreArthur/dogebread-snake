use bevy::prelude::*;
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

/// AI personality affects decision-making
enum AiPersonality {
    Hungry,     // Prioritizes food
    Cautious,   // Stays far from walls and snakes
    Aggressive, // Tries to get near other snake heads (cut them off)
}

fn personality_for(id: u32) -> AiPersonality {
    match id % 3 {
        0 => AiPersonality::Hungry,
        1 => AiPersonality::Cautious,
        _ => AiPersonality::Aggressive,
    }
}

/// AI with varied personalities: avoid obstacles, chase objectives
pub fn ai_tick(
    mut ai_query: Query<(&mut Snake, &SnakeId), With<AiControlled>>,
    non_ai_snakes: Query<(&Snake, &SnakeId), Without<AiControlled>>,
    food_query: Query<&Food>,
    bounds: Res<ArenaBounds>,
) {
    // Collect all snake data for collision avoidance
    let mut snake_data: Vec<(SnakeId, Vec<GridPos>, GridPos)> = non_ai_snakes
        .iter()
        .filter(|(s, _)| s.alive)
        .map(|(s, id)| (*id, s.segments.iter().copied().collect(), s.head()))
        .collect();

    for (s, id) in ai_query.iter() {
        if s.alive {
            snake_data.push((*id, s.segments.iter().copied().collect(), s.head()));
        }
    }

    let foods: Vec<GridPos> = food_query.iter().map(|f| f.pos).collect();

    for (mut snake, my_id) in &mut ai_query {
        if !snake.alive {
            continue;
        }

        let head = snake.head();
        let personality = personality_for(my_id.0);

        let mut best_dir = snake.direction;
        let mut best_score = i32::MIN;

        for dir in Direction::ALL {
            if dir == snake.direction.opposite() {
                continue;
            }

            let delta = dir.delta();
            let next = GridPos::new(head.x + delta.x, head.y + delta.y);

            if !bounds.contains(next) {
                continue;
            }

            // Check collision with any snake body
            let mut blocked = false;
            for (sid, segments, _) in &snake_data {
                if *sid == *my_id {
                    if segments.iter().skip(1).any(|s| *s == next) {
                        blocked = true;
                        break;
                    }
                } else if segments.contains(&next) {
                    blocked = true;
                    break;
                }
            }
            if blocked {
                continue;
            }

            // 2-step look-ahead: check if next position has at least 1 safe exit
            let mut has_exit = false;
            for exit_dir in Direction::ALL {
                if exit_dir == dir.opposite() {
                    continue;
                }
                let exit_delta = exit_dir.delta();
                let exit_pos = GridPos::new(next.x + exit_delta.x, next.y + exit_delta.y);
                if bounds.contains(exit_pos) {
                    let exit_blocked = snake_data.iter().any(|(sid, segs, _)| {
                        if *sid == *my_id {
                            segs.iter().skip(1).any(|s| *s == exit_pos)
                        } else {
                            segs.contains(&exit_pos)
                        }
                    });
                    if !exit_blocked {
                        has_exit = true;
                        break;
                    }
                }
            }
            if !has_exit {
                continue; // Dead end — avoid
            }

            let mut score = 0i32;

            // Base: distance to nearest food
            if let Some(nearest_food) = foods.iter().min_by_key(|f| f.distance(next)) {
                let food_score = 100 - nearest_food.distance(next);
                score += match personality {
                    AiPersonality::Hungry => food_score * 2,
                    AiPersonality::Cautious => food_score,
                    AiPersonality::Aggressive => food_score / 2,
                };
            }

            // Wall distance
            let wall_dist = bounds.wall_distance(next);
            score += match personality {
                AiPersonality::Cautious => wall_dist * 5,
                _ => wall_dist * 2,
            };

            // Aggressive: bonus for being near other snake heads (try to cut them off)
            if matches!(personality, AiPersonality::Aggressive) {
                for (sid, _, other_head) in &snake_data {
                    if *sid == *my_id {
                        continue;
                    }
                    let dist = next.distance(*other_head);
                    if dist < 8 {
                        score += (8 - dist) * 4; // Bonus for being near enemy heads
                    }
                }
            }

            // Cautious: penalty for being near any snake
            if matches!(personality, AiPersonality::Cautious) {
                for (sid, _, other_head) in &snake_data {
                    if *sid == *my_id {
                        continue;
                    }
                    let dist = next.distance(*other_head);
                    if dist < 6 {
                        score -= (6 - dist) * 3;
                    }
                }
            }

            // Slight preference for keeping current direction
            if dir == snake.direction {
                score += 3;
            }

            if score > best_score {
                best_score = score;
                best_dir = dir;
            }
        }

        snake.set_direction(best_dir);
    }
}
