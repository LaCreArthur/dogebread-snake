use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

// Re-export effects functions used by lib.rs via rendering:: call sites
pub use crate::effects::{spawn_score_popup, spawn_death_particles, spawn_eat_particles};

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

/// Marker for the minimap container
#[derive(Component)]
pub struct MinimapContainer;

/// Minimap dot representing a snake
#[derive(Component)]
pub struct MinimapDot {
    pub snake_id: SnakeId,
}

// FloatingText and DeathParticle moved to effects.rs

/// Countdown text displayed during 3-2-1-GO phase
#[derive(Component)]
pub struct CountdownText;

/// Kill feed entry in the top-right corner
#[derive(Component)]
pub struct KillFeedEntry {
    pub timer: Timer,
}

/// Screen shake resource — set intensity to trigger, decays each frame
#[derive(Resource)]
pub struct ScreenShake {
    pub intensity: f32,
    pub decay: f32,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            decay: 0.9,
        }
    }
}

/// Shrink warning — flashes danger zone cells before arena shrinks
#[derive(Resource, Default)]
pub struct ShrinkWarning {
    pub active: bool,
}

const MINIMAP_SIZE: f32 = 150.0;
const MINIMAP_DOT: f32 = 5.0;
const MINIMAP_MARGIN: f32 = 10.0;

// Doge-inspired color palette
const DOGE_GOLD: Color = Color::srgb(0.91, 0.69, 0.29); // #e8b04b
const COLOR_GRID_A: Color = Color::srgb(0.12, 0.12, 0.20);
const COLOR_GRID_B: Color = Color::srgb(0.14, 0.14, 0.22);
const COLOR_WALL: Color = Color::srgb(0.58, 0.45, 0.20); // golden-brown border
const COLOR_DANGER: Color = Color::srgb(0.85, 0.35, 0.15); // orange-red danger zone
const COLOR_DANGER_BRIGHT: Color = Color::srgb(1.0, 0.55, 0.25); // brighter warning flash
const COLOR_FOOD: Color = Color::srgb(0.95, 0.73, 0.20); // golden coin


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
            right: Val::Px(10.0),
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
                if let Ok(mut ec) = commands.get_entity(*entity) {
                    ec.despawn();
                }
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
                    (1.0 - t) * ((t * 12.0).sin() * 0.5 + 0.5)
                } else {
                    0.5
                };
                sprite.color = Color::srgba(0.4, 0.4, 0.4, blink);
            } else if seg.index == 0 {
                // Head: full brightness
                sprite.color = color.head;
            } else {
                // Body: 80% brightness of body color
                let c = color.body.to_srgba();
                sprite.color = Color::srgb(c.red * 0.8, c.green * 0.8, c.blue * 0.8);
            }

            // Head is full cell size (bubble head), body has more gap
            if seg.index == 0 {
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE));
            } else {
                sprite.custom_size = Some(Vec2::splat(CELL_SIZE - 3.0));
            }
        }
    }
}

/// Render food with pulsing animation
pub fn render_food(
    mut commands: Commands,
    time: Res<Time>,
    food_query: Query<&Food>,
    mut food_sprite_query: Query<(Entity, &mut Transform, &mut Sprite), With<FoodSprite>>,
) {
    let foods: Vec<&Food> = food_query.iter().collect();

    if foods.is_empty() {
        for (entity, _, _) in &food_sprite_query {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
        return;
    }

    let elapsed = time.elapsed_secs();
    let base_size = CELL_SIZE - 5.0;

    let mut existing: Vec<_> = food_sprite_query.iter_mut().collect();
    for (i, food) in foods.iter().enumerate() {
        let phase_hash = (food.pos.x * 7 + food.pos.y * 13) as f32;
        let pulse = (elapsed * 3.0 + phase_hash).sin();
        let scale = 1.0 + pulse * 0.15; // oscillate between 0.85x and 1.15x
        let pulsed_size = base_size * scale;

        if i < existing.len() {
            existing[i].1.translation = food.pos.to_world().extend(0.5);
            existing[i].2.custom_size = Some(Vec2::splat(pulsed_size));
        } else {
            commands.spawn((
                Sprite::from_color(COLOR_FOOD, Vec2::splat(pulsed_size)),
                Transform::from_translation(food.pos.to_world().extend(0.5)),
                FoodSprite,
            ));
        }
    }

    // Remove excess
    for (entity, _, _) in existing.iter().skip(foods.len()) {
        if let Ok(mut ec) = commands.get_entity(*entity) {
            ec.despawn();
        }
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
            "much alive: {} / {}  •  wow score: {}  •  kills: {}",
            match_state.alive_count, match_state.total_snakes,
            player.score, player.kills
        );
    } else {
        **text = format!("much alive: {} / {}", match_state.alive_count, match_state.total_snakes);
    }
}

