use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

/// Grid background cell with its position
#[derive(Component)]
pub struct GridCell {
    pub pos: GridPos,
}

/// Links a sprite to its snake entity and segment index
#[derive(Component)]
pub struct SnakeSegmentSprite {
    pub snake_entity: Entity,
    pub index: usize,
}

/// Marker for food sprite
#[derive(Component)]
pub struct FoodSprite;

/// Marker for the alive count text
#[derive(Component)]
pub struct AliveText;

/// Marker for the match timer text
#[derive(Component)]
pub struct TimerText;

// Colors
const COLOR_GRID_A: Color = Color::srgb(0.15, 0.15, 0.18);
const COLOR_GRID_B: Color = Color::srgb(0.17, 0.17, 0.20);
const COLOR_WALL: Color = Color::srgb(0.35, 0.3, 0.2);
const COLOR_DANGER: Color = Color::srgb(0.5, 0.15, 0.1); // red-ish danger zone
const COLOR_FOOD: Color = Color::srgb(1.0, 0.85, 0.2);


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

/// Spawn HUD elements
pub fn spawn_ui(mut commands: Commands) {
    // Alive count (top-left)
    commands.spawn((
        Text::new("Alive: 0 / 0"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(COLOR_FOOD),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        AliveText,
    ));

    // Match timer (top-right)
    commands.spawn((
        Text::new("0:00"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        TimerText,
    ));
}

/// Sync all snake entities to their sprite representations
pub fn render_snakes(
    mut commands: Commands,
    snake_query: Query<(Entity, &Snake, &SnakeColor, Option<&DeathTimer>)>,
    mut segment_query: Query<(Entity, &mut Transform, &mut Sprite, &SnakeSegmentSprite)>,
) {
    // Build a set of alive snake entities for cleanup
    let alive_snake_entities: Vec<Entity> = snake_query.iter().map(|(e, _, _, _)| e).collect();

    // For each snake, update/spawn/despawn its segments
    for (snake_entity, snake, color, _) in &snake_query {
        // Count existing segments for this snake
        let existing: Vec<(Entity, usize)> = segment_query
            .iter()
            .filter(|(_, _, _, seg)| seg.snake_entity == snake_entity)
            .map(|(e, _, _, seg)| (e, seg.index))
            .collect();

        let needed = snake.segments.len();
        let existing_count = existing.len();

        // Spawn new segments
        if needed > existing_count {
            for i in existing_count..needed {
                let pos = snake.segments[i];
                commands.spawn((
                    Sprite::from_color(color.body, Vec2::splat(CELL_SIZE - 2.0)),
                    Transform::from_translation(pos.to_world().extend(1.0)),
                    SnakeSegmentSprite {
                        snake_entity,
                        index: i,
                    },
                ));
            }
        }

        // Remove excess segments
        if needed < existing_count {
            let mut sorted = existing.clone();
            sorted.sort_by_key(|(_, i)| *i);
            for (entity, _) in sorted.iter().rev().take(existing_count - needed) {
                commands.entity(*entity).despawn();
            }
        }
    }

    // Update positions and colors for all segment sprites
    for (_, mut transform, mut sprite, seg) in &mut segment_query {
        // Check if the parent snake still exists
        if !alive_snake_entities.contains(&seg.snake_entity) {
            continue;
        }

        let Ok((_, snake, color, death_timer)) = snake_query.get(seg.snake_entity) else {
            continue;
        };

        if seg.index < snake.segments.len() {
            let pos = snake.segments[seg.index];
            transform.translation = pos.to_world().extend(if seg.index == 0 { 2.0 } else { 1.0 });

            if !snake.alive {
                // Dead: blink between gray and transparent
                let blink = if let Some(dt) = death_timer {
                    let t = dt.timer.fraction();
                    // Fast blink that fades out
                    let alpha = (1.0 - t) * ((t * 12.0).sin() * 0.5 + 0.5);
                    alpha
                } else {
                    0.5
                };
                sprite.color = Color::srgba(0.4, 0.4, 0.4, blink);
            } else if seg.index == 0 {
                sprite.color = color.head;
            } else {
                sprite.color = color.body;
            }

            // Head is slightly larger
            if seg.index == 0 {
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE - 1.0));
            }
        }
    }
}

/// Render food
pub fn render_food(
    mut commands: Commands,
    food_query: Query<&Food>,
    mut food_sprite_query: Query<(Entity, &mut Transform), With<FoodSprite>>,
) {
    let foods: Vec<&Food> = food_query.iter().collect();

    if foods.is_empty() {
        for (entity, _) in &food_sprite_query {
            commands.entity(entity).despawn();
        }
        return;
    }

    let mut existing: Vec<_> = food_sprite_query.iter_mut().collect();
    for (i, food) in foods.iter().enumerate() {
        if i < existing.len() {
            existing[i].1.translation = food.pos.to_world().extend(0.5);
        } else {
            commands.spawn((
                Sprite::from_color(COLOR_FOOD, Vec2::splat(CELL_SIZE - 4.0)),
                Transform::from_translation(food.pos.to_world().extend(0.5)),
                FoodSprite,
            ));
        }
    }

    // Remove excess
    for (entity, _) in existing.iter().skip(foods.len()) {
        commands.entity(*entity).despawn();
    }
}

