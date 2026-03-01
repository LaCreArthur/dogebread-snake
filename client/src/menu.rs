use bevy::prelude::*;
use shared::game::*;

use crate::rendering;

// ── Player name resource ───────────────────────────────────────────────

/// The name the player chose before the match.
#[derive(Resource, Default, Clone)]
pub struct PlayerName {
    pub name: String,
}

impl PlayerName {
    /// Returns the display name — falls back to "Player" if empty.
    pub fn display(&self) -> &str {
        if self.name.trim().is_empty() {
            "Player"
        } else {
            self.name.trim()
        }
    }
}

// ── LocalStorage helpers (WASM) ────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub(crate) fn load_name_from_storage() -> Option<String> {
    let result = js_sys::eval("window.localStorage.getItem('dogebread_player_name')").ok()?;
    result.as_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn load_name_from_storage() -> Option<String> {
    None
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn save_name_to_storage(name: &str) {
    let escaped = name.replace('\\', "\\\\").replace('\'', "\\'");
    let js = format!("window.localStorage.setItem('dogebread_player_name', '{}')", escaped);
    let _ = js_sys::eval(&js);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn save_name_to_storage(_name: &str) {}

// ── Button markers ─────────────────────────────────────────────────────

#[derive(Component)]
pub(crate) struct PlayButton;

#[derive(Component)]
pub(crate) struct LeaderboardButton;

#[derive(Component)]
pub(crate) struct HomeButton;

#[derive(Component)]
pub(crate) struct PlayAgainButton;

// ── Name entry screen markers ──────────────────────────────────────────

#[derive(Component)]
pub(crate) struct NameEntryScreen;

/// Marker for the text node that shows the current name being typed
#[derive(Component)]
pub(crate) struct NameInputText;

/// Marker for the GO! button on the name entry screen
#[derive(Component)]
pub(crate) struct StartGameButton;

// ── Home screen marker ─────────────────────────────────────────────────

#[derive(Component)]
pub(crate) struct HomeScreen;

// ── Leaderboard screen marker ──────────────────────────────────────────

#[derive(Component)]
pub(crate) struct LeaderboardScreen;

// ── Leaderboard data ───────────────────────────────────────────────────

#[derive(Clone)]
pub struct LeaderboardEntry {
    pub winner_name: String,
    pub winner_color: Color,
    pub total_kills: u32,
    pub match_number: u32,
    pub player_rank: u32,
    pub player_score: u32,
    pub player_kills: u32,
}

#[derive(Resource, Default)]
pub struct LeaderboardData {
    pub entries: Vec<LeaderboardEntry>,
    pub match_counter: u32,
}

impl LeaderboardData {
    pub fn add_entry(&mut self, entry: LeaderboardEntry) {
        self.match_counter += 1;
        self.entries.push(entry);
        // Keep last 20 entries
        if self.entries.len() > 20 {
            self.entries.remove(0);
        }
    }
}

// ── Constants ──────────────────────────────────────────────────────────

const BTN_NORMAL: Color = Color::srgb(0.20, 0.18, 0.30);
const BTN_HOVERED: Color = Color::srgb(0.30, 0.26, 0.42);
const BTN_PRESSED: Color = Color::srgb(0.91, 0.69, 0.29);

// ── Home Screen ────────────────────────────────────────────────────────

pub fn show_home(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            HomeScreen,
        ))
        .with_children(|parent| {
            // Title: DOGE BREAD
            parent.spawn((
                Text::new("DOGE BREAD"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(rendering::DOGE_GOLD),
            ));

            // Subtitle
            parent.spawn((
                Text::new("such snake   very battle   wow"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.6, 0.4)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // PLAY button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(BTN_NORMAL),
                    BorderColor::all(rendering::DOGE_GOLD),
                    PlayButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("PLAY"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.90, 0.80)),
                    ));
                });

            // LEADERBOARD button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(BTN_NORMAL),
                    BorderColor::all(Color::srgb(0.5, 0.45, 0.35)),
                    LeaderboardButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("LEADERBOARD"),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.75, 0.70, 0.60)),
                    ));
                });
        });
}

