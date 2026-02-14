mod input;
mod rendering;
mod ui;

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use shared::constants::*;
use shared::game::*;

const NUM_SNAKES: u32 = 4; // 1 player + 3 AI
const NUM_FOOD: usize = 8;

/// Timer resource for tick-based movement
#[derive(Resource)]
struct GameTick {
    timer: Timer,
}

/// Simple RNG resource
#[derive(Resource)]
struct SimpleRng {
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(rendering::window_setup()))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.13)))
        .insert_resource(GameTick {
            timer: Timer::from_seconds(TICK_INTERVAL as f32, TimerMode::Repeating),
        })
        .insert_resource(SimpleRng::new())
        .insert_resource(MatchState::default())
        .insert_resource(ScreenshotTimer {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            enabled: std::env::var("AUTO_SCREENSHOT").is_ok(),
            counter: 0,
        })
        .init_state::<GameState>()
        .add_systems(Startup, (rendering::spawn_grid, rendering::spawn_ui, spawn_match))
        .add_systems(
            Update,
            (
                input::handle_input,
                input::ai_tick,
                game_tick.after(input::handle_input).after(input::ai_tick),
                rendering::render_snakes.after(game_tick),
                rendering::render_food.after(game_tick),
                rendering::update_alive_text.after(game_tick),
                rendering::camera_follow,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, (restart_on_space, cleanup_dead_snakes))
        .add_systems(OnEnter(GameState::GameOver), rendering::show_game_over)
        .add_systems(OnExit(GameState::GameOver), rendering::hide_game_over)
        .add_systems(Update, restart_on_space.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, auto_screenshot)
        .run();
}

/// Spawn positions for snakes — corners, heading along walls (not toward center)
fn spawn_positions() -> Vec<(i32, i32, Direction)> {
    vec![
        (5, 5, Direction::Right),           // Player: bottom-left, heading right
        (GRID_WIDTH - 6, 5, Direction::Up),        // AI 1: bottom-right, heading up
        (5, GRID_HEIGHT - 6, Direction::Right),    // AI 2: top-left, heading right
        (GRID_WIDTH - 6, GRID_HEIGHT - 6, Direction::Down), // AI 3: top-right, heading down
        (GRID_WIDTH / 2, 5, Direction::Up),
        (5, GRID_HEIGHT / 2, Direction::Right),
        (GRID_WIDTH - 6, GRID_HEIGHT / 2, Direction::Left),
        (GRID_WIDTH / 2, GRID_HEIGHT - 6, Direction::Down),
    ]
}

/// Spawn all game entities for a match
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

    // Spawn food
    for _ in 0..NUM_FOOD {
        let x = rng.range(2, GRID_WIDTH - 2);
        let y = rng.range(2, GRID_HEIGHT - 2);
        commands.spawn(Food::new(x, y));
    }

    match_state.total_snakes = NUM_SNAKES;
    match_state.alive_count = NUM_SNAKES;
}

