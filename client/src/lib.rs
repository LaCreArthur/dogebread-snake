mod audio;
mod effects;
mod input;
mod rendering;
mod ui;

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use shared::constants::*;
use shared::game::*;

const NUM_SNAKES: u32 = 10;
const NUM_FOOD: usize = 35;
const ARENA_SHRINK_INTERVAL: f32 = 12.0;
const SPEED_INCREASE_INTERVAL: f32 = 20.0;

#[derive(Resource)]
pub struct GameTick {
    timer: Timer,
}

#[derive(Resource)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        let seed = (js_sys::Date::now() * 1000.0) as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let seed = {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        };
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }

    fn range(&mut self, min: i32, max: i32) -> i32 {
        min + (self.next_u32() % (max - min) as u32) as i32
    }
}

/// Countdown resource: tracks the 3-2-1-GO! timer
#[derive(Resource)]
struct CountdownTimer {
    timer: Timer,
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    console_error_panic_hook::set_once();
    run();
}

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(rendering::window_setup()))
        .insert_resource(ClearColor(Color::srgb(0.10, 0.10, 0.18))) // DOGE_BACKGROUND
        .insert_resource(GameTick {
            timer: Timer::from_seconds(TICK_INTERVAL as f32, TimerMode::Repeating),
        })
        .insert_resource(SimpleRng::new())
        .insert_resource(MatchState::default())
        .insert_resource(ArenaBounds::default())
        .insert_resource(ArenaShrinkTimer {
            timer: Timer::from_seconds(ARENA_SHRINK_INTERVAL, TimerMode::Repeating),
        })
        .insert_resource(SpeedTimer {
            timer: Timer::from_seconds(SPEED_INCREASE_INTERVAL, TimerMode::Repeating),
        })
        .insert_resource(MatchTimer { elapsed: 0.0 })
        .insert_resource(ScreenshotTimer {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            enabled: std::env::var("AUTO_SCREENSHOT").is_ok(),
            counter: 0,
        })
        .insert_resource({
            let enabled = std::env::var("AUTO_TEST").is_ok();
            if enabled {
                std::fs::create_dir_all("test-output").ok();
                info!("[AUTO_TEST] mode enabled — capturing screenshots at game events");
            }
            AutoTestState {
                enabled,
                captured: HashSet::new(),
                prev_alive_count: 0,
                arena_shrunk: false,
                exit_timer: None,
            }
        })
        .insert_resource(rendering::ScreenShake::default())
        .insert_resource(rendering::ShrinkWarning::default())
        .insert_resource(effects::TrailSpawner {
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
        })
        .init_state::<GameState>()
        .add_systems(Startup, (rendering::spawn_grid, rendering::spawn_ui, spawn_match, audio::setup_audio))
        .add_systems(OnEnter(GameState::WaitingToStart), rendering::show_start_prompt)
        .add_systems(OnExit(GameState::WaitingToStart), rendering::hide_start_prompt)
        .add_systems(Update, wait_for_start.run_if(in_state(GameState::WaitingToStart)))
        // Countdown phase
        .add_systems(OnEnter(GameState::Countdown), rendering::spawn_countdown_overlay)
        .add_systems(OnExit(GameState::Countdown), rendering::despawn_countdown_overlay)
        .add_systems(Update, run_countdown.run_if(in_state(GameState::Countdown)))
        // Playing phase
        .add_systems(
            Update,
            (
                input::handle_input,
                input::ai_tick,
                game_tick.after(input::handle_input).after(input::ai_tick),
                arena_shrink.after(game_tick),
                speed_increase,
                track_match_time,
                effects::spawn_trail_particles,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), rendering::show_game_over)
        .add_systems(OnExit(GameState::GameOver), rendering::hide_game_over)
        .add_systems(Update, (
            restart_on_space,
            rendering::animate_game_over,
        ).run_if(in_state(GameState::GameOver)))
        .add_systems(Update, (
            rendering::render_snakes,
            rendering::render_food,
            rendering::update_alive_text,
            rendering::update_grid_cells,
            rendering::update_minimap,
            rendering::update_spectating,
            rendering::camera_follow,
            update_timer_text,
        ))
        .add_systems(Update, (
            cleanup_dead_snakes,
            handle_esc_quit,
            auto_screenshot,
            auto_test_system,
            rendering::animate_kill_feed,
        ))
        .add_systems(Update, (
            effects::animate_floating_text,
            effects::animate_death_particles,
            effects::animate_eat_particles,
            effects::animate_speed_up_text,
            effects::animate_trail_particles,
        ))
        .run();
}

