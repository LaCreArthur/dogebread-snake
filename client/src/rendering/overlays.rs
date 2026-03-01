use bevy::prelude::*;
use shared::game::*;

use super::*;

fn snake_color_name(id: u32) -> &'static str {
    match id % 8 {
        0 => "Doge",
        1 => "Cheems",
        2 => "Bonk",
        3 => "Shibe",
        4 => "Floof",
        5 => "Bork",
        6 => "Snoot",
        7 => "Woofer",
        _ => "???",
    }
}

/// Get the color name for a snake ID (public for kill feed)
pub fn get_snake_color_name(id: u32) -> &'static str {
    snake_color_name(id)
}

/// Show "press arrow to start" prompt
pub fn show_start_prompt(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
            StartPrompt,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("DOGEBREAD SNAKE"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(DOGE_GOLD),
            ));
            parent.spawn((
                Text::new("such snake   very battle   wow"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.6, 0.4)),
            ));
            parent.spawn((
                Text::new("press arrow key to begin"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

/// Remove start prompt
pub fn hide_start_prompt(mut commands: Commands, query: Query<Entity, With<StartPrompt>>) {
    for entity in &query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

/// Show game over screen
pub fn show_game_over(
    mut commands: Commands,
    existing: Query<Entity, With<GameOverOverlay>>,
    snake_query: Query<(&Snake, &SnakeColor, &SnakeId)>,
) {
    if !existing.is_empty() {
        return;
    }

    let winner = snake_query.iter().find(|(s, _, _)| s.alive);
    let player_won = winner.map(|(_, _, id)| id.0 == 0).unwrap_or(false);
    let player_lost = !player_won;
    let winner_text = if let Some((_, _, id)) = winner {
        if id.0 == 0 {
            "VICTORY! very win! so champion! wow!".to_string()
        } else {
            format!("{} wins! much skill! so impress!", snake_color_name(id.0))
        }
    } else {
        "wow such draw! no survivors! very dead!".to_string()
    };

    let mut rankings: Vec<RankingEntry> = snake_query
        .iter()
        .map(|(s, color, id)| RankingEntry {
            name: if id.0 == 0 {
                "You".to_string()
            } else {
                snake_color_name(id.0).to_string()
            },
            score: s.score,
            kills: s.kills,
            alive: s.alive,
            color: color.head,
            is_player: id.0 == 0,
        })
        .collect();
    rankings.sort_by(|a, b| {
        b.alive
            .cmp(&a.alive)
            .then(b.score.cmp(&a.score))
            .then(b.kills.cmp(&a.kills))
    });

    let total_kills: u32 = rankings.iter().map(|r| r.kills).sum();

    commands.insert_resource(GameOverAnimation {
        timer: Timer::from_seconds(0.6, TimerMode::Once),
        phase: 0,
        player_won,
        player_lost,
        winner_text,
        rankings,
        total_kills,
    });

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        GameOverOverlay,
    ));
}

/// Phased reveal animation for the game over screen.
#[allow(clippy::too_many_arguments)]
pub fn animate_game_over(
    mut commands: Commands,
    time: Res<Time>,
    mut anim: ResMut<GameOverAnimation>,
    mut overlay_query: Query<(Entity, &mut BackgroundColor), With<GameOverOverlay>>,
    title_query: Query<&GameOverTitle>,
    winner_query: Query<&GameOverWinner>,
    rankings_query: Query<&GameOverRankings>,
    restart_query: Query<&GameOverRestart>,
) {
    anim.timer.tick(time.delta());

    if anim.phase == 0 {
        let frac = anim.timer.fraction();
        let alpha = (frac * 1.2).min(0.8);
        for (_, mut bg) in &mut overlay_query {
            bg.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
        }
    }

    if !anim.timer.just_finished() {
        return;
    }

    let current_phase = anim.phase;
    match current_phase {
        0 => {
            let Ok((overlay_entity, mut bg)) = overlay_query.single_mut() else {
                return;
            };
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.8);

            if title_query.is_empty() {
                let title = commands
                    .spawn((
                        Text::new("GAME OVER"),
                        TextFont {
                            font_size: 52.0,
                            ..default()
                        },
                        TextColor(DOGE_GOLD),
                        GameOverTitle,
                    ))
                    .id();
                commands.entity(overlay_entity).add_child(title);
            }

            anim.phase = 1;
            anim.timer = Timer::from_seconds(1.0, TimerMode::Once);
        }
        1 => {
            let Ok((overlay_entity, _)) = overlay_query.single_mut() else {
                return;
            };

            if winner_query.is_empty() {
                let (text, color, font_size) = if anim.player_won {
                    (anim.winner_text.clone(), Color::srgb(1.0, 0.84, 0.0), 44.0)
                } else if anim.player_lost {
                    (anim.winner_text.clone(), Color::srgb(0.65, 0.65, 0.72), 32.0)
                } else {
                    (anim.winner_text.clone(), Color::srgb(0.85, 0.75, 0.5), 32.0)
                };

                let winner = commands
                    .spawn((
                        Text::new(text),
                        TextFont { font_size, ..default() },
                        TextColor(color),
                        GameOverWinner,
                    ))
                    .id();
                commands.entity(overlay_entity).add_child(winner);
            }

            anim.phase = 2;
            anim.timer = Timer::from_seconds(1.0, TimerMode::Once);
        }
        2 => {
            let Ok((overlay_entity, _)) = overlay_query.single_mut() else {
                return;
            };

            if rankings_query.is_empty() {
                let rankings_container = commands
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Start,
                            row_gap: Val::Px(4.0),
                            padding: UiRect::all(Val::Px(16.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.6)),
                        GameOverRankings,
                    ))
                    .id();
                commands.entity(overlay_entity).add_child(rankings_container);

                let header = commands
                    .spawn((
                        Text::new("RANKINGS"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.55, 0.4)),
                    ))
                    .id();
                commands.entity(rankings_container).add_child(header);

                for (i, entry) in anim.rankings.iter().enumerate() {
                    let rank_label = match i {
                        0 => "1st",
                        1 => "2nd",
                        2 => "3rd",
                        _ => match i + 1 {
                            4 => "4th",
                            5 => "5th",
                            6 => "6th",
                            7 => "7th",
                            8 => "8th",
                            9 => "9th",
                            10 => "10th",
                            _ => "??",
                        },
                    };

                    let you_marker = if entry.is_player { "  <- YOU" } else { "" };
                    let crown = if i == 0 && entry.alive { " [W]" } else { "" };
                    let status = if entry.alive { " *" } else { "" };

                    let row_text = format!(
                        "{}{}  {}{}   {} noms   {} bonks{}",
                        rank_label, crown, entry.name, you_marker, entry.score, entry.kills, status,
                    );

                    let text_color = if entry.is_player {
                        Color::srgb(1.0, 0.84, 0.0)
                    } else if i == 0 && entry.alive {
                        entry.color
                    } else {
                        let c = entry.color.to_srgba();
                        Color::srgb(c.red * 0.8, c.green * 0.8, c.blue * 0.8)
                    };

                    let font_size = if entry.is_player || (i == 0 && entry.alive) {
                        19.0
                    } else {
                        16.0
                    };

                    let row = commands
                        .spawn((
                            Text::new(row_text),
                            TextFont { font_size, ..default() },
                            TextColor(text_color),
                        ))
                        .id();
                    commands.entity(rankings_container).add_child(row);
                }

                let stats_text = format!("total bonks: {}   much carnage", anim.total_kills,);
                let stats = commands
                    .spawn((
                        Text::new(stats_text),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ))
                    .id();
                commands.entity(rankings_container).add_child(stats);
            }

            anim.phase = 3;
            anim.timer = Timer::from_seconds(1.0, TimerMode::Once);
        }
        3 => {
            let Ok((overlay_entity, _)) = overlay_query.single_mut() else {
                return;
            };

            if restart_query.is_empty() {
                // Button container
                let btn_container = commands
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(20.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        GameOverRestart,
                    ))
                    .id();
                commands.entity(overlay_entity).add_child(btn_container);

                // PLAY AGAIN button
                let play_again = commands
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.20, 0.18, 0.30)),
                        BorderColor::all(DOGE_GOLD),
                        crate::menu::PlayAgainButton,
                    ))
                    .id();
                commands.entity(btn_container).add_child(play_again);

                let play_text = commands
                    .spawn((
                        Text::new("PLAY AGAIN"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.90, 0.80)),
                    ))
                    .id();
                commands.entity(play_again).add_child(play_text);

                // HOME button
                let home = commands
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.20, 0.18, 0.30)),
                        BorderColor::all(Color::srgb(0.5, 0.45, 0.35)),
                        crate::menu::HomeButton,
                    ))
                    .id();
                commands.entity(btn_container).add_child(home);

                let home_text = commands
                    .spawn((
                        Text::new("HOME"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.75, 0.70, 0.60)),
                    ))
                    .id();
                commands.entity(home).add_child(home_text);

                // Also keep keyboard hint
                let hint = commands
                    .spawn((
                        Text::new("or press SPACE / R"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.45, 0.45, 0.45)),
                    ))
                    .id();
                commands.entity(overlay_entity).add_child(hint);
            }

            anim.phase = 4;
            anim.timer = Timer::from_seconds(999.0, TimerMode::Once);
        }
        _ => {}
    }
}

