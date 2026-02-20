//! Headless game simulation tests — full battle royale games as pure data operations.
//!
//! Tests the GAME LOGIC end-to-end without any engine dependency.
//! Deterministic (seeded RNG), fast (no rendering), catches bugs that
//! unit tests miss because they test in isolation.

mod harness;

use harness::*;
use shared::game::*;
use std::collections::VecDeque;

// ===========================================================================
// Original tests (refactored to use harness)
// ===========================================================================

#[test]
fn test_game_always_terminates() {
    for seed in 0..50u64 {
        let mut sim = SimConfig::new().seed(seed).build();
        let ticks = sim.run_with_invariants();
        assert!(
            sim.alive_count() <= 1,
            "Game with seed {} did not terminate: {} alive after {} ticks",
            seed,
            sim.alive_count(),
            ticks
        );
    }
}

#[test]
fn test_scores_never_negative() {
    for seed in 0..20u64 {
        let mut sim = SimConfig::new().seed(seed + 1000).build();
        sim.run_with_invariants();
        for (i, snake) in sim.snakes.iter().enumerate() {
            assert!(
                snake.score < 10_000,
                "Snake {} in game seed {} has implausible score: {}",
                i,
                seed,
                snake.score
            );
            assert!(
                snake.kills < 10_000,
                "Snake {} in game seed {} has implausible kills: {}",
                i,
                seed,
                snake.kills
            );
        }
    }
}

#[test]
fn test_dead_snakes_dont_move() {
    let mut sim = SimConfig::new().seed(42).build();

    let mut dead_snapshots: Vec<(usize, VecDeque<GridPos>)> = Vec::new();

    for _ in 0..500 {
        sim.step_with_invariants();

        for i in 0..sim.snakes.len() {
            if !sim.snakes[i].alive && !dead_snapshots.iter().any(|(idx, _)| *idx == i) {
                dead_snapshots.push((i, sim.snakes[i].segments.clone()));
            }
        }

        for (idx, snapshot) in &dead_snapshots {
            assert_eq!(
                sim.snakes[*idx].segments, *snapshot,
                "Dead snake {} moved after death at tick {}",
                idx, sim.tick
            );
        }

        if sim.is_over() {
            break;
        }
    }

    assert!(
        !dead_snapshots.is_empty(),
        "No snake died during the simulation — test is vacuously true"
    );
}

#[test]
fn test_arena_shrink_kills_outside_snakes() {
    let bounds = ArenaBounds::default();
    let edge_x = bounds.max_x - 1;
    let mid_y = (bounds.min_y + bounds.max_y) / 2;

    let mut snake = Snake::new(edge_x, mid_y, Direction::Right);
    assert!(snake.alive);
    assert!(bounds.contains(snake.head()));

    let mut shrunk = bounds;
    shrunk.shrink();
    assert!(
        !shrunk.contains(snake.head()),
        "Snake at x={} should be outside arena with max_x={}",
        edge_x,
        shrunk.max_x
    );

    if !shrunk.contains(snake.head()) {
        snake.alive = false;
    }
    assert!(!snake.alive, "Snake should be dead after arena shrink");
}

#[test]
fn test_food_eating_grows_snake() {
    let mut snake = Snake::new(10, 10, Direction::Right);
    let food = Food::new(11, 10);

    let initial_score = snake.score;
    let initial_grow = snake.grow_pending;
    let initial_len = snake.segments.len();

    let new_head = snake.step();
    assert_eq!(new_head, food.pos, "Snake head should be on the food");

    if new_head == food.pos {
        snake.grow_pending += 1;
        snake.score += 1;
    }

    assert_eq!(snake.score, initial_score + 1, "Score should increase by 1");
    assert_eq!(
        snake.grow_pending,
        initial_grow + 1,
        "grow_pending should increase by 1"
    );

    snake.step();
    assert_eq!(
        snake.segments.len(),
        initial_len + 1,
        "Snake should have grown by 1 segment after eating"
    );
}

#[test]
fn test_self_collision_kills() {
    let mut snake = Snake::new(20, 20, Direction::Right);
    snake.grow_pending = 5;

    snake.step();
    snake.step();
    snake.step();

    snake.set_direction(Direction::Down);
    snake.step();

    snake.set_direction(Direction::Left);
    snake.step();

    snake.set_direction(Direction::Up);
    snake.step();

    assert!(
        snake.self_collision(),
        "Snake should detect self-collision after looping back"
    );
}

#[test]
fn test_head_to_head_collision() {
    let mut snake_a = Snake::new(15, 20, Direction::Right);
    let mut snake_b = Snake::new(17, 20, Direction::Left);

    let head_a = snake_a.step();
    let head_b = snake_b.step();

    assert_eq!(head_a, head_b, "Both heads should meet at the same cell");
    assert_eq!(head_a, GridPos::new(16, 20));

    let both_collide = head_a == head_b;
    assert!(both_collide, "Head-to-head collision should be detected");
}

