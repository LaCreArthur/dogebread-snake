mod audio;
mod debug_bridge;
mod effects;
mod game_systems;
mod input;
mod match_lifecycle;
mod menu;
mod rendering;
mod testing;
mod ui;

use std::collections::HashSet;

use bevy::prelude::*;
use shared::constants::*;
use shared::game::*;

pub(crate) const NUM_SNAKES: u32 = 10;
pub(crate) const NUM_FOOD: usize = 35;
const ARENA_SHRINK_INTERVAL: f32 = 12.0;
const SPEED_INCREASE_INTERVAL: f32 = 20.0;

#[derive(Resource)]
pub(crate) struct GameTick {
    pub timer: Timer,
    pub tick_count: u32,
}

#[derive(Resource)]
pub(crate) struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        let seed = (js_sys::Date::now() * 1000.0) as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let seed = {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
        };
        Self { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }

    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        min + (self.next_u32() % (max - min) as u32) as i32
    }
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
        .insert_resource(ClearColor(Color::srgb(0.10, 0.10, 0.18)))
        .insert_resource(GameTick {
            timer: Timer::from_seconds(TICK_INTERVAL as f32, TimerMode::Repeating),
            tick_count: 0,
        })
        .insert_resource(SimpleRng::new())
        .insert_resource(MatchState::default())
        .insert_resource(ArenaBounds::default())
        .insert_resource(game_systems::ArenaShrinkTimer::new(ARENA_SHRINK_INTERVAL))
        .insert_resource(game_systems::SpeedTimer::new(SPEED_INCREASE_INTERVAL))
        .insert_resource(game_systems::MatchTimer { elapsed: 0.0 })
        .insert_resource(testing::ScreenshotTimer {
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
            testing::AutoTestState {
                enabled,
                captured: HashSet::new(),
                prev_alive_count: 0,
                arena_shrunk: false,
                exit_timer: None,
            }
        })
        .insert_resource(rendering::ScreenShake::default())
        .insert_resource(rendering::ShrinkWarning::default())
        .insert_resource(rendering::PlayerStats::default())
        .insert_resource(effects::TrailSpawner {
            timer: Timer::from_seconds(0.15, TimerMode::Repeating),
        })
        .insert_resource(menu::LeaderboardData::from_storage())
        .insert_resource(menu::PlayerName::default())
        .init_state::<GameState>()
        .add_systems(
            Startup,
            (
                rendering::load_sprite_assets,
                rendering::spawn_grid,
                rendering::spawn_ui,
                audio::setup_audio,
            ),
        )
        // Home screen
        .add_systems(OnEnter(GameState::Home), menu::show_home)
        .add_systems(OnExit(GameState::Home), menu::hide_home)
        .add_systems(
            Update,
            (menu::home_play_button, menu::home_leaderboard_button, testing::auto_skip_home)
                .run_if(in_state(GameState::Home)),
        )
        // Name entry screen
        .add_systems(OnEnter(GameState::NameEntry), menu::show_name_entry)
        .add_systems(OnExit(GameState::NameEntry), menu::hide_name_entry)
        .add_systems(
            Update,
            (
                menu::name_entry_input,
                menu::name_entry_start_button,
                testing::auto_skip_name_entry,
            )
                .run_if(in_state(GameState::NameEntry)),
        )
        // Leaderboard screen
        .add_systems(OnEnter(GameState::Leaderboard), menu::show_leaderboard)
        .add_systems(OnExit(GameState::Leaderboard), menu::hide_leaderboard)
        .add_systems(
            Update,
            menu::leaderboard_home_button.run_if(in_state(GameState::Leaderboard)),
        )
        // WaitingToStart: spawn match + show prompt
        .add_systems(
            OnEnter(GameState::WaitingToStart),
            (match_lifecycle::spawn_match, rendering::show_start_prompt),
        )
        .add_systems(OnExit(GameState::WaitingToStart), rendering::hide_start_prompt)
        .add_systems(
            Update,
            match_lifecycle::wait_for_start.run_if(in_state(GameState::WaitingToStart)),
        )
        .add_systems(OnEnter(GameState::Countdown), rendering::spawn_countdown_overlay)
        .add_systems(OnExit(GameState::Countdown), rendering::despawn_countdown_overlay)
        .add_systems(
            Update,
            match_lifecycle::run_countdown.run_if(in_state(GameState::Countdown)),
        )
        .add_systems(
            Update,
            (
                input::handle_input,
                input::ai_tick,
                game_systems::game_tick.after(input::handle_input).after(input::ai_tick),
                game_systems::arena_shrink.after(game_systems::game_tick),
                game_systems::speed_increase,
                game_systems::track_match_time,
                effects::spawn_trail_particles,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            OnEnter(GameState::GameOver),
            (menu::save_match_to_leaderboard, rendering::show_game_over),
        )
        .add_systems(
            OnExit(GameState::GameOver),
            rendering::hide_game_over,
        )
        .add_systems(
            OnExit(GameState::GameOver),
            menu::cleanup_match_entities,
        )
        .add_systems(
            OnExit(GameState::GameOver),
            menu::cleanup_match_resources,
        )
        .add_systems(
            Update,
            (
                match_lifecycle::restart_on_space,
                rendering::animate_game_over,
                menu::gameover_play_again_button,
                menu::gameover_home_button,
            )
                .run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            Update,
            (
                rendering::render_snakes,
                rendering::render_food,
                rendering::update_alive_text,
                rendering::update_grid_cells,
                rendering::update_minimap,
                rendering::update_spectating,
                rendering::camera_follow,
                game_systems::update_timer_text,
            ),
        )
        .add_systems(
            Update,
            (
                match_lifecycle::cleanup_dead_snakes,
                game_systems::handle_esc_quit,
                testing::auto_test_ai,
                testing::auto_screenshot,
                testing::auto_test_system,
                rendering::animate_kill_feed,
                debug_bridge::sync_debug_state,
            ),
        )
        .add_systems(
            Update,
            (
                effects::animate_floating_text,
                effects::animate_death_particles,
                effects::animate_eat_particles,
                effects::animate_speed_up_text,
                effects::animate_trail_particles,
            ),
        )
        .add_systems(Update, menu::button_hover_system)
        .run();
}
