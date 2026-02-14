use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

/// Marker for grid background cells
#[derive(Component)]
pub struct GridCell;

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

// Colors
const COLOR_GRID_A: Color = Color::srgb(0.15, 0.15, 0.18);
const COLOR_GRID_B: Color = Color::srgb(0.17, 0.17, 0.20);
const COLOR_FOOD: Color = Color::srgb(1.0, 0.85, 0.2);
const COLOR_DEAD: Color = Color::srgb(0.3, 0.3, 0.3);

/// Spawn the grid background
pub fn spawn_grid(mut commands: Commands) {
    commands.spawn(Camera2d);

    let cell_visual = CELL_SIZE - 1.0;
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            let pos = GridPos::new(x, y);
            let color = if (x + y) % 2 == 0 {
                COLOR_GRID_A
            } else {
                COLOR_GRID_B
            };
            commands.spawn((
                Sprite::from_color(color, Vec2::splat(cell_visual)),
                Transform::from_translation(pos.to_world().extend(0.0)),
                GridCell,
            ));
        }
    }
}

/// Spawn alive count UI
pub fn spawn_ui(mut commands: Commands) {
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
}

/// Sync all snake entities to their sprite representations
pub fn render_snakes(
    mut commands: Commands,
    snake_query: Query<(Entity, &Snake, &SnakeColor)>,
    mut segment_query: Query<(Entity, &mut Transform, &mut Sprite, &SnakeSegmentSprite)>,
) {
    // Build a set of alive snake entities for cleanup
    let alive_snake_entities: Vec<Entity> = snake_query.iter().map(|(e, _, _)| e).collect();

    // For each snake, update/spawn/despawn its segments
    for (snake_entity, snake, color) in &snake_query {
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

        let Ok((_, snake, color)) = snake_query.get(seg.snake_entity) else {
            continue;
        };

        if seg.index < snake.segments.len() {
            let pos = snake.segments[seg.index];
            transform.translation = pos.to_world().extend(if seg.index == 0 { 2.0 } else { 1.0 });

            sprite.color = if !snake.alive {
                COLOR_DEAD
            } else if seg.index == 0 {
                color.head
            } else {
                color.body
            };

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

/// Update alive count display
pub fn update_alive_text(
    match_state: Res<MatchState>,
    mut text_query: Query<&mut Text, With<AliveText>>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    **text = format!("Alive: {} / {}", match_state.alive_count, match_state.total_snakes);
}

/// Camera follows the player's snake head
pub fn camera_follow(
    player_query: Query<&Snake, With<PlayerControlled>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let Ok(mut cam_transform) = camera_query.single_mut() else {
        return;
    };

    let Ok(snake) = player_query.single() else {
        return;
    };

    let target = if snake.alive {
        snake.head().to_world()
    } else {
        Vec2::ZERO
    };

    let current = cam_transform.translation.truncate();
    let smoothed = current.lerp(target, 0.1);
    cam_transform.translation.x = smoothed.x;
    cam_transform.translation.y = smoothed.y;
}

/// Marker for game over UI overlay
#[derive(Component)]
pub struct GameOverOverlay;

/// Show game over screen
pub fn show_game_over(
    mut commands: Commands,
    existing: Query<Entity, With<GameOverOverlay>>,
    snake_query: Query<(&Snake, &SnakeColor, &SnakeId)>,
) {
    // Don't spawn if already exists
    if !existing.is_empty() {
        return;
    }

    // Find winner (last alive, or if all dead, none)
    let winner = snake_query.iter().find(|(s, _, _)| s.alive);
    let winner_text = if let Some((_snake, _color, id)) = winner {
        let name = match id.0 {
            0 => "You",
            n => &format!("Snake {}", n),
        };
        format!("{} wins!", name)
    } else {
        "Draw!".to_string()
    };

    // Spawn overlay
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GameOverOverlay,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: 48.0,
                ..default()
            },
            TextColor(COLOR_FOOD),
        ));
        parent.spawn((
            Text::new(winner_text),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new("Press SPACE to restart"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
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