pub fn hide_home(mut commands: Commands, query: Query<Entity, With<HomeScreen>>) {
    for entity in &query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

// ── Leaderboard Screen ─────────────────────────────────────────────────

pub fn show_leaderboard(mut commands: Commands, leaderboard: Res<LeaderboardData>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            LeaderboardScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("LEADERBOARD"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(rendering::DOGE_GOLD),
            ));

            parent.spawn((
                Text::new("much history   very stats"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.55, 0.40)),
            ));

            // Entries container
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        row_gap: Val::Px(4.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        max_height: Val::Px(400.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.6)),
                ))
                .with_children(|entries_parent| {
                    if leaderboard.entries.is_empty() {
                        entries_parent.spawn((
                            Text::new("no matches yet! such empty!"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        ));
                    } else {
                        // Header
                        entries_parent.spawn((
                            Text::new("  #   WINNER             YOU            BONKS"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.45, 0.35)),
                        ));

                        // Show entries in reverse order (most recent first)
                        for entry in leaderboard.entries.iter().rev().take(10) {
                            let row_text = format!(
                                " {:>2}   {:<16} #{} ({} pts, {} kills)   {} bonks",
                                entry.match_number,
                                entry.winner_name,
                                entry.player_rank,
                                entry.player_score,
                                entry.player_kills,
                                entry.total_kills,
                            );

                            entries_parent.spawn((
                                Text::new(row_text),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(entry.winner_color),
                            ));
                        }
                    }
                });

            // Spacer
            parent.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

            // HOME button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(BTN_NORMAL),
                    BorderColor::all(rendering::DOGE_GOLD),
                    HomeButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("HOME"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.90, 0.80)),
                    ));
                });
        });
}

pub fn hide_leaderboard(mut commands: Commands, query: Query<Entity, With<LeaderboardScreen>>) {
    for entity in &query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

// ── Button interaction system (global) ─────────────────────────────────

#[allow(clippy::type_complexity)]
pub fn button_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(BTN_PRESSED);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(BTN_HOVERED);
            }
            Interaction::None => {
                *color = BackgroundColor(BTN_NORMAL);
            }
        }
    }
}

// ── Home button handlers ───────────────────────────────────────────────

pub fn home_play_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Go to NameEntry first so player can set their name
            next_state.set(GameState::NameEntry);
        }
    }
}

pub fn home_leaderboard_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LeaderboardButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Leaderboard);
        }
    }
}

// ── Leaderboard button handlers ────────────────────────────────────────

pub fn leaderboard_home_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<HomeButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Home);
        }
    }
}

// ── Game Over button handlers ──────────────────────────────────────────

pub fn gameover_play_again_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayAgainButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::WaitingToStart);
        }
    }
}

pub fn gameover_home_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<HomeButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Home);
        }
    }
}

// ── Save match to leaderboard (on enter GameOver) ──────────────────────

pub fn save_match_to_leaderboard(
    mut leaderboard: ResMut<LeaderboardData>,
    snake_query: Query<(&Snake, &SnakeColor, &SnakeId)>,
    player_name: Res<PlayerName>,
) {
    let winner = snake_query.iter().find(|(s, _, _)| s.alive);
    let winner_name = if let Some((_, _, id)) = winner {
        if id.0 == 0 {
            player_name.display().to_string()
        } else {
            rendering::get_snake_color_name(id.0).to_string()
        }
    } else {
        "Draw".to_string()
    };
    let winner_color = if let Some((_, color, _)) = winner {
        color.head
    } else {
        Color::srgb(0.5, 0.5, 0.5)
    };

    let total_kills: u32 = snake_query.iter().map(|(s, _, _)| s.kills).sum();

    // Find player rank
    let mut rankings: Vec<(u32, u32, bool, u32)> = snake_query
        .iter()
        .map(|(s, _, id)| (id.0, s.score, s.alive, s.kills))
        .collect();
    rankings.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then(b.1.cmp(&a.1))
            .then(b.3.cmp(&a.3))
    });
    let player_rank = rankings
        .iter()
        .position(|(id, _, _, _)| *id == 0)
        .map(|i| i as u32 + 1)
        .unwrap_or(0);

    let (player_score, player_kills) = snake_query
        .iter()
        .find(|(_, _, id)| id.0 == 0)
        .map(|(s, _, _)| (s.score, s.kills))
        .unwrap_or((0, 0));

    let match_number = leaderboard.match_counter + 1;

    leaderboard.add_entry(LeaderboardEntry {
        winner_name,
        winner_color,
        total_kills,
        match_number,
        player_rank,
        player_score,
        player_kills,
    });
}