fn spawn_positions() -> Vec<(i32, i32, Direction)> {
    vec![
        (10, 10, Direction::Right),
        (GRID_WIDTH - 11, 10, Direction::Up),
        (10, GRID_HEIGHT - 11, Direction::Right),
        (GRID_WIDTH - 11, GRID_HEIGHT - 11, Direction::Down),
        (GRID_WIDTH / 2, 10, Direction::Up),
        (10, GRID_HEIGHT / 2, Direction::Right),
        (GRID_WIDTH - 11, GRID_HEIGHT / 2, Direction::Left),
        (GRID_WIDTH / 2, GRID_HEIGHT - 11, Direction::Down),
        (GRID_WIDTH / 3, GRID_HEIGHT / 3, Direction::Right),
        (2 * GRID_WIDTH / 3, 2 * GRID_HEIGHT / 3, Direction::Left),
    ]
}

fn spawn_match(
    mut commands: Commands,
    mut rng: ResMut<SimpleRng>,
    mut match_state: ResMut<MatchState>,
) {
    let positions = spawn_positions();

    for i in 0..NUM_SNAKES {
        let (x, y, dir) = positions[i as usize];
        let snake = Snake::new(x, y, dir);
        let color = SnakeColor::palette(i);
        let id = SnakeId(i);

        if i == 0 {
            commands.spawn((snake, color, id, PlayerControlled));
        } else {
            commands.spawn((snake, color, id, AiControlled));
        }
    }

    for _ in 0..NUM_FOOD {
        let x = rng.range(2, GRID_WIDTH - 2);
        let y = rng.range(2, GRID_HEIGHT - 2);
        commands.spawn(Food::new(x, y));
    }

    match_state.total_snakes = NUM_SNAKES;
    match_state.alive_count = NUM_SNAKES;
}

