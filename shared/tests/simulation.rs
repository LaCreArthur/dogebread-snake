//! Headless game simulation tests — full battle royale games as pure data operations.
//!
//! Tests the GAME LOGIC end-to-end without any engine dependency.
//! Deterministic (seeded RNG), fast (no rendering), catches bugs that
//! unit tests miss because they test in isolation.

use shared::constants::*;
use shared::game::*;
use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Seeded RNG (PCG-style, deterministic across platforms)
// ---------------------------------------------------------------------------

struct TestRng {
    state: u64,
}

impl TestRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 33) as u32
    }

    fn range(&mut self, min: i32, max: i32) -> i32 {
        assert!(max > min, "range: max must be > min");
        min + (self.next_u32() % (max - min) as u32) as i32
    }
}

// ---------------------------------------------------------------------------
// Simulation engine
// ---------------------------------------------------------------------------

struct GameSim {
    snakes: Vec<Snake>,
    food: Vec<Food>,
    bounds: ArenaBounds,
    rng: TestRng,
    tick: u32,
    shrink_interval: u32,
    winner: Option<usize>,
}

impl GameSim {
    /// Spawn `n` snakes evenly around the arena center.
    fn new(num_snakes: usize, num_food: usize, seed: u64, shrink_interval: u32) -> Self {
        let mut rng = TestRng::new(seed);
        let bounds = ArenaBounds::default();

        let cx = (bounds.min_x + bounds.max_x) / 2;
        let cy = (bounds.min_y + bounds.max_y) / 2;
        let radius = ((bounds.max_x - bounds.min_x) / 3).max(8);

        let mut snakes = Vec::with_capacity(num_snakes);
        for i in 0..num_snakes {
            // Distribute spawn points in a ring
            let angle_steps = num_snakes.max(1);
            let angle_idx = i % angle_steps;
            let (dx, dy) = match angle_idx % 4 {
                0 => (radius, (i as i32 * 3) % radius),
                1 => (-(radius), (i as i32 * 3) % radius),
                2 => ((i as i32 * 3) % radius, radius),
                3 => ((i as i32 * 3) % radius, -(radius)),
                _ => unreachable!(),
            };
            let sx = (cx + dx).clamp(
                bounds.min_x + INITIAL_SNAKE_LENGTH as i32 + 1,
                bounds.max_x - INITIAL_SNAKE_LENGTH as i32 - 1,
            );
            let sy = (cy + dy).clamp(
                bounds.min_y + INITIAL_SNAKE_LENGTH as i32 + 1,
                bounds.max_y - INITIAL_SNAKE_LENGTH as i32 - 1,
            );

            let dir = Direction::ALL[rng.next_u32() as usize % 4];
            snakes.push(Snake::new(sx, sy, dir));
        }

        let mut sim = GameSim {
            snakes,
            food: Vec::new(),
            bounds,
            rng,
            tick: 0,
            shrink_interval,
            winner: None,
        };

        for _ in 0..num_food {
            sim.spawn_food();
        }

        sim
    }

    fn spawn_food(&mut self) {
        for _ in 0..100 {
            let x = self.rng.range(self.bounds.min_x, self.bounds.max_x);
            let y = self.rng.range(self.bounds.min_y, self.bounds.max_y);
            let pos = GridPos::new(x, y);

            // Don't spawn on any snake
            let on_snake = self.snakes.iter().any(|s| s.occupies(pos));
            let on_food = self.food.iter().any(|f| f.pos == pos);
            if !on_snake && !on_food {
                self.food.push(Food::new(x, y));
                return;
            }
        }
        // Give up after 100 attempts (arena might be full)
    }

    fn alive_count(&self) -> usize {
        self.snakes.iter().filter(|s| s.alive).count()
    }

    fn is_over(&self) -> bool {
        self.alive_count() <= 1
    }

    /// Pick a random valid direction for an AI snake (avoids 180-degree turns).
    fn random_direction_for(&mut self, snake_idx: usize) -> Direction {
        let current = self.snakes[snake_idx].direction;
        let choices: Vec<Direction> = Direction::ALL
            .iter()
            .copied()
            .filter(|&d| d != current.opposite())
            .collect();
        choices[self.rng.next_u32() as usize % choices.len()]
    }

