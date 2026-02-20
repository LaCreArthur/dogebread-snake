mod camera;
mod grid;
mod hud;
mod overlays;
mod world;

use bevy::prelude::*;

// Re-export effects functions used by lib.rs via rendering:: call sites
pub use crate::effects::{spawn_death_particles, spawn_eat_particles, spawn_score_popup};

// --- Components ---

/// Grid background cell with its position
#[derive(Component)]
pub struct GridCell {
    pub pos: shared::game::GridPos,
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
    pub snake_id: shared::game::SnakeId,
}

/// Countdown text displayed during 3-2-1-GO phase
#[derive(Component)]
pub struct CountdownText;

/// Kill feed entry in the top-right corner
#[derive(Component)]
pub struct KillFeedEntry {
    pub timer: Timer,
}

/// Marker for spectating text
#[derive(Component)]
pub struct SpectatingText;

/// Marker for start prompt UI
#[derive(Component)]
pub struct StartPrompt;

/// Marker for game over UI overlay
#[derive(Component)]
pub struct GameOverOverlay;

#[derive(Component)]
pub(crate) struct GameOverTitle;

#[derive(Component)]
pub(crate) struct GameOverWinner;

#[derive(Component)]
pub(crate) struct GameOverRankings;

#[derive(Component)]
pub(crate) struct GameOverRestart;

// --- Resources ---

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

/// Cached player stats — survives entity despawn so HUD keeps showing score after death
#[derive(Resource, Default)]
pub struct PlayerStats {
    pub score: u32,
    pub kills: u32,
}

/// Phased animation controller for the game over screen
#[derive(Resource)]
pub struct GameOverAnimation {
    pub(crate) timer: Timer,
    pub(crate) phase: u8,
    pub(crate) player_won: bool,
    pub(crate) player_lost: bool,
    pub(crate) winner_text: String,
    pub(crate) rankings: Vec<RankingEntry>,
    pub(crate) total_kills: u32,
}

pub(crate) struct RankingEntry {
    pub name: String,
    pub score: u32,
    pub kills: u32,
    pub alive: bool,
    pub color: Color,
    pub is_player: bool,
}

// --- Constants ---

pub(crate) const MINIMAP_SIZE: f32 = 150.0;
pub(crate) const MINIMAP_DOT: f32 = 5.0;
pub(crate) const MINIMAP_MARGIN: f32 = 20.0;

// Doge-inspired color palette
pub(crate) const DOGE_GOLD: Color = Color::srgb(0.91, 0.69, 0.29);
pub(crate) const COLOR_GRID_A: Color = Color::srgb(0.12, 0.12, 0.20);
pub(crate) const COLOR_GRID_B: Color = Color::srgb(0.14, 0.14, 0.22);
pub(crate) const COLOR_WALL: Color = Color::srgb(0.58, 0.45, 0.20);
pub(crate) const COLOR_DANGER: Color = Color::srgb(0.85, 0.35, 0.15);
pub(crate) const COLOR_DANGER_BRIGHT: Color = Color::srgb(1.0, 0.55, 0.25);
pub(crate) const COLOR_FOOD: Color = Color::srgb(0.95, 0.73, 0.20);

// --- Public re-exports ---

pub use camera::camera_follow;
pub use grid::{spawn_grid, update_grid_cells};
pub use hud::{spawn_ui, update_alive_text, update_minimap, update_spectating};
pub use overlays::{
    animate_game_over, animate_kill_feed, despawn_countdown_overlay, get_snake_color_name, hide_game_over,
    hide_start_prompt, show_game_over, show_start_prompt, spawn_countdown_overlay, spawn_kill_feed_entry,
};
pub use world::{render_food, render_snakes};

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