#[allow(clippy::too_many_arguments)]
fn game_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut tick: ResMut<GameTick>,
    mut rng: ResMut<SimpleRng>,
    mut snake_query: Query<(Entity, &mut Snake, &SnakeId, &SnakeColor, Option<&PlayerControlled>)>,
    food_query: Query<(Entity, &Food)>,
    mut match_state: ResMut<MatchState>,
    mut next_state: ResMut<NextState<GameState>>,
    bounds: Res<ArenaBounds>,
    mut shake: ResMut<rendering::ScreenShake>,
    sfx: Option<Res<audio::SoundEffects>>,
) {
    tick.timer.tick(time.delta());
    if !tick.timer.just_finished() {
        return;
    }

    let mut new_heads: Vec<(Entity, SnakeId, GridPos)> = Vec::new();
    for (entity, mut snake, id, _, _) in &mut snake_query {
        if !snake.alive {
            continue;
        }
        let head = snake.step();
        new_heads.push((entity, *id, head));
    }

    let body_map: Vec<(SnakeId, Vec<GridPos>)> = snake_query
        .iter()
        .filter(|(_, s, _, _, _)| s.alive)
        .map(|(_, s, id, _, _)| {
            (*id, s.segments.iter().skip(1).copied().collect())
        })
        .collect();

    let mut kills: Vec<(Entity, Option<SnakeId>)> = Vec::new();

    for (entity, my_id, head) in &new_heads {
        if !bounds.contains(*head) {
            kills.push((*entity, None));
            continue;
        }

        if let Ok((_, snake, _, _, _)) = snake_query.get(*entity)
            && snake.self_collision()
        {
            kills.push((*entity, None));
            continue;
        }

        for (other_id, body_segments) in &body_map {
            if *other_id == *my_id {
                continue;
            }
            if body_segments.contains(head) {
                kills.push((*entity, Some(*other_id)));
                break;
            }
        }

        for (other_entity, other_id, other_head) in &new_heads {
            if *other_id == *my_id {
                continue;
            }
            if *head == *other_head {
                kills.push((*entity, None));
                kills.push((*other_entity, None));
            }
        }
    }

    let mut seen = Vec::new();
    let mut unique_kills = Vec::new();
    for (entity, killer) in &kills {
        if !seen.contains(entity) {
            seen.push(*entity);
            unique_kills.push((*entity, *killer));
        }
    }

    let kill_credits: Vec<SnakeId> = unique_kills
        .iter()
        .filter_map(|(_, killer)| *killer)
        .collect();

    for killer_id in &kill_credits {
        for (_, mut snake, id, _, _) in &mut snake_query {
            if *id == *killer_id {
                snake.kills += 1;
            }
        }
    }

    // Get player's current kill count for screen shake scaling
    let player_kills = snake_query
        .iter()
        .find(|(_, _, id, _, _)| id.0 == 0)
        .map(|(_, snake, _, _, _)| snake.kills)
        .unwrap_or(0);

    // Process deaths: trigger screen shake + death particles + kill feed
    for (entity, killer) in &unique_kills {
        if let Ok((_, mut snake, dead_id, color, _)) = snake_query.get_mut(*entity)
            && snake.alive
        {
            let death_pos = snake.head().to_world();
            snake.alive = false;
            commands.entity(*entity).insert(DeathTimer {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
            });

            // Screen shake on any death — stronger for player kills
            if let Some(killer_id) = killer {
                if killer_id.0 == 0 {
                    // Player kill: scale with kill count
                    shake.intensity = 8.0 + (player_kills as f32 * 2.0).min(12.0);
                } else {
                    shake.intensity = 8.0;
                }
            } else {
                shake.intensity = 8.0;
            }

            // Death explosion particles
            rendering::spawn_death_particles(
                &mut commands,
                death_pos,
                color.head,
                time.elapsed_secs(),
            );

            // Kill feed entry
            let dead_name = if dead_id.0 == 0 {
                "You".to_string()
            } else {
                rendering::get_snake_color_name(dead_id.0).to_string()
            };
            let (message, feed_color) = if let Some(killer_id) = killer {
                let killer_name = if killer_id.0 == 0 {
                    "You".to_string()
                } else {
                    rendering::get_snake_color_name(killer_id.0).to_string()
                };
                (
                    format!("{} bonked by {}! wow", dead_name, killer_name),
                    color.head,
                )
            } else {
                (format!("{} ded! much rip!", dead_name), color.head)
            };
            rendering::spawn_kill_feed_entry(&mut commands, message, feed_color);

            // Death sound
            if let Some(ref sfx) = sfx {
                audio::play_sfx(&mut commands, &sfx.death);
            }
        }
    }

    let alive = snake_query.iter().filter(|(_, s, _, _, _)| s.alive).count() as u32;
    match_state.alive_count = alive;

    if alive <= 1 && match_state.total_snakes > 1 {
        if let Some(ref sfx) = sfx {
            audio::play_sfx(&mut commands, &sfx.game_over);
        }
        next_state.set(GameState::GameOver);
    }

    for (_, snake, _, _, _) in &snake_query {
        if !snake.alive {
            continue;
        }
        let head = snake.head();
        for (food_entity, food) in &food_query {
            if food.pos == head
                && let Ok(mut ec) = commands.get_entity(food_entity)
            {
                ec.despawn();
            }
        }
    }

    let eaten_positions: Vec<GridPos> = snake_query
        .iter()
        .filter(|(_, s, _, _, _)| s.alive)
        .map(|(_, s, _, _, _)| s.head())
        .collect();

    let mut foods_eaten = 0;
    for (food_entity, food) in &food_query {
        if eaten_positions.contains(&food.pos) {
            if let Ok(mut ec) = commands.get_entity(food_entity) {
                ec.despawn();
            }
            foods_eaten += 1;

            for (_, mut snake, _, _, player) in &mut snake_query {
                if snake.alive && snake.head() == food.pos {
                    snake.grow_pending += 2;
                    snake.score += 1;

                    // Score popup + eat particles + sound only for the player's snake
                    if player.is_some() {
                        let eat_pos = food.pos.to_world();
                        rendering::spawn_score_popup(&mut commands, eat_pos);
                        rendering::spawn_eat_particles(&mut commands, eat_pos, time.elapsed_secs());
                        shake.intensity = 2.0;
                        if let Some(ref sfx) = sfx {
                            audio::play_sfx(&mut commands, &sfx.eat);
                        }
                    }
                }
            }
        }
    }

    let remaining_food = food_query.iter().count() - foods_eaten;
    let target_food = NUM_FOOD;
    if remaining_food < target_food {
        for _ in 0..(target_food - remaining_food) {
            let x = rng.range(bounds.min_x + 1, bounds.max_x - 1);
            let y = rng.range(bounds.min_y + 1, bounds.max_y - 1);
            commands.spawn(Food::new(x, y));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn restart_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    snake_query: Query<Entity, With<Snake>>,
    food_query: Query<Entity, With<Food>>,
    segment_query: Query<Entity, With<rendering::SnakeSegmentSprite>>,
    food_sprite_query: Query<Entity, With<rendering::FoodSprite>>,
    overlay_query: Query<Entity, With<rendering::GameOverOverlay>>,
    mut rng: ResMut<SimpleRng>,
    mut match_state: ResMut<MatchState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut bounds: ResMut<ArenaBounds>,
    mut tick: ResMut<GameTick>,
    mut match_timer: ResMut<MatchTimer>,
    floating_text_query: Query<Entity, With<effects::FloatingText>>,
    particle_query: Query<Entity, With<effects::DeathParticle>>,
    mut shake: ResMut<rendering::ScreenShake>,
) {
    let should_restart = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyR);

    if !should_restart {
        return;
    }

    for entity in &snake_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    for entity in &food_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    for entity in &segment_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    for entity in &food_sprite_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    for entity in &overlay_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    for entity in &floating_text_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    for entity in &particle_query {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    shake.intensity = 0.0;

    // Also clean up any kill feed entries and game over animation
    commands.remove_resource::<CountdownTimer>();
    commands.remove_resource::<rendering::GameOverAnimation>();

    let positions = spawn_positions();
    for i in 0..NUM_SNAKES {
        let (x, y, dir) = positions[i as usize];
        let snake = Snake::new(x, y, dir);
        let color = SnakeColor::palette(i);
        let id = SnakeId(i);

        if i == 0 {
            commands.spawn((snake, color, id, PlayerControlled));
        } else {
            commands.spawn((snake, color, id, AiControlled));
        }
    }

    for _ in 0..NUM_FOOD {
        let x = rng.range(2, GRID_WIDTH - 2);
        let y = rng.range(2, GRID_HEIGHT - 2);
        commands.spawn(Food::new(x, y));
    }

    match_state.total_snakes = NUM_SNAKES;
    match_state.alive_count = NUM_SNAKES;
    *bounds = ArenaBounds::default();
    tick.timer.set_duration(std::time::Duration::from_secs_f32(TICK_INTERVAL as f32));
    match_timer.elapsed = 0.0;
    next_state.set(GameState::WaitingToStart);
}

fn wait_for_start(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut snake_query: Query<&mut Snake, With<PlayerControlled>>,
    auto_test: Res<AutoTestState>,
) {
    let dir = if auto_test.enabled {
        // AUTO_TEST: skip waiting, start immediately
        Some(Direction::Right)
    } else if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
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

    if let Some(d) = dir {
        if let Ok(mut snake) = snake_query.single_mut() {
            snake.set_direction(d);
        }
        // Start countdown instead of going directly to Playing
        commands.insert_resource(CountdownTimer {
            timer: Timer::from_seconds(3.5, TimerMode::Once),
        });
        next_state.set(GameState::Countdown);
    }
}

/// Run the 3-2-1-GO! countdown, then transition to Playing
#[allow(clippy::too_many_arguments)]
fn run_countdown(
    mut commands: Commands,
    time: Res<Time>,
    mut countdown: ResMut<CountdownTimer>,
    mut next_state: ResMut<NextState<GameState>>,
    countdown_parent: Query<&Children, With<rendering::CountdownText>>,
    mut text_query: Query<&mut Text, Without<rendering::CountdownText>>,
    sfx: Option<Res<audio::SoundEffects>>,
    mut last_phase: Local<u8>,
) {
    countdown.timer.tick(time.delta());
    let elapsed = countdown.timer.elapsed_secs();

    // Determine what text to show: 3 (0-1s), 2 (1-2s), 1 (2-3s), GO! (3-3.5s)
    let (label, phase) = if elapsed < 1.0 {
        ("such 3", 1u8)
    } else if elapsed < 2.0 {
        ("very 2", 2)
    } else if elapsed < 3.0 {
        ("much 1", 3)
    } else {
        ("WOW GO!", 4)
    };

    // Play sound on phase transitions
    if phase != *last_phase {
        *last_phase = phase;
        if let Some(ref sfx) = sfx {
            if phase <= 3 {
                audio::play_sfx(&mut commands, &sfx.countdown_beep);
            } else {
                audio::play_sfx(&mut commands, &sfx.countdown_go);
            }
        }
    }

    // Update the text child of the countdown overlay
    for children in &countdown_parent {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = label.to_string();
            }
        }
    }

    if countdown.timer.just_finished() {
        commands.remove_resource::<CountdownTimer>();
        next_state.set(GameState::Playing);
    }
}

