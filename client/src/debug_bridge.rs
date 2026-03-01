//! Debug bridge: exposes game state to JavaScript via window.__gameDebug.
//!
//! On WASM: serializes game state every tick via inline JS.
//! On native: no-op system (zero cost).

#[cfg(target_arch = "wasm32")]
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
use shared::game::*;

#[cfg(target_arch = "wasm32")]
use crate::GameTick;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = "
    export function setDebugState(json) {
        window.__gameDebug = JSON.parse(json);
    }
")]
unsafe extern "C" {
    fn setDebugState(json: &str);
}

/// Bevy system that pushes game state to JS on every game tick.
#[cfg(target_arch = "wasm32")]
pub fn sync_debug_state(
    tick: Res<GameTick>,
    state: Res<State<GameState>>,
    match_state: Res<MatchState>,
    bounds: Res<ArenaBounds>,
    snakes: Query<(&Snake, Option<&PlayerControlled>)>,
) {
    let (player_score, player_kills) = snakes
        .iter()
        .find(|(_, p)| p.is_some())
        .map(|(s, _)| (s.score, s.kills))
        .unwrap_or((0, 0));

    let json = format!(
        r#"{{"loaded":true,"gameState":"{}","aliveCount":{},"totalSnakes":{},"playerScore":{},"playerKills":{},"tick":{},"arenaBounds":{{"min_x":{},"min_y":{},"max_x":{},"max_y":{}}}}}"#,
        state_str(*state.get()),
        match_state.alive_count,
        match_state.total_snakes,
        player_score,
        player_kills,
        tick.tick_count,
        bounds.min_x,
        bounds.min_y,
        bounds.max_x,
        bounds.max_y,
    );

    setDebugState(&json);
}

#[cfg(target_arch = "wasm32")]
fn state_str(state: GameState) -> &'static str {
    match state {
        GameState::Home => "Home",
        GameState::NameEntry => "NameEntry",
        GameState::WaitingToStart => "WaitingToStart",
        GameState::Countdown => "Countdown",
        GameState::Playing => "Playing",
        GameState::GameOver => "GameOver",
        GameState::Leaderboard => "Leaderboard",
    }
}

/// No-op on native — Bevy handles zero-param systems fine.
#[cfg(not(target_arch = "wasm32"))]
pub fn sync_debug_state() {}
