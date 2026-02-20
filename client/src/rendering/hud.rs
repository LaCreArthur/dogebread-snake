use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

use super::overlays::get_snake_color_name;
use super::*;

/// Spawn HUD elements
pub fn spawn_ui(mut commands: Commands) {
    // Alive count (top-left)
    commands.spawn((
        Text::new("much alive: 0 / 0"),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(DOGE_GOLD),
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
        TextColor(Color::srgb(0.75, 0.60, 0.30)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(20.0),
            ..default()
        },
        TimerText,
    ));

    // Minimap (bottom-right)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(MINIMAP_MARGIN),
            right: Val::Px(MINIMAP_MARGIN),
            width: Val::Px(MINIMAP_SIZE),
            height: Val::Px(MINIMAP_SIZE),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        MinimapContainer,
    ));
}

/// Update alive count and player score display
pub fn update_alive_text(
    match_state: Res<MatchState>,
    mut text_query: Query<&mut Text, With<AliveText>>,
    player_query: Query<&Snake, With<PlayerControlled>>,
    mut player_stats: ResMut<PlayerStats>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    if let Ok(player) = player_query.single() {
        player_stats.score = player.score;
        player_stats.kills = player.kills;
    }
    **text = format!(
        "alive: {}/{}   score: {}   kills: {}",
        match_state.alive_count, match_state.total_snakes, player_stats.score, player_stats.kills
    );
}

/// Update minimap dots to show snake positions
pub fn update_minimap(
    mut commands: Commands,
    snake_query: Query<(&Snake, &SnakeColor, &SnakeId)>,
    mut dot_query: Query<(Entity, &MinimapDot, &mut Node, &mut BackgroundColor)>,
    minimap_query: Query<Entity, With<MinimapContainer>>,
    bounds: Res<ArenaBounds>,
) {
    let Ok(minimap_entity) = minimap_query.single() else {
        return;
    };

    let alive_ids: Vec<SnakeId> = snake_query
        .iter()
        .filter(|(s, _, _)| s.alive)
        .map(|(_, _, id)| *id)
        .collect();

    for (entity, dot, _, _) in &dot_query {
        if !alive_ids.contains(&dot.snake_id)
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }

    let scale_x = MINIMAP_SIZE / GRID_WIDTH as f32;
    let scale_y = MINIMAP_SIZE / GRID_HEIGHT as f32;

    let _bounds_x = bounds.min_x as f32 * scale_x;
    let _bounds_y = (GRID_HEIGHT - bounds.max_y) as f32 * scale_y;

    for (snake, color, id) in &snake_query {
        if !snake.alive {
            continue;
        }

        let head = snake.head();
        let mx = head.x as f32 * scale_x;
        let my = (GRID_HEIGHT - 1 - head.y) as f32 * scale_y;

        let existing = dot_query.iter_mut().find(|(_, d, _, _)| d.snake_id == *id);

        if let Some((_, _, mut node, mut bg_color)) = existing {
            node.left = Val::Px(mx);
            node.top = Val::Px(my);
            bg_color.0 = color.head;
        } else {
            let dot_entity = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(mx),
                        top: Val::Px(my),
                        width: Val::Px(MINIMAP_DOT),
                        height: Val::Px(MINIMAP_DOT),
                        ..default()
                    },
                    BackgroundColor(color.head),
                    MinimapDot { snake_id: *id },
                ))
                .id();
            commands.entity(minimap_entity).add_child(dot_entity);
        }
    }
}

/// Show/hide spectating message based on player alive status
pub fn update_spectating(
    mut commands: Commands,
    player_query: Query<&Snake, With<PlayerControlled>>,
    spectate_query: Query<(&Snake, &SnakeId), Without<PlayerControlled>>,
    mut existing: Query<(Entity, &Children), With<SpectatingText>>,
    mut text_query: Query<&mut Text, Without<SpectatingText>>,
    state: Res<State<GameState>>,
) {
    let player_dead = player_query.single().map(|s| !s.alive).unwrap_or(true);

    let show = *state.get() == GameState::Playing && player_dead;

    if show {
        let target = spectate_query
            .iter()
            .filter(|(s, _)| s.alive)
            .max_by_key(|(s, _)| s.score);

        let message = if let Some((_, id)) = target {
            format!("spectating {} doge", get_snake_color_name(id.0))
        } else {
            "spectating".to_string()
        };

        if let Ok((_entity, children)) = existing.single_mut() {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = message.clone();
                }
            }
        } else {
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(MINIMAP_MARGIN + MINIMAP_SIZE + 10.0),
                        right: Val::Px(MINIMAP_MARGIN),
                        ..default()
                    },
                    SpectatingText,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(message),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.91, 0.69, 0.29, 0.7)),
                    ));
                });
        }
    } else {
        for (entity, _) in &existing {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}