/// Remove game over overlay
pub fn hide_game_over(mut commands: Commands, overlay_query: Query<Entity, With<GameOverOverlay>>) {
    for entity in &overlay_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    commands.remove_resource::<GameOverAnimation>();
}

/// Spawn the countdown overlay
pub fn spawn_countdown_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            CountdownText,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("3"),
                TextFont {
                    font_size: 120.0,
                    ..default()
                },
                TextColor(DOGE_GOLD),
            ));
        });
}

/// Remove countdown overlay
pub fn despawn_countdown_overlay(mut commands: Commands, query: Query<Entity, With<CountdownText>>) {
    for entity in &query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

/// Spawn a kill feed entry
pub fn spawn_kill_feed_entry(commands: &mut Commands, message: String, color: Color) {
    commands.spawn((
        Text::new(message),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(color),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            right: Val::Px(20.0),
            ..default()
        },
        KillFeedEntry {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        },
    ));
}

/// Animate kill feed entries: fade out and despawn, reposition stack
pub fn animate_kill_feed(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut KillFeedEntry, &mut TextColor, &mut Node)>,
) {
    let mut entries: Vec<(Entity, f32)> = Vec::new();
    for (entity, mut entry, mut color, _) in &mut query {
        entry.timer.tick(time.delta());
        let frac = entry.timer.fraction();
        let alpha = if frac > 0.67 { 1.0 - ((frac - 0.67) / 0.33) } else { 1.0 };
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);

        if entry.timer.just_finished() {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        } else {
            entries.push((entity, entry.timer.remaining_secs()));
        }
    }

    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (i, (entity, _)) in entries.iter().enumerate() {
        if i >= 4 {
            if let Ok(mut ec) = commands.get_entity(*entity) {
                ec.despawn();
            }
            continue;
        }
        if let Ok((_, _, _, mut node)) = query.get_mut(*entity) {
            node.top = Val::Px(40.0 + i as f32 * 22.0);
        }
    }
}
