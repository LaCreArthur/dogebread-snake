use std::collections::HashSet;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use shared::game::*;

use crate::match_lifecycle::CountdownTimer;
use crate::rendering;

#[derive(Resource)]
pub(crate) struct ScreenshotTimer {
    pub timer: Timer,
    pub enabled: bool,
    pub counter: u32,
}

/// Event-driven visual testing mode (AUTO_TEST=1).
/// Captures screenshots at meaningful game events instead of timed intervals.
#[derive(Resource)]
pub(crate) struct AutoTestState {
    pub enabled: bool,
    pub captured: HashSet<String>,
    pub prev_alive_count: u32,
    pub arena_shrunk: bool,
    pub exit_timer: Option<Timer>,
}

/// In AUTO_TEST mode, give the player snake AI control so it actually plays.
pub fn auto_test_ai(
    mut commands: Commands,
    auto_test: Res<AutoTestState>,
    player_query: Query<Entity, (With<PlayerControlled>, Without<AiControlled>)>,
) {
    if !auto_test.enabled {
        return;
    }
    for entity in &player_query {
        commands.entity(entity).insert(AiControlled);
    }
}

pub fn auto_screenshot(mut commands: Commands, time: Res<Time>, mut screenshot_timer: ResMut<ScreenshotTimer>) {
    if !screenshot_timer.enabled {
        return;
    }

    screenshot_timer.timer.tick(time.delta());
    if screenshot_timer.timer.just_finished() {
        let path = format!("/tmp/dogebread-auto-{}.png", screenshot_timer.counter);
        screenshot_timer.counter += 1;
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }
}

fn auto_test_capture(name: &str, commands: &mut Commands) {
    let path = format!("test-output/{}", name);
    info!("[AUTO_TEST] capturing {}", path);
    commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
}

#[allow(clippy::too_many_arguments)]
pub fn auto_test_system(
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

    if let Some(ref mut timer) = auto_test.exit_timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            info!("[AUTO_TEST] all captures done, exiting");
            std::process::exit(0);
        }
        return;
    }

    let current_state = *state.get();

    if current_state == GameState::Countdown
        && let Some(ref cd) = countdown
    {
        let elapsed = cd.timer.elapsed_secs();
        if (0.1..0.9).contains(&elapsed) && !auto_test.captured.contains("01") {
            auto_test.captured.insert("01".to_string());
            auto_test_capture("01-countdown-3.png", &mut commands);
        }
        if elapsed >= 3.1 && !auto_test.captured.contains("02") {
            auto_test.captured.insert("02".to_string());
            auto_test_capture("02-countdown-go.png", &mut commands);
        }
    }

    if current_state == GameState::Playing && !auto_test.captured.contains("03") {
        auto_test.captured.insert("03".to_string());
        auto_test_capture("03-gameplay-start.png", &mut commands);
        auto_test.prev_alive_count = match_state.alive_count;
    }

    if current_state == GameState::Playing {
        if match_state.alive_count < auto_test.prev_alive_count && !auto_test.captured.contains("04") {
            auto_test.captured.insert("04".to_string());
            auto_test_capture("04-first-death.png", &mut commands);
        }

        let default_bounds = ArenaBounds::default();
        if !auto_test.arena_shrunk && bounds.min_x > default_bounds.min_x {
            auto_test.arena_shrunk = true;
        }
        if auto_test.arena_shrunk && !auto_test.captured.contains("05") {
            auto_test.captured.insert("05".to_string());
            auto_test_capture("05-arena-shrink.png", &mut commands);
        }

        if match_state.alive_count <= 3 && !auto_test.captured.contains("06") {
            auto_test.captured.insert("06".to_string());
            auto_test_capture("06-late-game.png", &mut commands);
        }

        auto_test.prev_alive_count = match_state.alive_count;
    }

    if current_state == GameState::GameOver
        && let Some(ref ga) = anim
    {
        if ga.phase >= 1 && !auto_test.captured.contains("07") {
            auto_test.captured.insert("07".to_string());
            auto_test_capture("07-gameover-title.png", &mut commands);
        }
        if ga.phase >= 3 && !auto_test.captured.contains("08") {
            auto_test.captured.insert("08".to_string());
            auto_test_capture("08-gameover-rankings.png", &mut commands);
        }
        if ga.phase >= 4 && !auto_test.captured.contains("09") {
            auto_test.captured.insert("09".to_string());
            auto_test_capture("09-gameover-complete.png", &mut commands);
            auto_test.exit_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
        }
    }
}