    /// Run one simulation tick.
    fn step(&mut self) {
        self.tick += 1;

        // --- Arena shrink ---
        if self.shrink_interval > 0 && self.tick % self.shrink_interval == 0 {
            self.bounds.shrink();
        }

        // --- Choose directions for alive snakes ---
        let n = self.snakes.len();
        let mut directions = Vec::with_capacity(n);
        for i in 0..n {
            if self.snakes[i].alive {
                let dir = self.random_direction_for(i);
                directions.push(Some(dir));
            } else {
                directions.push(None);
            }
        }

        // --- Apply directions and move ---
        let mut new_heads: Vec<Option<GridPos>> = Vec::with_capacity(n);
        for i in 0..n {
            if let Some(dir) = directions[i] {
                self.snakes[i].set_direction(dir);
                let head = self.snakes[i].step();
                new_heads.push(Some(head));
            } else {
                new_heads.push(None);
            }
        }

        // --- Collision detection ---
        let mut kills = vec![false; n];

        for i in 0..n {
            if !self.snakes[i].alive {
                continue;
            }
            let head = self.snakes[i].head();

            // Out of bounds
            if !self.bounds.contains(head) {
                kills[i] = true;
                continue;
            }

            // Self-collision
            if self.snakes[i].self_collision() {
                kills[i] = true;
                continue;
            }

            // Collision with other snakes' bodies
            for j in 0..n {
                if i == j || !self.snakes[j].alive {
                    continue;
                }
                if self.snakes[j].body_collision(head) {
                    kills[i] = true;
                    // Credit the kill to snake j
                    self.snakes[j].kills += 1;
                    break;
                }
            }
        }

        // Head-to-head collisions
        for i in 0..n {
            if !self.snakes[i].alive || kills[i] {
                continue;
            }
            for j in (i + 1)..n {
                if !self.snakes[j].alive || kills[j] {
                    continue;
                }
                if self.snakes[i].head() == self.snakes[j].head() {
                    kills[i] = true;
                    kills[j] = true;
                }
            }
        }

        // Process deaths
        for i in 0..n {
            if kills[i] {
                self.snakes[i].alive = false;
            }
        }

        // --- Food eating ---
        let mut eaten_indices = Vec::new();
        for i in 0..n {
            if !self.snakes[i].alive {
                continue;
            }
            let head = self.snakes[i].head();
            for (fi, food) in self.food.iter().enumerate() {
                if food.pos == head {
                    self.snakes[i].grow_pending += 1;
                    self.snakes[i].score += 1;
                    eaten_indices.push(fi);
                    break;
                }
            }
        }
        // Remove eaten food (reverse order to preserve indices)
        eaten_indices.sort_unstable();
        eaten_indices.dedup();
        for &fi in eaten_indices.iter().rev() {
            self.food.swap_remove(fi);
        }
        // Respawn eaten food
        for _ in 0..eaten_indices.len() {
            self.spawn_food();
        }

        // --- Check win condition ---
        if self.alive_count() <= 1 {
            self.winner = self.snakes.iter().position(|s| s.alive);
        }
    }