fn cleanup_dead_snakes(
    mut commands: Commands,
    time: Res<Time>,
    mut dead_query: Query<(Entity, &mut DeathTimer)>,
    segment_query: Query<(Entity, &rendering::SnakeSegmentSprite)>,
) {
    for (snake_entity, mut death_timer) in &mut dead_query {
        death_timer.timer.tick(time.delta());
        if death_timer.timer.just_finished() {
            for (seg_entity, seg) in &segment_query {
                if seg.snake_entity == snake_entity
                    && let Ok(mut ec) = commands.get_entity(seg_entity)
                {
                    ec.despawn();
                }
            }
            if let Ok(mut ec) = commands.get_entity(snake_entity) {
                ec.despawn();
            }
        }
    }
}

#[derive(Resource)]
struct ArenaShrinkTimer {
    timer: Timer,
}

#[derive(Resource)]
struct SpeedTimer {
    timer: Timer,
}

#[derive(Resource)]
struct MatchTimer {
    elapsed: f32,
}

#[allow(clippy::too_many_arguments)]
fn arena_shrink(
    mut commands: Commands,
    time: Res<Time>,
    mut shrink_timer: ResMut<ArenaShrinkTimer>,
    mut bounds: ResMut<ArenaBounds>,
    mut snake_query: Query<(Entity, &mut Snake, &SnakeColor, &SnakeId)>,
    food_query: Query<(Entity, &Food)>,
    mut shake: ResMut<rendering::ScreenShake>,
    mut warning: ResMut<rendering::ShrinkWarning>,
    sfx: Option<Res<audio::SoundEffects>>,
) {
    shrink_timer.timer.tick(time.delta());

    // Activate shrink warning when ~2 seconds remain (timer elapsed > 10s of 12s interval)
    let elapsed = shrink_timer.timer.elapsed_secs();
    let duration = shrink_timer.timer.duration().as_secs_f32();
    let was_warning = warning.active;
    warning.active = bounds.can_shrink() && elapsed > (duration - 2.0);

    // Play warning sound on activation (not every frame)
    if warning.active && !was_warning
        && let Some(ref sfx) = sfx
    {
        audio::play_sfx(&mut commands, &sfx.shrink_warning);
    }

    if !shrink_timer.timer.just_finished() {
        return;
    }

    if !bounds.can_shrink() {
        return;
    }

    bounds.shrink();

    // Camera shake + impact sound on arena shrink
    if let Some(ref sfx) = sfx {
        audio::play_sfx(&mut commands, &sfx.shrink_impact);
    }
    shake.intensity = 4.0;

    for (entity, mut snake, color, snake_id) in &mut snake_query {
        if !snake.alive {
            continue;
        }
        if !bounds.contains(snake.head()) {
            let death_pos = snake.head().to_world();
            snake.alive = false;
            commands.entity(entity).insert(DeathTimer {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
            });
            // Death particles for snakes crushed by arena
            rendering::spawn_death_particles(
                &mut commands,
                death_pos,
                color.head,
                time.elapsed_secs(),
            );
            shake.intensity = 8.0; // Stronger shake if someone dies from shrink

            // Kill feed for arena crush
            let dead_name = if snake_id.0 == 0 {
                "You".to_string()
            } else {
                rendering::get_snake_color_name(snake_id.0).to_string()
            };
            rendering::spawn_kill_feed_entry(
                &mut commands,
                format!("{} crushed! such squish!", dead_name),
                color.head,
            );
        }
    }

    for (entity, food) in &food_query {
        if !bounds.contains(food.pos)
            && let Ok(mut ec) = commands.get_entity(entity)
        {
            ec.despawn();
        }
    }
}

