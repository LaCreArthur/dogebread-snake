mod input;
mod rendering;
mod ui;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use shared::constants::*;
use shared::game::*;

const NUM_SNAKES: u32 = 6;
const NUM_FOOD: usize = 25;
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
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
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

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
        .insert_resource(rendering::ScreenShake::default())
        .insert_resource(rendering::ShrinkWarning::default())
        .init_state::<GameState>()
        .add_systems(Startup, (rendering::spawn_grid, rendering::spawn_ui, spawn_match))
        .add_systems(OnEnter(GameState::WaitingToStart), rendering::show_start_prompt)
        .add_systems(OnExit(GameState::WaitingToStart), rendering::hide_start_prompt)
        .add_systems(Update, wait_for_start.run_if(in_state(GameState::WaitingToStart)))
        .add_systems(
            Update,
            (
                input::handle_input,
                input::ai_tick,
                game_tick.after(input::handle_input).after(input::ai_tick),
                arena_shrink.after(game_tick),
                speed_increase,
                track_match_time,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), rendering::show_game_over)
        .add_systems(OnExit(GameState::GameOver), rendering::hide_game_over)
        .add_systems(Update, restart_on_space.run_if(in_state(GameState::GameOver)))
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
        .add_systems(Update, (cleanup_dead_snakes, handle_esc_quit, auto_screenshot))
        .add_systems(Update, (
            rendering::animate_floating_text,
            rendering::animate_death_particles,
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

        if let Ok((_, snake, _, _, _)) = snake_query.get(*entity) {
            if snake.self_collision() {
                kills.push((*entity, None));
                continue;
            }
        }

        for (other_id, body_segments) in &body_map {
            if *other_id == *my_id {
                continue;
            }
            if body_segments.iter().any(|s| *s == *head) {
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

    // Process deaths: trigger screen shake + death particles
    for (entity, _) in &unique_kills {
        if let Ok((_, mut snake, _, color, _)) = snake_query.get_mut(*entity) {
            if snake.alive {
                let death_pos = snake.head().to_world();
                snake.alive = false;
                commands.entity(*entity).insert(DeathTimer {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                });

                // Screen shake on any death
                shake.intensity = 8.0;

                // Death explosion particles
                rendering::spawn_death_particles(
                    &mut commands,
                    death_pos,
                    color.head,
                    time.elapsed_secs(),
                );
            }
        }
    }

    let alive = snake_query.iter().filter(|(_, s, _, _, _)| s.alive).count() as u32;
    match_state.alive_count = alive;

    if alive <= 1 && match_state.total_snakes > 1 {
        next_state.set(GameState::GameOver);
    }

    for (_, snake, _, _, _) in &snake_query {
        if !snake.alive {
            continue;
        }
        let head = snake.head();
        for (food_entity, food) in &food_query {
            if food.pos == head {
                commands.entity(food_entity).despawn();
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
            commands.entity(food_entity).despawn();
            foods_eaten += 1;

            for (_, mut snake, _, _, player) in &mut snake_query {
                if snake.alive && snake.head() == food.pos {
                    snake.grow_pending += 2;
                    snake.score += 1;

                    // Score popup only for the player's snake
                    if player.is_some() {
                        rendering::spawn_score_popup(&mut commands, food.pos.to_world());
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
    floating_text_query: Query<Entity, With<rendering::FloatingText>>,
    particle_query: Query<Entity, With<rendering::DeathParticle>>,
    mut shake: ResMut<rendering::ScreenShake>,
) {
    let should_restart = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyR);

    if !should_restart {
        return;
    }

    for entity in &snake_query {
        commands.entity(entity).despawn();
    }
    for entity in &food_query {
        commands.entity(entity).despawn();
    }
    for entity in &segment_query {
        commands.entity(entity).despawn();
    }
    for entity in &food_sprite_query {
        commands.entity(entity).despawn();
    }
    for entity in &overlay_query {
        commands.entity(entity).despawn();
    }
    for entity in &floating_text_query {
        commands.entity(entity).despawn();
    }
    for entity in &particle_query {
        commands.entity(entity).despawn();
    }
    shake.intensity = 0.0;

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
    mut next_state: ResMut<NextState<GameState>>,
    mut snake_query: Query<&mut Snake, With<PlayerControlled>>,
) {
    let dir = if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
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
                if seg.snake_entity == snake_entity {
                    commands.entity(seg_entity).despawn();
                }
            }
            commands.entity(snake_entity).despawn();
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

fn arena_shrink(
    mut commands: Commands,
    time: Res<Time>,
    mut shrink_timer: ResMut<ArenaShrinkTimer>,
    mut bounds: ResMut<ArenaBounds>,
    mut snake_query: Query<(Entity, &mut Snake, &SnakeColor)>,
    food_query: Query<(Entity, &Food)>,
    mut shake: ResMut<rendering::ScreenShake>,
    mut warning: ResMut<rendering::ShrinkWarning>,
) {
    shrink_timer.timer.tick(time.delta());

    // Activate shrink warning when ~2 seconds remain (timer elapsed > 10s of 12s interval)
    let elapsed = shrink_timer.timer.elapsed_secs();
    let duration = shrink_timer.timer.duration().as_secs_f32();
    warning.active = bounds.can_shrink() && elapsed > (duration - 2.0);

    if !shrink_timer.timer.just_finished() {
        return;
    }

    if !bounds.can_shrink() {
        return;
    }

    bounds.shrink();

    // Camera shake on arena shrink
    shake.intensity = 4.0;

    for (entity, mut snake, color) in &mut snake_query {
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
        }
    }

    for (entity, food) in &food_query {
        if !bounds.contains(food.pos) {
            commands.entity(entity).despawn();
        }
    }
}

fn speed_increase(
    time: Res<Time>,
    mut speed_timer: ResMut<SpeedTimer>,
    mut tick: ResMut<GameTick>,
) {
    speed_timer.timer.tick(time.delta());
    if !speed_timer.timer.just_finished() {
        return;
    }

    let current = tick.timer.duration().as_secs_f32();
    let new_interval = (current * 0.85).max(0.06);
    tick.timer.set_duration(std::time::Duration::from_secs_f32(new_interval));
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