/// Camera follows the player's snake head during gameplay, centers during menus.
/// Adds screen shake offset when ScreenShake intensity > 0.
/// Zooms out as arena shrinks.
pub fn camera_follow(
    player_query: Query<&Snake, With<PlayerControlled>>,
    spectate_query: Query<(&Snake, &SnakeId), Without<PlayerControlled>>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    state: Res<State<GameState>>,
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    bounds: Res<ArenaBounds>,
) {
    let Ok((mut cam_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let target = match state.get() {
        GameState::WaitingToStart | GameState::Countdown | GameState::GameOver => {
            Vec2::ZERO
        }
        GameState::Playing => {
            if let Ok(snake) = player_query.single() {
                if snake.alive {
                    snake.head().to_world()
                } else {
                    // Auto-spectate: follow the strongest alive snake
                    spectate_query.iter()
                        .filter(|(s, _)| s.alive)
                        .max_by_key(|(s, _)| s.score)
                        .map(|(s, _)| s.head().to_world())
                        .unwrap_or(Vec2::ZERO)
                }
            } else {
                Vec2::ZERO
            }
        }
    };

    let current = cam_transform.translation.truncate();
    let smoothing = 1.0 - (-5.0 * time.delta_secs()).exp();
    let mut smoothed = current.lerp(target, smoothing);

    // Apply screen shake
    if shake.intensity > 0.1 {
        let t = time.elapsed_secs();
        let offset_x = (t * 137.0).sin() * shake.intensity;
        let offset_y = (t * 251.0).cos() * shake.intensity;
        smoothed.x += offset_x;
        smoothed.y += offset_y;
        shake.intensity *= shake.decay;
    } else {
        shake.intensity = 0.0;
    }

    cam_transform.translation.x = smoothed.x;
    cam_transform.translation.y = smoothed.y;

    // Arena zoom: zoom out as arena shrinks, dramatic zoom on game over
    if let Projection::Orthographic(ortho) = projection.as_mut() {
        let bounds_width = (bounds.max_x - bounds.min_x) as f32;
        let arena_fraction = bounds_width / (GRID_WIDTH as f32 - 2.0);
        let mut target_scale = 1.0 + (1.0 - arena_fraction) * 0.5;

        // Dramatic zoom-out on game over
        if *state.get() == GameState::GameOver {
            target_scale = 2.0;
        }

        let scale_smoothing = 1.0 - (-3.0 * time.delta_secs()).exp();
        ortho.scale = ortho.scale + (target_scale - ortho.scale) * scale_smoothing;
    }
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

    // Remove dots for snakes that no longer exist
    let alive_ids: Vec<SnakeId> = snake_query.iter()
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

    // Minimap coordinate scale
    let scale_x = MINIMAP_SIZE / GRID_WIDTH as f32;
    let scale_y = MINIMAP_SIZE / GRID_HEIGHT as f32;

    // Draw arena bounds outline as a border hint
    let bounds_x = bounds.min_x as f32 * scale_x;
    let bounds_y = (GRID_HEIGHT - bounds.max_y) as f32 * scale_y;
    let bounds_w = (bounds.max_x - bounds.min_x) as f32 * scale_x;
    let bounds_h = (bounds.max_y - bounds.min_y) as f32 * scale_y;
    let _ = (bounds_x, bounds_y, bounds_w, bounds_h); // future: draw arena outline

    for (snake, color, id) in &snake_query {
        if !snake.alive {
            continue;
        }

        let head = snake.head();
        // Convert grid pos to minimap pos (y is flipped: grid y=0 is bottom, UI y=0 is top)
        let mx = head.x as f32 * scale_x;
        let my = (GRID_HEIGHT - 1 - head.y) as f32 * scale_y;

        // Check if dot already exists for this snake
        let existing = dot_query.iter_mut().find(|(_, d, _, _)| d.snake_id == *id);

        if let Some((_, _, mut node, mut bg_color)) = existing {
            node.left = Val::Px(mx);
            node.top = Val::Px(my);
            bg_color.0 = color.head;
        } else {
            // Spawn new dot as child of minimap
            let dot_entity = commands.spawn((
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
            )).id();
            commands.entity(minimap_entity).add_child(dot_entity);
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
    // Run every frame when warning is active (for blinking), otherwise only when bounds change
    if !bounds.is_changed() && !warning.active {
        return;
    }

    let default = ArenaBounds::default();
    let has_shrunk = bounds.min_x > default.min_x;

    // Compute blink factor for warning flash
    let blink = if warning.active {
        let t = (time.elapsed_secs() * 8.0).sin();
        t * 0.5 + 0.5 // 0.0 to 1.0
    } else {
        0.0
    };

    for (cell, mut sprite) in &mut cell_query {
        let pos = cell.pos;
        let is_outer_border = pos.x == 0 || pos.y == 0 || pos.x == GRID_WIDTH - 1 || pos.y == GRID_HEIGHT - 1;

        if is_outer_border || !bounds.contains(pos) {
            sprite.color = COLOR_WALL;
        } else if has_shrunk && bounds.wall_distance(pos) <= 1 {
            // Danger zone: blink between normal and bright when warning active
            if warning.active {
                sprite.color = lerp_color(COLOR_DANGER, COLOR_DANGER_BRIGHT, blink);
            } else {
                sprite.color = COLOR_DANGER;
            }
        } else if warning.active && bounds.wall_distance(pos) <= 2 {
            // About-to-become-danger cells flash when warning active
            let warn_color = lerp_color(
                if (pos.x + pos.y) % 2 == 0 { COLOR_GRID_A } else { COLOR_GRID_B },
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

/// Marker for spectating text
#[derive(Component)]
pub struct SpectatingText;

/// Show/hide spectating message based on player alive status
pub fn update_spectating(
    mut commands: Commands,
    player_query: Query<&Snake, With<PlayerControlled>>,
    spectate_query: Query<(&Snake, &SnakeId), Without<PlayerControlled>>,
    mut existing: Query<(Entity, &Children), With<SpectatingText>>,
    mut text_query: Query<&mut Text, Without<SpectatingText>>,
    state: Res<State<GameState>>,
) {
    let player_dead = player_query
        .single()
        .map(|s| !s.alive)
        .unwrap_or(true);

    let show = *state.get() == GameState::Playing && player_dead;

    if show {
        // Find strongest alive snake
        let target = spectate_query.iter()
            .filter(|(s, _)| s.alive)
            .max_by_key(|(s, _)| s.score);

        let message = if let Some((_, id)) = target {
            format!("much spectate • following {} doge", get_snake_color_name(id.0))
        } else {
            "much spectate • wow".to_string()
        };

        // Update existing text or spawn new
        if let Ok((_entity, children)) = existing.single_mut() {
            // Update text content
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = message.clone();
                }
            }
        } else {
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(MINIMAP_MARGIN + MINIMAP_SIZE + 10.0),
                    right: Val::Px(MINIMAP_MARGIN),
                    ..default()
                },
                SpectatingText,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(message),
                    TextFont { font_size: 28.0, ..default() },
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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        StartPrompt,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("DOGEBREAD SNAKE"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(DOGE_GOLD),
        ));
        parent.spawn((
            Text::new("such snake • very battle • wow"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.6, 0.4)),
        ));
        parent.spawn((
            Text::new("press arrow key • much begin"),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

/// Remove start prompt
pub fn hide_start_prompt(
    mut commands: Commands,
    query: Query<Entity, With<StartPrompt>>,
) {
    for entity in &query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}

/// Marker for game over UI overlay
#[derive(Component)]
pub struct GameOverOverlay;

/// Marker for the game-over title text
#[derive(Component)]
pub(crate) struct GameOverTitle;

/// Marker for the winner announcement text
#[derive(Component)]
pub(crate) struct GameOverWinner;

/// Marker for the rankings container (holds individual ranking entries)
#[derive(Component)]
pub(crate) struct GameOverRankings;

/// Marker for the restart prompt
#[derive(Component)]
pub(crate) struct GameOverRestart;

/// Phased animation controller for the game over screen
#[derive(Resource)]
pub struct GameOverAnimation {
    timer: Timer,
    pub(crate) phase: u8,
    /// Cached data so animate system doesn't re-query snakes
    player_won: bool,
    player_lost: bool,
    winner_text: String,
    rankings: Vec<RankingEntry>,
    total_kills: u32,
}

struct RankingEntry {
    name: String,
    score: u32,
    kills: u32,
    alive: bool,
    color: Color,
    is_player: bool,
}

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

/// Show game over screen — spawns the overlay container and inserts the animation resource.
/// The actual content is revealed progressively by `animate_game_over`.
pub fn show_game_over(
    mut commands: Commands,
    existing: Query<Entity, With<GameOverOverlay>>,
    snake_query: Query<(&Snake, &SnakeColor, &SnakeId)>,
) {
    if !existing.is_empty() {
        return;
    }

    // --- Collect data for the animation resource ---
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

    // Build sorted rankings: alive first, then by score desc, then kills desc
    let mut rankings: Vec<RankingEntry> = snake_query
        .iter()
        .map(|(s, color, id)| RankingEntry {
            name: if id.0 == 0 { "You".to_string() } else { snake_color_name(id.0).to_string() },
            score: s.score,
            kills: s.kills,
            alive: s.alive,
            color: color.head,
            is_player: id.0 == 0,
        })
        .collect();
    rankings.sort_by(|a, b| {
        b.alive.cmp(&a.alive)
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

    // --- Spawn overlay container (starts semi-transparent, darkens via animation) ---
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
/// Phase 0 (0-0.6s): Overlay fades in, "GAME OVER" title appears
/// Phase 1 (0.6-1.6s): Winner announcement fades in
/// Phase 2 (1.6-2.6s): Rankings appear
/// Phase 3 (2.6s+): Restart prompt + stats
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

    // Fade in overlay background during phase 0
    if anim.phase == 0 {
        let frac = anim.timer.fraction();
        let alpha = (frac * 1.2).min(0.8); // fade to 0.8
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
            // Phase 0 complete → spawn title, start phase 1
            let Ok((overlay_entity, mut bg)) = overlay_query.single_mut() else { return; };
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.8);

            if title_query.is_empty() {
                let title = commands.spawn((
                    Text::new("such game over • wow"),
                    TextFont { font_size: 52.0, ..default() },
                    TextColor(DOGE_GOLD),
                    GameOverTitle,
                )).id();
                commands.entity(overlay_entity).add_child(title);
            }

            anim.phase = 1;
            anim.timer = Timer::from_seconds(1.0, TimerMode::Once);
        }
        1 => {
            // Phase 1 complete → spawn winner announcement, start phase 2
            let Ok((overlay_entity, _)) = overlay_query.single_mut() else { return; };

            if winner_query.is_empty() {
                let (text, color, font_size) = if anim.player_won {
                    (anim.winner_text.clone(), Color::srgb(1.0, 0.84, 0.0), 44.0) // bright gold
                } else if anim.player_lost {
                    (anim.winner_text.clone(), Color::srgb(0.65, 0.65, 0.72), 32.0) // muted
                } else {
                    (anim.winner_text.clone(), Color::srgb(0.85, 0.75, 0.5), 32.0)
                };

                let winner = commands.spawn((
                    Text::new(text),
                    TextFont { font_size, ..default() },
                    TextColor(color),
                    GameOverWinner,
                )).id();
                commands.entity(overlay_entity).add_child(winner);
            }

            anim.phase = 2;
            anim.timer = Timer::from_seconds(1.0, TimerMode::Once);
        }
        2 => {
            // Phase 2 complete → spawn rankings, start phase 3
            let Ok((overlay_entity, _)) = overlay_query.single_mut() else { return; };

            if rankings_query.is_empty() {
                // Rankings container
                let rankings_container = commands.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        row_gap: Val::Px(4.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.6)),
                    GameOverRankings,
                )).id();
                commands.entity(overlay_entity).add_child(rankings_container);

                // Header
                let header = commands.spawn((
                    Text::new("── such rankings • very final ──"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.55, 0.4)),
                )).id();
                commands.entity(rankings_container).add_child(header);

                // Individual ranking rows
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

                    let you_marker = if entry.is_player { "  ← YOU" } else { "" };
                    let crown = if i == 0 && entry.alive { " ♛" } else { "" };
                    let status = if entry.alive { " ★" } else { "" };

                    let row_text = format!(
                        "{}{}  {}{}  •  {} noms  •  {} bonks{}",
                        rank_label, crown, entry.name, you_marker, entry.score, entry.kills, status,
                    );

                    // Color: snake's actual color for name, brighter for player
                    let text_color = if entry.is_player {
                        // Player row: bright gold
                        Color::srgb(1.0, 0.84, 0.0)
                    } else if i == 0 && entry.alive {
                        // Winner (non-player): their snake color at full brightness
                        entry.color
                    } else {
                        // Others: their snake color, slightly muted
                        let c = entry.color.to_srgba();
                        Color::srgb(c.red * 0.8, c.green * 0.8, c.blue * 0.8)
                    };

                    let font_size = if entry.is_player || (i == 0 && entry.alive) {
                        19.0
                    } else {
                        16.0
                    };

                    let row = commands.spawn((
                        Text::new(row_text),
                        TextFont { font_size, ..default() },
                        TextColor(text_color),
                    )).id();
                    commands.entity(rankings_container).add_child(row);
                }

                // Stats summary
                let stats_text = format!(
                    "total bonks: {}  •  much carnage  •  wow",
                    anim.total_kills,
                );
                let stats = commands.spawn((
                    Text::new(stats_text),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                )).id();
                commands.entity(rankings_container).add_child(stats);
            }

            anim.phase = 3;
            anim.timer = Timer::from_seconds(1.0, TimerMode::Once);
        }
        3 => {
            // Phase 3 complete → show restart prompt
            let Ok((overlay_entity, _)) = overlay_query.single_mut() else { return; };

            if restart_query.is_empty() {
                let restart = commands.spawn((
                    Text::new("Press SPACE for much restart"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    GameOverRestart,
                )).id();
                commands.entity(overlay_entity).add_child(restart);
            }

            // Stay at phase 4 (done)
            anim.phase = 4;
            anim.timer = Timer::from_seconds(999.0, TimerMode::Once);
        }
        _ => {
            // Animation complete, nothing to do
        }
    }
}

/// Remove game over overlay
pub fn hide_game_over(
    mut commands: Commands,
    overlay_query: Query<Entity, With<GameOverOverlay>>,
) {
    for entity in &overlay_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    commands.remove_resource::<GameOverAnimation>();
}

// animate_floating_text, animate_death_particles, spawn_score_popup,
// spawn_death_particles moved to effects.rs

/// Spawn the countdown overlay (centered large text)
pub fn spawn_countdown_overlay(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        CountdownText,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("3"),
            TextFont { font_size: 120.0, ..default() },
            TextColor(DOGE_GOLD),
        ));
    });
}

/// Remove countdown overlay
pub fn despawn_countdown_overlay(
    mut commands: Commands,
    query: Query<Entity, With<CountdownText>>,
) {
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
        TextFont { font_size: 16.0, ..default() },
        TextColor(color),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            right: Val::Px(10.0),
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
        let alpha = if frac > 0.67 {
            1.0 - ((frac - 0.67) / 0.33)
        } else {
            1.0
        };
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

/// Set window properties
pub fn window_setup() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "DogeBread Snake".to_string(),
            resolution: (900, 780).into(),
            resizable: true,
            ..default()
        }),
        ..default()
    }
}