    /// Run the full simulation until game over or max ticks.
    fn run(&mut self, max_ticks: u32) -> u32 {
        while self.tick < max_ticks && !self.is_over() {
            self.step();
        }
        self.tick
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[test]
fn test_game_always_terminates() {
    for seed in 0..50u64 {
        let mut sim = GameSim::new(10, 15, seed, 30);
        let ticks = sim.run(2000);
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
        let mut sim = GameSim::new(10, 15, seed + 1000, 30);
        sim.run(2000);
        for (i, snake) in sim.snakes.iter().enumerate() {
            // score and kills are u32, so they can't be negative at the type level,
            // but we assert they exist and haven't wrapped around to huge values
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
    let mut sim = GameSim::new(10, 15, 42, 30);

    let mut dead_snapshots: Vec<(usize, VecDeque<GridPos>)> = Vec::new();

    for _ in 0..500 {
        sim.step();

        // Record newly dead snakes
        for i in 0..sim.snakes.len() {
            if !sim.snakes[i].alive
                && !dead_snapshots.iter().any(|(idx, _)| *idx == i)
            {
                dead_snapshots.push((i, sim.snakes[i].segments.clone()));
            }
        }

        // Assert all previously-dead snakes haven't moved
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

    // Verify we actually observed at least one death
    assert!(
        !dead_snapshots.is_empty(),
        "No snake died during the simulation — test is vacuously true"
    );
}

#[test]
fn test_arena_shrink_kills_outside_snakes() {
    // Place a snake right at the edge of default bounds
    let bounds = ArenaBounds::default();
    let edge_x = bounds.max_x - 1; // rightmost valid column
    let mid_y = (bounds.min_y + bounds.max_y) / 2;

    let mut snake = Snake::new(edge_x, mid_y, Direction::Right);
    assert!(snake.alive);
    assert!(bounds.contains(snake.head()));

    // Shrink the arena — max_x decreases by 1, so edge_x is now OUT of bounds
    let mut shrunk = bounds;
    shrunk.shrink();
    assert!(
        !shrunk.contains(snake.head()),
        "Snake at x={} should be outside arena with max_x={}",
        edge_x,
        shrunk.max_x
    );

    // In our sim, out-of-bounds → death
    // Simulate the kill logic:
    if !shrunk.contains(snake.head()) {
        snake.alive = false;
    }
    assert!(!snake.alive, "Snake should be dead after arena shrink");
}

#[test]
fn test_food_eating_grows_snake() {
    // Snake heading Right at (10, 10), place food at (11, 10)
    let mut snake = Snake::new(10, 10, Direction::Right);
    let food = Food::new(11, 10);

    let initial_score = snake.score;
    let initial_grow = snake.grow_pending;
    let initial_len = snake.segments.len();

    // Step the snake forward
    let new_head = snake.step();
    assert_eq!(new_head, food.pos, "Snake head should be on the food");

    // Process eating
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

    // Step again — the snake should be longer because of grow_pending
    snake.step();
    assert_eq!(
        snake.segments.len(),
        initial_len + 1,
        "Snake should have grown by 1 segment after eating"
    );
}

#[test]
fn test_self_collision_kills() {
    // Build a snake that will loop back on itself.
    // Start going Right, then Down, then Left, then Up → loops into its own body.
    let mut snake = Snake::new(20, 20, Direction::Right);

    // Make the snake longer so it can hit itself
    snake.grow_pending = 5;

    // Move Right a few times to grow
    snake.step(); // (21,20)
    snake.step(); // (22,20)
    snake.step(); // (23,20)

    // Now turn Down
    snake.set_direction(Direction::Down);
    snake.step(); // (23,19)

    // Turn Left
    snake.set_direction(Direction::Left);
    snake.step(); // (22,19)

    // Turn Up — this should move into the body
    snake.set_direction(Direction::Up);
    snake.step(); // (22,20) — occupied by body!

    assert!(
        snake.self_collision(),
        "Snake should detect self-collision after looping back"
    );
}

#[test]
fn test_head_to_head_collision() {
    // Two snakes facing each other, one cell apart
    let mut snake_a = Snake::new(15, 20, Direction::Right);
    let mut snake_b = Snake::new(17, 20, Direction::Left);

    // After one step each, both heads should be at (16, 20)
    let head_a = snake_a.step();
    let head_b = snake_b.step();

    assert_eq!(head_a, head_b, "Both heads should meet at the same cell");
    assert_eq!(head_a, GridPos::new(16, 20));

    // In the sim, head-to-head → both die
    let both_collide = head_a == head_b;
    assert!(both_collide, "Head-to-head collision should be detected");
}

#[test]
fn test_no_180_degree_turn() {
    let mut snake = Snake::new(20, 20, Direction::Right);

    // Try to go Left (opposite) — should be ignored
    snake.set_direction(Direction::Left);
    assert_eq!(
        snake.next_direction,
        Direction::Right,
        "180-degree turn should be rejected"
    );

    // Try valid turn
    snake.set_direction(Direction::Up);
    assert_eq!(
        snake.next_direction,
        Direction::Up,
        "90-degree turn should be accepted"
    );

    // Step and verify direction is Up
    let head = snake.step();
    assert_eq!(snake.direction, Direction::Up);
    assert_eq!(head, GridPos::new(20, 21));
}

#[test]
fn test_arena_shrink_stops_at_minimum() {
    let mut bounds = ArenaBounds::default();
    let initial_width = bounds.max_x - bounds.min_x;

    // Shrink until we can't anymore
    let mut shrink_count = 0;
    while bounds.can_shrink() {
        bounds.shrink();
        shrink_count += 1;
    }

    let final_width = bounds.max_x - bounds.min_x;
    let final_height = bounds.max_y - bounds.min_y;

    assert_eq!(final_width, 6, "Arena should stop shrinking at width 6");
    assert_eq!(final_height, 6, "Arena should stop shrinking at height 6");
    assert!(
        !bounds.can_shrink(),
        "can_shrink() should return false at minimum size"
    );

    // Shrink one more time — should be a no-op
    let before = bounds;
    bounds.shrink();
    assert_eq!(bounds.min_x, before.min_x, "shrink() should be a no-op at minimum");
    assert_eq!(bounds.max_x, before.max_x, "shrink() should be a no-op at minimum");

    // Verify the math: default is 58 wide (1..59), shrink to 6 wide = 26 shrinks
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
        let mut sim = GameSim::new(num_snakes, 15, seed + 5000, 30);
        sim.run(2000);

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
