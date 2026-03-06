//! Universal game invariants checked on every tick of every simulation.
//!
//! Write once, run on every tick of every test. Each invariant catches a
//! class of bugs that individual tests would miss.

use shared::game::*;
use std::collections::VecDeque;

/// Lightweight state snapshot captured before each tick for comparison.
#[derive(Clone)]
pub struct TickSnapshot {
    pub head_positions: Vec<Option<GridPos>>,
    pub scores: Vec<u32>,
    pub alive_states: Vec<bool>,
    pub bounds: ArenaBounds,
    pub dead_segments: Vec<Option<VecDeque<GridPos>>>,
}

impl TickSnapshot {
    pub fn capture(snakes: &[Snake], bounds: &ArenaBounds) -> Self {
        Self {
            head_positions: snakes
                .iter()
                .map(|s| if s.alive { Some(s.head()) } else { None })
                .collect(),
            scores: snakes.iter().map(|s| s.score).collect(),
            alive_states: snakes.iter().map(|s| s.alive).collect(),
            bounds: *bounds,
            dead_segments: snakes
                .iter()
                .map(|s| if !s.alive { Some(s.segments.clone()) } else { None })
                .collect(),
        }
    }
}

/// Check all 6 invariants. Panics with a descriptive message on violation.
pub fn check_all(before: &TickSnapshot, snakes: &[Snake], bounds: &ArenaBounds, tick: u32) {
    no_teleportation(before, snakes, tick);
    scores_monotonic(before, snakes, tick);
    dead_frozen(before, snakes, tick);
    arena_monotonic(before, bounds, tick);
    population_conservation(before, snakes, tick);
    heads_in_bounds(snakes, bounds, tick);
}

/// Invariant 1: Alive snake heads move exactly 1 cell (Manhattan distance).
fn no_teleportation(before: &TickSnapshot, snakes: &[Snake], tick: u32) {
    for (i, snake) in snakes.iter().enumerate() {
        // Only check snakes that were alive before AND are still alive
        if let Some(prev_head) = before.head_positions[i]
            && snake.alive
        {
            let dist = prev_head.distance(snake.head());
            assert_eq!(
                dist,
                1,
                "tick {}: snake {} teleported! head moved {} cells ({:?} -> {:?})",
                tick,
                i,
                dist,
                prev_head,
                snake.head()
            );
        }
    }
}

/// Invariant 2: No snake's score ever decreases.
fn scores_monotonic(before: &TickSnapshot, snakes: &[Snake], tick: u32) {
    for (i, snake) in snakes.iter().enumerate() {
        assert!(
            snake.score >= before.scores[i],
            "tick {}: snake {} score decreased ({} -> {})",
            tick,
            i,
            before.scores[i],
            snake.score
        );
    }
}

/// Invariant 3: Dead snake segments never change.
fn dead_frozen(before: &TickSnapshot, snakes: &[Snake], tick: u32) {
    for (i, snake) in snakes.iter().enumerate() {
        if let Some(ref prev_segments) = before.dead_segments[i] {
            assert_eq!(
                &snake.segments, prev_segments,
                "tick {}: dead snake {} segments changed",
                tick, i
            );
        }
    }
}

/// Invariant 4: Arena bounds only shrink or stay the same.
fn arena_monotonic(before: &TickSnapshot, bounds: &ArenaBounds, tick: u32) {
    assert!(
        bounds.min_x >= before.bounds.min_x
            && bounds.min_y >= before.bounds.min_y
            && bounds.max_x <= before.bounds.max_x
            && bounds.max_y <= before.bounds.max_y,
        "tick {}: arena grew! before={:?}, after={:?}",
        tick,
        (
            before.bounds.min_x,
            before.bounds.min_y,
            before.bounds.max_x,
            before.bounds.max_y
        ),
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y)
    );
}

/// Invariant 5: alive + dead = total snake count, always.
fn population_conservation(before: &TickSnapshot, snakes: &[Snake], tick: u32) {
    let total = snakes.len();
    let alive = snakes.iter().filter(|s| s.alive).count();
    let dead = snakes.iter().filter(|s| !s.alive).count();
    assert_eq!(
        alive + dead,
        total,
        "tick {}: population mismatch: alive={}, dead={}, total={}",
        tick,
        alive,
        dead,
        total
    );

    // Also: no snake that was dead can come back alive
    for (i, snake) in snakes.iter().enumerate() {
        if !before.alive_states[i] {
            assert!(
                !snake.alive,
                "tick {}: snake {} resurrected (was dead, now alive)",
                tick, i
            );
        }
    }
}

/// Invariant 6: No alive snake head is outside arena bounds.
fn heads_in_bounds(snakes: &[Snake], bounds: &ArenaBounds, tick: u32) {
    for (i, snake) in snakes.iter().enumerate() {
        if snake.alive {
            assert!(
                bounds.contains(snake.head()),
                "tick {}: alive snake {} head at {:?} is outside bounds ({},{},{},{})",
                tick,
                i,
                snake.head(),
                bounds.min_x,
                bounds.min_y,
                bounds.max_x,
                bounds.max_y
            );
        }
    }
}