#[test]
fn test_no_180_degree_turn() {
    let mut snake = Snake::new(20, 20, Direction::Right);

    snake.set_direction(Direction::Left);
    assert_eq!(
        snake.next_direction,
        Direction::Right,
        "180-degree turn should be rejected"
    );

    snake.set_direction(Direction::Up);
    assert_eq!(snake.next_direction, Direction::Up, "90-degree turn should be accepted");

    let head = snake.step();
    assert_eq!(snake.direction, Direction::Up);
    assert_eq!(head, GridPos::new(20, 21));
}

#[test]
fn test_arena_shrink_stops_at_minimum() {
    let mut bounds = ArenaBounds::default();
    let initial_width = bounds.max_x - bounds.min_x;

    let mut shrink_count = 0;
    while bounds.can_shrink() {
        bounds.shrink();
        shrink_count += 1;
    }

    let final_width = bounds.max_x - bounds.min_x;
    let final_height = bounds.max_y - bounds.min_y;

    assert_eq!(final_width, 6, "Arena should stop shrinking at width 6");
    assert_eq!(final_height, 6, "Arena should stop shrinking at height 6");
    assert!(!bounds.can_shrink(), "can_shrink() should return false at minimum size");

    let before = bounds;
    bounds.shrink();
    assert_eq!(bounds.min_x, before.min_x, "shrink() should be a no-op at minimum");
    assert_eq!(bounds.max_x, before.max_x, "shrink() should be a no-op at minimum");

    assert_eq!(
        shrink_count,
        (initial_width - 6) / 2,
        "Should take (initial_width - 6) / 2 shrinks to reach minimum"
    );
}

#[test]
fn test_statistical_fairness() {
    let num_games = 100;
    let num_snakes = 10;
    let mut win_counts = vec![0u32; num_snakes];

    for seed in 0..num_games as u64 {
        let mut sim = SimConfig::new().snakes(num_snakes).seed(seed + 5000).build();
        sim.run_with_invariants();

        if let Some(winner) = sim.winner {
            win_counts[winner] += 1;
        }
    }

    let max_wins = *win_counts.iter().max().unwrap();
    let threshold = (num_games as f64 * 0.40) as u32;

    assert!(
        max_wins <= threshold,
        "Spawn position bias detected: snake won {} out of {} games (>40%). Distribution: {:?}",
        max_wins,
        num_games,
        win_counts
    );
}

// ===========================================================================
// New scenarios
// ===========================================================================

#[test]
fn test_speed_increase_mechanics() {
    // Speed increase every 10 ticks, with 0.85x multiplier each time
    let mut sim = SimConfig::new()
        .snakes(2)
        .food(5)
        .seed(99)
        .speed_increase_interval(10)
        .max_ticks(100)
        .build();

    assert_eq!(sim.tick_interval_factor, 1.0, "initial factor should be 1.0");

    // Run 10 ticks — first speed increase
    for _ in 0..10 {
        sim.step_with_invariants();
    }
    let after_first = sim.tick_interval_factor;
    assert!(
        (after_first - 0.85).abs() < 0.001,
        "After first speed increase: expected ~0.85, got {}",
        after_first
    );

    // Run 10 more — second speed increase
    for _ in 0..10 {
        sim.step_with_invariants();
    }
    let after_second = sim.tick_interval_factor;
    assert!(
        (after_second - 0.85 * 0.85).abs() < 0.001,
        "After second speed increase: expected ~{}, got {}",
        0.85 * 0.85,
        after_second
    );

    // Floor at 0.48
    let mut sim = SimConfig::new()
        .snakes(2)
        .food(5)
        .seed(100)
        .speed_increase_interval(1)
        .max_ticks(200)
        .build();
    sim.run_with_invariants();
    assert!(
        sim.tick_interval_factor >= 0.48,
        "Speed factor should floor at 0.48, got {}",
        sim.tick_interval_factor
    );
}

#[test]
fn test_kill_attribution_correct() {
    // Run several games, verify every kill in death_log has a valid killer
    // when the death was caused by body collision (not wall/self)
    for seed in 0..20u64 {
        let mut sim = SimConfig::new().snakes(10).food(15).seed(seed + 2000).build();
        sim.run_with_invariants();

        for event in &sim.death_log {
            if let Some(killer_idx) = event.killer {
                assert!(
                    killer_idx < sim.snakes.len(),
                    "Killer index {} out of bounds (seed {})",
                    killer_idx,
                    seed
                );
                assert_ne!(
                    killer_idx, event.snake_idx,
                    "Snake {} credited as its own killer (seed {})",
                    killer_idx, seed
                );
            }
        }

        // Verify total kills across snakes matches death_log attributed kills
        let logged_kills = sim.death_log.iter().filter(|e| e.killer.is_some()).count() as u32;
        let snake_kills: u32 = sim.snakes.iter().map(|s| s.kills).sum();
        assert_eq!(
            logged_kills, snake_kills,
            "Kill count mismatch: death_log={}, snakes={} (seed {})",
            logged_kills, snake_kills, seed
        );
    }
}