fn speed_increase(
    mut commands: Commands,
    time: Res<Time>,
    mut speed_timer: ResMut<SpeedTimer>,
    mut tick: ResMut<GameTick>,
    sfx: Option<Res<audio::SoundEffects>>,
) {
    speed_timer.timer.tick(time.delta());
    if !speed_timer.timer.just_finished() {
        return;
    }

    let current = tick.timer.duration().as_secs_f32();
    let new_interval = (current * 0.85).max(0.06);
    tick.timer.set_duration(std::time::Duration::from_secs_f32(new_interval));

    // Show "SPEED UP!" indicator + sound
    if let Some(ref sfx) = sfx {
        audio::play_sfx(&mut commands, &sfx.speed_up);
    }
    effects::spawn_speed_up_text(&mut commands);
}

fn track_match_time(
    time: Res<Time>,
    mut match_timer: ResMut<MatchTimer>,
) {
    match_timer.elapsed += time.delta_secs();
}

fn update_timer_text(
    match_timer: Res<MatchTimer>,
    mut text_query: Query<&mut Text, With<rendering::TimerText>>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };
    let secs = match_timer.elapsed as u32;
    let mins = secs / 60;
    let secs = secs % 60;
    **text = format!("{}:{:02}", mins, secs);
}

fn handle_esc_quit(
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

#[derive(Resource)]
struct ScreenshotTimer {
    timer: Timer,
    enabled: bool,
    counter: u32,
}

/// Event-driven visual testing mode (AUTO_TEST=1).
/// Captures screenshots at meaningful game events instead of timed intervals.
#[derive(Resource)]
struct AutoTestState {
    enabled: bool,
    captured: HashSet<String>,
    prev_alive_count: u32,
    arena_shrunk: bool,
    exit_timer: Option<Timer>,
}

fn auto_screenshot(
    mut commands: Commands,
    time: Res<Time>,
    mut screenshot_timer: ResMut<ScreenshotTimer>,
) {
    if !screenshot_timer.enabled {
        return;
    }

    screenshot_timer.timer.tick(time.delta());
    if screenshot_timer.timer.just_finished() {
        let path = format!("/tmp/dogebread-auto-{}.png", screenshot_timer.counter);
        screenshot_timer.counter += 1;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn auto_test_capture(name: &str, commands: &mut Commands) {
    let path = format!("test-output/{}", name);
    info!("[AUTO_TEST] capturing {}", path);
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

#[allow(clippy::too_many_arguments)]
fn auto_test_system(
    mut commands: Commands,
    time: Res<Time>,
    mut auto_test: ResMut<AutoTestState>,
    state: Res<State<GameState>>,
    match_state: Res<MatchState>,
    countdown: Option<Res<CountdownTimer>>,
    anim: Option<Res<rendering::GameOverAnimation>>,
    bounds: Res<ArenaBounds>,
) {
    if !auto_test.enabled {
        return;
    }

    // Handle exit timer
    if let Some(ref mut timer) = auto_test.exit_timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            info!("[AUTO_TEST] all captures done, exiting");
            std::process::exit(0);
        }
        return;
    }

    let current_state = *state.get();

    // --- Countdown captures ---
    if current_state == GameState::Countdown
        && let Some(ref cd) = countdown
    {
        let elapsed = cd.timer.elapsed_secs();
        // "3" shows at elapsed 0-1s — capture early in that window
        if (0.1..0.9).contains(&elapsed) && !auto_test.captured.contains("01") {
            auto_test.captured.insert("01".to_string());
            auto_test_capture("01-countdown-3.png", &mut commands);
        }
        // "GO!" shows at elapsed >= 3.0s
        if elapsed >= 3.1 && !auto_test.captured.contains("02") {
            auto_test.captured.insert("02".to_string());
            auto_test_capture("02-countdown-go.png", &mut commands);
        }
    }

    // --- Gameplay start ---
    if current_state == GameState::Playing && !auto_test.captured.contains("03") {
        auto_test.captured.insert("03".to_string());
        auto_test_capture("03-gameplay-start.png", &mut commands);
        // Initialize alive tracking now that we're playing
        auto_test.prev_alive_count = match_state.alive_count;
    }

    // --- Track deaths (only during Playing) ---
    if current_state == GameState::Playing {
        // First death: alive_count drops below total_snakes
        if match_state.alive_count < auto_test.prev_alive_count
            && !auto_test.captured.contains("04")
        {
            auto_test.captured.insert("04".to_string());
            auto_test_capture("04-first-death.png", &mut commands);
        }

        // Arena shrink: detect when bounds differ from default
        let default_bounds = ArenaBounds::default();
        if !auto_test.arena_shrunk && bounds.min_x > default_bounds.min_x {
            auto_test.arena_shrunk = true;
        }
        if auto_test.arena_shrunk && !auto_test.captured.contains("05") {
            auto_test.captured.insert("05".to_string());
            auto_test_capture("05-arena-shrink.png", &mut commands);
        }

        // Late game: 3 or fewer alive
        if match_state.alive_count <= 3 && !auto_test.captured.contains("06") {
            auto_test.captured.insert("06".to_string());
            auto_test_capture("06-late-game.png", &mut commands);
        }

        auto_test.prev_alive_count = match_state.alive_count;
    }

    // --- Game over phases ---
    if current_state == GameState::GameOver
        && let Some(ref ga) = anim
    {
        // Phase 1: title visible (phase advances to 1 after title spawns)
        if ga.phase >= 1 && !auto_test.captured.contains("07") {
            auto_test.captured.insert("07".to_string());
            auto_test_capture("07-gameover-title.png", &mut commands);
        }
        // Phase 3: rankings visible
        if ga.phase >= 3 && !auto_test.captured.contains("08") {
            auto_test.captured.insert("08".to_string());
            auto_test_capture("08-gameover-rankings.png", &mut commands);
        }
        // Phase 4: restart prompt visible (final state)
        if ga.phase >= 4 && !auto_test.captured.contains("09") {
            auto_test.captured.insert("09".to_string());
            auto_test_capture("09-gameover-complete.png", &mut commands);
            // Start exit timer
            auto_test.exit_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
        }
    }
}
