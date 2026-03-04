use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

use crate::audio;
use crate::rendering;
use crate::testing::AutoTestState;
use crate::{NUM_FOOD, NUM_SNAKES, SimpleRng};

/// Countdown resource: tracks the 3-2-1-GO! timer
#[derive(Resource)]
pub(crate) struct CountdownTimer {
    pub timer: Timer,
}

pub(crate) fn spawn_positions() -> Vec<(i32, i32, Direction)> {
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

pub fn spawn_match(mut commands: Commands, mut rng: ResMut<SimpleRng>, mut match_state: ResMut<MatchState>) {
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

pub fn restart_on_space(
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let should_restart = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::KeyR)
        || touches.iter_just_pressed().next().is_some();

    if should_restart {
        next_state.set(GameState::WaitingToStart);
    }
}

pub fn wait_for_start(
    keyboard: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut snake_query: Query<&mut Snake, With<PlayerControlled>>,
    auto_test: Res<AutoTestState>,
) {
    let dir = if auto_test.enabled {
        Some(Direction::Right)
    } else if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        Some(Direction::Up)
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        Some(Direction::Down)
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        Some(Direction::Left)
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        Some(Direction::Right)
    } else if touches.iter_just_pressed().next().is_some() {
        // Any tap starts the game — swipe direction is handled by handle_touch_input during play
        Some(Direction::Right)
    } else {
        None
    };

    if let Some(d) = dir {
        if let Ok(mut snake) = snake_query.single_mut() {
            snake.set_direction(d);
        }
        commands.insert_resource(CountdownTimer {
            timer: Timer::from_seconds(3.5, TimerMode::Once),
        });
        next_state.set(GameState::Countdown);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_countdown(
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

    let (label, phase) = if elapsed < 1.0 {
        ("such 3", 1u8)
    } else if elapsed < 2.0 {
        ("very 2", 2)
    } else if elapsed < 3.0 {
        ("much 1", 3)
    } else {
        ("WOW GO!", 4)
    };

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

pub fn cleanup_dead_snakes(
    mut commands: Commands,
    time: Res<Time>,
    mut dead_query: Query<(Entity, &mut DeathTimer, &mut Snake)>,
    segment_query: Query<(Entity, &rendering::SnakeSegmentSprite)>,
) {
    for (snake_entity, mut death_timer, mut snake) in &mut dead_query {
        death_timer.timer.tick(time.delta());
        if death_timer.timer.just_finished() {
            for (seg_entity, seg) in &segment_query {
                if seg.snake_entity == snake_entity
                    && let Ok(mut ec) = commands.get_entity(seg_entity)
                {
                    ec.despawn();
                }
            }
            // Clear segments so render_snakes won't re-create sprites.
            // Keep the entity alive so show_game_over can read score/kills.
            snake.segments.clear();
            commands.entity(snake_entity).remove::<DeathTimer>();
        }
    }
}