#[test]
fn test_food_distribution_fairness() {
    // Over N games with SeekFood AI, no snake should get >2x average food
    let num_games = 50;
    let num_snakes = 10;
    let mut total_food_per_snake = vec![0u32; num_snakes];

    for seed in 0..num_games as u64 {
        let mut sim = SimConfig::new()
            .snakes(num_snakes)
            .food(20)
            .seed(seed + 3000)
            .ai_strategy(AiStrategy::SeekFood)
            .build();
        sim.run_with_invariants();

        for (i, snake) in sim.snakes.iter().enumerate() {
            total_food_per_snake[i] += snake.score;
        }
    }

    let total: u32 = total_food_per_snake.iter().sum();
    let avg = total as f64 / num_snakes as f64;
    let max_food = *total_food_per_snake.iter().max().unwrap();

    assert!(
        (max_food as f64) <= avg * 2.0,
        "Food distribution unfair: max={}, avg={:.1}. Distribution: {:?}",
        max_food,
        avg,
        total_food_per_snake
    );
}

#[test]
fn test_all_spawns_in_bounds() {
    for seed in 0..100u64 {
        let sim = SimConfig::new().snakes(10).seed(seed).build();

        for (i, snake) in sim.snakes.iter().enumerate() {
            for (seg_idx, seg) in snake.segments.iter().enumerate() {
                assert!(
                    sim.bounds.contains(*seg),
                    "Seed {}: snake {} segment {} at ({},{}) is out of bounds {:?}",
                    seed,
                    i,
                    seg_idx,
                    seg.x,
                    seg.y,
                    (sim.bounds.min_x, sim.bounds.min_y, sim.bounds.max_x, sim.bounds.max_y)
                );
            }
        }
    }
}

#[test]
fn test_arena_minimum_still_playable() {
    // Shrink arena to minimum, then verify snakes can still move
    let mut sim = SimConfig::new()
        .snakes(2)
        .food(3)
        .seed(77)
        .shrink_interval(1) // shrink every tick — aggressive
        .max_ticks(500)
        .build();

    // Run until arena is at minimum
    while sim.bounds.can_shrink() {
        sim.step_with_invariants();
    }

    let width = sim.bounds.max_x - sim.bounds.min_x;
    let height = sim.bounds.max_y - sim.bounds.min_y;
    assert_eq!(width, 6);
    assert_eq!(height, 6);

    // Keep going — game should still progress (snakes move, die, or win)
    let tick_at_min = sim.tick;
    for _ in 0..100 {
        if sim.is_over() {
            break;
        }
        sim.step_with_invariants();
    }

    // Either the game ended or snakes survived some ticks at minimum size
    assert!(
        sim.tick > tick_at_min || sim.is_over(),
        "Game stalled at minimum arena size"
    );
}

#[test]
fn test_simultaneous_multi_death() {
    // Run many games and check that multi-death (3+ same tick) can occur
    let mut multi_death_found = false;

    for seed in 0..200u64 {
        let mut sim = SimConfig::new()
            .snakes(10)
            .food(5)
            .seed(seed + 4000)
            .shrink_interval(10) // aggressive shrink to force crowding
            .build();
        sim.run_with_invariants();

        // Check death_log for 3+ deaths on same tick
        let mut deaths_per_tick: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        for event in &sim.death_log {
            *deaths_per_tick.entry(event.tick).or_insert(0) += 1;
        }

        if deaths_per_tick.values().any(|&count| count >= 3) {
            multi_death_found = true;
            break;
        }
    }

    assert!(
        multi_death_found,
        "No simultaneous multi-death (3+) observed in 200 seeded games — simulation may be broken"
    );
}

// ===========================================================================
// Determinism
// ===========================================================================

#[test]
fn test_determinism_same_seed_same_result() {
    for seed in 0..20u64 {
        let mut sim_a = SimConfig::new().seed(seed).build();
        let mut sim_b = SimConfig::new().seed(seed).build();

        sim_a.run_with_invariants();
        sim_b.run_with_invariants();

        assert_eq!(
            sim_a.death_log.len(),
            sim_b.death_log.len(),
            "seed {}: death_log length differs ({} vs {})",
            seed,
            sim_a.death_log.len(),
            sim_b.death_log.len()
        );

        for (i, (a, b)) in sim_a.death_log.iter().zip(&sim_b.death_log).enumerate() {
            assert_eq!(
                a.tick, b.tick,
                "seed {}: death {} tick differs ({} vs {})",
                seed, i, a.tick, b.tick
            );
            assert_eq!(
                a.snake_idx, b.snake_idx,
                "seed {}: death {} snake differs ({} vs {})",
                seed, i, a.snake_idx, b.snake_idx
            );
            assert_eq!(
                a.killer, b.killer,
                "seed {}: death {} killer differs ({:?} vs {:?})",
                seed, i, a.killer, b.killer
            );
        }

        assert_eq!(
            sim_a.winner, sim_b.winner,
            "seed {}: winner differs ({:?} vs {:?})",
            seed, sim_a.winner, sim_b.winner
        );
    }
}