/// Update alive count and player score display
pub fn update_alive_text(
    match_state: Res<MatchState>,
    mut text_query: Query<&mut Text, With<AliveText>>,
    player_query: Query<&Snake, With<PlayerControlled>>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    if let Ok(player) = player_query.single() {
        **text = format!(
            "Alive: {} / {}  |  Score: {}  Kills: {}",
            match_state.alive_count, match_state.total_snakes,
            player.score, player.kills
        );
    } else {
        **text = format!("Alive: {} / {}", match_state.alive_count, match_state.total_snakes);
    }
}

/// Camera follows the player's snake head during gameplay, centers during menus
pub fn camera_follow(
    player_query: Query<&Snake, With<PlayerControlled>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    state: Res<State<GameState>>,
) {
    let Ok(mut cam_transform) = camera_query.single_mut() else {
        return;
    };

    let target = match state.get() {
        GameState::WaitingToStart | GameState::GameOver => {
            // Center on grid
            Vec2::ZERO
        }
        GameState::Playing => {
            if let Ok(snake) = player_query.single() {
                if snake.alive {
                    snake.head().to_world()
                } else {
                    Vec2::ZERO
                }
            } else {
                Vec2::ZERO
            }
        }
    };

    let current = cam_transform.translation.truncate();
    let smoothed = current.lerp(target, 0.08);
    cam_transform.translation.x = smoothed.x;
    cam_transform.translation.y = smoothed.y;
}

/// Update grid cell colors based on current arena bounds
pub fn update_grid_cells(
    bounds: Res<ArenaBounds>,
    mut cell_query: Query<(&GridCell, &mut Sprite)>,
) {
    if !bounds.is_changed() {
        return;
    }

    let default = ArenaBounds::default();
    let has_shrunk = bounds.min_x > default.min_x;

    for (cell, mut sprite) in &mut cell_query {
        let pos = cell.pos;
        let is_outer_border = pos.x == 0 || pos.y == 0 || pos.x == GRID_WIDTH - 1 || pos.y == GRID_HEIGHT - 1;

        if is_outer_border {
            sprite.color = COLOR_WALL;
        } else if !bounds.contains(pos) {
            // Outside current arena = wall
            sprite.color = COLOR_WALL;
        } else if has_shrunk && bounds.wall_distance(pos) <= 1 {
            // Danger zone: only show after arena has started shrinking
            sprite.color = COLOR_DANGER;
        } else if (pos.x + pos.y) % 2 == 0 {
            sprite.color = COLOR_GRID_A;
        } else {
            sprite.color = COLOR_GRID_B;
        }
    }
}

/// Marker for start prompt UI
#[derive(Component)]
pub struct StartPrompt;

/// Show "press arrow to start" prompt
pub fn show_start_prompt(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        StartPrompt,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("DOGEBREAD SNAKE"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(COLOR_FOOD),
        ));
        parent.spawn((
            Text::new("Press arrow key to start"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ));
    });
}

/// Remove start prompt
pub fn hide_start_prompt(
    mut commands: Commands,
    query: Query<Entity, With<StartPrompt>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Marker for game over UI overlay
#[derive(Component)]
pub struct GameOverOverlay;

fn snake_color_name(id: u32) -> &'static str {
    match id % 8 {
        0 => "Gold",
        1 => "Green",
        2 => "Red",
        3 => "Blue",
        4 => "Pink",
        5 => "Cyan",
        6 => "Orange",
        7 => "Lavender",
        _ => "???",
    }
}

/// Show game over screen with scores
pub fn show_game_over(
    mut commands: Commands,
    existing: Query<Entity, With<GameOverOverlay>>,
    snake_query: Query<(&Snake, &SnakeColor, &SnakeId)>,
) {
    if !existing.is_empty() {
        return;
    }

    // Find winner
    let winner = snake_query.iter().find(|(s, _, _)| s.alive);
    let winner_text = if let Some((_, _, id)) = winner {
        if id.0 == 0 {
            "You win!".to_string()
        } else {
            format!("{} snake wins!", snake_color_name(id.0))
        }
    } else {
        "Draw!".to_string()
    };

    // Build scoreboard sorted by score descending
    let mut scores: Vec<(u32, u32, u32, bool)> = snake_query
        .iter()
        .map(|(s, _, id)| (id.0, s.score, s.kills, s.alive))
        .collect();
    scores.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));

    let scoreboard: String = scores
        .iter()
        .map(|(id, score, kills, alive)| {
            let name = if *id == 0 { "You" } else { snake_color_name(*id) };
            let status = if *alive { " *" } else { "" };
            format!("{}  -  {} food, {} kills{}", name, score, kills, status)
        })
        .collect::<Vec<_>>()
        .join("\n");

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GameOverOverlay,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont { font_size: 48.0, ..default() },
            TextColor(COLOR_FOOD),
        ));
        parent.spawn((
            Text::new(winner_text),
            TextFont { font_size: 32.0, ..default() },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new(scoreboard),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
        parent.spawn((
            Text::new("Press SPACE to restart"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}

/// Remove game over overlay
pub fn hide_game_over(
    mut commands: Commands,
    overlay_query: Query<Entity, With<GameOverOverlay>>,
) {
    for entity in &overlay_query {
        commands.entity(entity).despawn();
    }
}

/// Set window properties
pub fn window_setup() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "DogeBread Snake".to_string(),
            resolution: (800, 700).into(),
            resizable: true,
            ..default()
        }),
        ..default()
    }
}