// ── Name entry screen ─────────────────────────────────────────────────

/// Show the name entry screen. Loads stored name from localStorage if available.
pub fn show_name_entry(mut commands: Commands, mut player_name: ResMut<PlayerName>) {
    // Pre-fill from localStorage
    if let Some(stored) = load_name_from_storage()
        && !stored.trim().is_empty()
    {
        player_name.name = stored;
    }

    let cursor_text = if player_name.name.is_empty() {
        "_".to_string()
    } else {
        format!("{}_", player_name.name)
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(22.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.88)),
            NameEntryScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("ENTER YOUR NAME"),
                TextFont { font_size: 44.0, ..default() },
                TextColor(rendering::DOGE_GOLD),
            ));

            // Doge subtitle
            parent.spawn((
                Text::new("such identity   very player   wow"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.6, 0.55, 0.40)),
            ));

            // Input box
            parent
                .spawn((
                    Node {
                        width: Val::Px(340.0),
                        height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.16, 0.95)),
                    BorderColor::all(rendering::DOGE_GOLD),
                ))
                .with_children(|input_parent| {
                    input_parent.spawn((
                        Text::new(cursor_text),
                        TextFont { font_size: 28.0, ..default() },
                        TextColor(Color::srgb(0.95, 0.90, 0.80)),
                        NameInputText,
                    ));
                });

            // Hint
            parent.spawn((
                Text::new("type name, press Enter or GO!   (max 16 chars)"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.45, 0.40, 0.30)),
            ));

            // GO! button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(BTN_NORMAL),
                    BorderColor::all(rendering::DOGE_GOLD),
                    StartGameButton,
                ))
                .with_children(|btn_parent| {
                    btn_parent.spawn((
                        Text::new("GO!"),
                        TextFont { font_size: 32.0, ..default() },
                        TextColor(Color::srgb(0.95, 0.90, 0.80)),
                    ));
                });
        });
}