/// Main game tick
fn game_tick(
    mut commands: Commands,
    time: Res<Time>,
    mut tick: ResMut<GameTick>,
    mut rng: ResMut<SimpleRng>,
    mut snake_query: Query<(Entity, &mut Snake, &SnakeId)>,
    food_query: Query<(Entity, &Food)>,
    mut match_state: ResMut<MatchState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    tick.timer.tick(time.delta());
    if !tick.timer.just_finished() {
        return;
    }

    // Step 1: Move all alive snakes
    let mut new_heads: Vec<(Entity, SnakeId, GridPos)> = Vec::new();
    for (entity, mut snake, id) in &mut snake_query {
        if !snake.alive {
            continue;
        }
        let head = snake.step();
        new_heads.push((entity, *id, head));
    }

    // Step 2: Collect all body segments (excluding heads) for collision checks
    let body_map: Vec<(SnakeId, Vec<GridPos>)> = snake_query
        .iter()
        .filter(|(_, s, _)| s.alive)
        .map(|(_, s, id)| {
            (*id, s.segments.iter().skip(1).copied().collect())
        })
        .collect();

    // Step 3: Check collisions
    let mut kills: Vec<Entity> = Vec::new();

    for (entity, my_id, head) in &new_heads {
        // Wall collision
        if !head.in_bounds() {
            kills.push(*entity);
            continue;
        }

        // Self collision
        if let Ok((_, snake, _)) = snake_query.get(*entity) {
            if snake.self_collision() {
                kills.push(*entity);
                continue;
            }
        }

        // Head hits another snake's body
        for (other_id, body_segments) in &body_map {
            if *other_id == *my_id {
                continue;
            }
            if body_segments.iter().any(|s| *s == *head) {
                kills.push(*entity);
                break;
            }
        }

        // Head-to-head collision (both die)
        for (other_entity, other_id, other_head) in &new_heads {
            if *other_id == *my_id {
                continue;
            }
            if *head == *other_head {
                kills.push(*entity);
                kills.push(*other_entity);
            }
        }
    }

    // Apply deaths
    kills.sort();
    kills.dedup();
    for entity in &kills {
        if let Ok((_, mut snake, _)) = snake_query.get_mut(*entity) {
            if snake.alive {
                snake.alive = false;
                commands.entity(*entity).insert(DeathTimer {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                });
            }
        }
    }

    // Update alive count
    let alive = snake_query.iter().filter(|(_, s, _)| s.alive).count() as u32;
    match_state.alive_count = alive;

    // Check win condition: only 1 or 0 alive
    if alive <= 1 && match_state.total_snakes > 1 {
        next_state.set(GameState::GameOver);
    }

    // Food collision (for surviving snakes)
    for (_entity, snake, _) in &snake_query {
        if !snake.alive {
            continue;
        }
        let head = snake.head();
        for (food_entity, food) in &food_query {
            if food.pos == head {
                commands.entity(food_entity).despawn();
                // Need mut access — re-query below
            }
        }
    }

    // Grow snakes that ate food (separate pass for borrow checker)
    let eaten_positions: Vec<GridPos> = snake_query
        .iter()
        .filter(|(_, s, _)| s.alive)
        .map(|(_, s, _)| s.head())
        .collect();

    let mut foods_eaten = 0;
    for (food_entity, food) in &food_query {
        if eaten_positions.contains(&food.pos) {
            commands.entity(food_entity).despawn();
            foods_eaten += 1;

            // Find which snake ate it and grow them
            for (_, mut snake, _) in &mut snake_query {
                if snake.alive && snake.head() == food.pos {
                    snake.grow_pending += 2;
                }
            }
        }
    }

    // Respawn food to maintain count
    let remaining_food = food_query.iter().count() - foods_eaten;
    let target_food = NUM_FOOD;
    if remaining_food < target_food {
        for _ in 0..(target_food - remaining_food) {
            let x = rng.range(1, GRID_WIDTH - 1);
            let y = rng.range(1, GRID_HEIGHT - 1);
            commands.spawn(Food::new(x, y));
        }
    }
}

/// Restart match on Space key
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
    state: Res<State<GameState>>,
) {
    // Only restart if game is over, or if R is pressed during play
    let should_restart = (*state.get() == GameState::GameOver && keyboard.just_pressed(KeyCode::Space))
        || keyboard.just_pressed(KeyCode::KeyR);

    if !should_restart {
        return;
    }

    // Despawn everything
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

    // Re-spawn match
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
    next_state.set(GameState::Playing);
}

/// Clean up dead snake bodies after their death timer expires
fn cleanup_dead_snakes(
    mut commands: Commands,
    time: Res<Time>,
    mut dead_query: Query<(Entity, &mut DeathTimer)>,
    segment_query: Query<(Entity, &rendering::SnakeSegmentSprite)>,
) {
    for (snake_entity, mut death_timer) in &mut dead_query {
        death_timer.timer.tick(time.delta());
        if death_timer.timer.just_finished() {
            // Remove all segment sprites for this snake
            for (seg_entity, seg) in &segment_query {
                if seg.snake_entity == snake_entity {
                    commands.entity(seg_entity).despawn();
                }
            }
            // Remove the snake entity itself
            commands.entity(snake_entity).despawn();
        }
    }
}

/// Auto-screenshot resource (enable with AUTO_SCREENSHOT=1 env var)
#[derive(Resource)]
struct ScreenshotTimer {
    timer: Timer,
    enabled: bool,
    counter: u32,
}

/// Takes screenshots automatically when enabled via AUTO_SCREENSHOT env var
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
        // Save to both numbered and "latest" path using the same screenshot
        let path = format!("/tmp/dogebread-auto-{}.png", screenshot_timer.counter);
        screenshot_timer.counter += 1;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}