pub fn hide_name_entry(mut commands: Commands, query: Query<Entity, With<NameEntryScreen>>) {
    for entity in &query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

/// Helper: finalise the player name and transition to WaitingToStart
fn confirm_name(player_name: &mut PlayerName, next_state: &mut NextState<GameState>) {
    let trimmed = player_name.name.trim().to_string();
    player_name.name = if trimmed.is_empty() {
        "Player".to_string()
    } else {
        trimmed
    };
    save_name_to_storage(&player_name.name);
    next_state.set(GameState::WaitingToStart);
}

/// Handle keyboard input on the name entry screen
pub fn name_entry_input(
    mut keyboard_reader: MessageReader<bevy::input::keyboard::KeyboardInput>,
    mut player_name: ResMut<PlayerName>,
    mut text_query: Query<&mut Text, With<NameInputText>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use bevy::input::keyboard::Key;

    let mut changed = false;

    for keyboard_input in keyboard_reader.read() {
        if !keyboard_input.state.is_pressed() {
            continue;
        }
        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Enter, _) => {
                confirm_name(&mut player_name, &mut next_state);
                return;
            }
            (Key::Backspace, _) => {
                if player_name.name.pop().is_some() {
                    changed = true;
                }
            }
            (_, Some(text)) => {
                if player_name.name.len() < 16 {
                    for ch in text.chars() {
                        if !ch.is_ascii_control()
                            && (ch.is_alphanumeric() || matches!(ch, '_' | '-' | ' ' | '.'))
                        {
                            player_name.name.push(ch);
                            changed = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if changed {
        let display = if player_name.name.is_empty() {
            "_".to_string()
        } else {
            format!("{}_", player_name.name)
        };
        for mut text in &mut text_query {
            **text = display.clone();
        }
    }
}

/// GO! button on name entry screen
pub fn name_entry_start_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
    mut player_name: ResMut<PlayerName>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            confirm_name(&mut player_name, &mut next_state);
        }
    }
}

// ── Cleanup match entities ─────────────────────────────────────────────

/// Despawn all game entities (snakes, food, sprites, effects)
#[allow(clippy::too_many_arguments)]
pub fn cleanup_match_entities(
    mut commands: Commands,
    snake_query: Query<Entity, With<Snake>>,
    food_query: Query<Entity, With<Food>>,
    segment_query: Query<Entity, With<rendering::SnakeSegmentSprite>>,
    food_sprite_query: Query<Entity, With<rendering::FoodSprite>>,
    overlay_query: Query<Entity, With<rendering::GameOverOverlay>>,
    floating_text_query: Query<Entity, With<crate::effects::FloatingText>>,
    particle_query: Query<Entity, With<crate::effects::DeathParticle>>,
    eat_particle_query: Query<Entity, With<crate::effects::EatParticle>>,
    trail_particle_query: Query<Entity, With<crate::effects::TrailParticle>>,
    speed_text_query: Query<Entity, With<crate::effects::SpeedUpText>>,
    spectating_query: Query<Entity, With<rendering::SpectatingText>>,
    kill_feed_query: Query<Entity, With<rendering::KillFeedEntry>>,
    minimap_dot_query: Query<Entity, With<rendering::MinimapDot>>,
) {
    for entity in snake_query.iter()
        .chain(food_query.iter())
        .chain(segment_query.iter())
        .chain(food_sprite_query.iter())
        .chain(overlay_query.iter())
        .chain(floating_text_query.iter())
        .chain(particle_query.iter())
        .chain(eat_particle_query.iter())
        .chain(trail_particle_query.iter())
        .chain(speed_text_query.iter())
        .chain(spectating_query.iter())
        .chain(kill_feed_query.iter())
        .chain(minimap_dot_query.iter())
    {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }

    commands.remove_resource::<crate::match_lifecycle::CountdownTimer>();
    commands.remove_resource::<rendering::GameOverAnimation>();
}

/// Reset game resources to default state
#[allow(clippy::too_many_arguments)]
pub fn cleanup_match_resources(
    mut shake: ResMut<rendering::ScreenShake>,
    mut bounds: ResMut<ArenaBounds>,
    mut match_state: ResMut<MatchState>,
    mut tick: ResMut<crate::GameTick>,
    mut match_timer: ResMut<crate::game_systems::MatchTimer>,
    mut player_stats: ResMut<rendering::PlayerStats>,
    mut shrink_timer: ResMut<crate::game_systems::ArenaShrinkTimer>,
    mut speed_timer: ResMut<crate::game_systems::SpeedTimer>,
    mut warning: ResMut<rendering::ShrinkWarning>,
) {
    shake.intensity = 0.0;
    *bounds = ArenaBounds::default();
    match_state.alive_count = 0;
    match_state.total_snakes = 0;
    tick.timer.set_duration(std::time::Duration::from_secs_f32(
        shared::constants::TICK_INTERVAL as f32,
    ));
    tick.tick_count = 0;
    match_timer.elapsed = 0.0;
    player_stats.score = 0;
    player_stats.kills = 0;
    shrink_timer.timer.reset();
    speed_timer.timer.reset();
    warning.active = false;
}
