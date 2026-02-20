//! Reusable headless game simulation harness.
//!
//! Builder-pattern config for running full battle royale games as pure data.
//! Deterministic (seeded RNG), fast (no rendering), composable.

use shared::constants::*;
use shared::game::*;

use super::invariants::{self, TickSnapshot};

// ---------------------------------------------------------------------------
// Seeded RNG (PCG-style, deterministic across platforms)
// ---------------------------------------------------------------------------

pub struct TestRng {
    state: u64,
}

impl TestRng {
    pub fn new(seed: u64) -> Self {
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
        assert!(max > min, "range: max must be > min");
        min + (self.next_u32() % (max - min) as u32) as i32
    }
}

// ---------------------------------------------------------------------------
// AI strategies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AiStrategy {
    /// Pick a random non-opposite direction each tick.
    Random,
    /// Move toward the nearest food (greedy).
    SeekFood,
    /// Move toward the nearest wall (suicidal — useful for testing deaths).
    SeekWall,
    /// Never change direction.
    Stationary,
}

// ---------------------------------------------------------------------------
// Simulation config (builder pattern)
// ---------------------------------------------------------------------------

pub struct SimConfig {
    pub num_snakes: usize,
    pub num_food: usize,
    pub seed: u64,
    pub shrink_interval: u32,
    pub speed_increase_interval: Option<u32>,
    pub max_ticks: u32,
    pub ai_strategy: AiStrategy,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            num_snakes: 10,
            num_food: 15,
            seed: 42,
            shrink_interval: 30,
            speed_increase_interval: None,
            max_ticks: 2000,
            ai_strategy: AiStrategy::Random,
        }
    }
}

impl SimConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snakes(mut self, n: usize) -> Self {
        self.num_snakes = n;
        self
    }

    pub fn food(mut self, n: usize) -> Self {
        self.num_food = n;
        self
    }

    pub fn seed(mut self, s: u64) -> Self {
        self.seed = s;
        self
    }

    pub fn shrink_interval(mut self, interval: u32) -> Self {
        self.shrink_interval = interval;
        self
    }

    pub fn speed_increase_interval(mut self, interval: u32) -> Self {
        self.speed_increase_interval = Some(interval);
        self
    }

    pub fn max_ticks(mut self, ticks: u32) -> Self {
        self.max_ticks = ticks;
        self
    }

    pub fn ai_strategy(mut self, strategy: AiStrategy) -> Self {
        self.ai_strategy = strategy;
        self
    }

    pub fn build(self) -> GameSim {
        GameSim::from_config(self)
    }
}

// ---------------------------------------------------------------------------
// Simulation engine
// ---------------------------------------------------------------------------

pub struct GameSim {
    pub snakes: Vec<Snake>,
    pub food: Vec<Food>,
    pub bounds: ArenaBounds,
    pub rng: TestRng,
    pub tick: u32,
    pub shrink_interval: u32,
    pub speed_increase_interval: Option<u32>,
    pub tick_interval_factor: f32,
    pub max_ticks: u32,
    pub winner: Option<usize>,
    pub death_log: Vec<DeathEvent>,
    ai_strategy: AiStrategy,
}

pub struct DeathEvent {
    pub tick: u32,
    pub snake_idx: usize,
    pub killer: Option<usize>,
}

impl GameSim {
    /// Convenience constructor matching the old API.
    #[allow(dead_code)]
    pub fn new(num_snakes: usize, num_food: usize, seed: u64, shrink_interval: u32) -> Self {
        SimConfig::new()
            .snakes(num_snakes)
            .food(num_food)
            .seed(seed)
            .shrink_interval(shrink_interval)
            .build()
    }

    fn from_config(cfg: SimConfig) -> Self {
        let mut rng = TestRng::new(cfg.seed);
        let bounds = ArenaBounds::default();

        let cx = (bounds.min_x + bounds.max_x) / 2;
        let cy = (bounds.min_y + bounds.max_y) / 2;
        let radius = ((bounds.max_x - bounds.min_x) / 3).max(8);

        let mut snakes = Vec::with_capacity(cfg.num_snakes);
        for i in 0..cfg.num_snakes {
            let angle_steps = cfg.num_snakes.max(1);
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
            shrink_interval: cfg.shrink_interval,
            speed_increase_interval: cfg.speed_increase_interval,
            tick_interval_factor: 1.0,
            max_ticks: cfg.max_ticks,
            winner: None,
            death_log: Vec::new(),
            ai_strategy: cfg.ai_strategy,
        };

        for _ in 0..cfg.num_food {
            sim.spawn_food();
        }

        sim
    }

    pub fn spawn_food(&mut self) {
        for _ in 0..100 {
            let x = self.rng.range(self.bounds.min_x, self.bounds.max_x);
            let y = self.rng.range(self.bounds.min_y, self.bounds.max_y);
            let pos = GridPos::new(x, y);

            let on_snake = self.snakes.iter().any(|s| s.occupies(pos));
            let on_food = self.food.iter().any(|f| f.pos == pos);
            if !on_snake && !on_food {
                self.food.push(Food::new(x, y));
                return;
            }
        }
    }

    pub fn alive_count(&self) -> usize {
        self.snakes.iter().filter(|s| s.alive).count()
    }

    pub fn is_over(&self) -> bool {
        self.alive_count() <= 1
    }

    /// Pick direction based on AI strategy.
    fn pick_direction(&mut self, snake_idx: usize) -> Direction {
        match self.ai_strategy {
            AiStrategy::Random => self.random_direction(snake_idx),
            AiStrategy::SeekFood => self.seek_food_direction(snake_idx),
            AiStrategy::SeekWall => self.seek_wall_direction(snake_idx),
            AiStrategy::Stationary => self.snakes[snake_idx].direction,
        }
    }

    fn random_direction(&mut self, snake_idx: usize) -> Direction {
        let current = self.snakes[snake_idx].direction;
        let choices: Vec<Direction> = Direction::ALL
            .iter()
            .copied()
            .filter(|&d| d != current.opposite())
            .collect();
        choices[self.rng.next_u32() as usize % choices.len()]
    }

    fn seek_food_direction(&mut self, snake_idx: usize) -> Direction {
        let head = self.snakes[snake_idx].head();
        let current = self.snakes[snake_idx].direction;

        if let Some(nearest) = self.food.iter().min_by_key(|f| f.pos.distance(head)) {
            let dx = nearest.pos.x - head.x;
            let dy = nearest.pos.y - head.y;

            let preferred = if dx.abs() > dy.abs() {
                if dx > 0 { Direction::Right } else { Direction::Left }
            } else {
                if dy > 0 { Direction::Up } else { Direction::Down }
            };

            if preferred != current.opposite() {
                return preferred;
            }
        }

        // Fallback to random
        self.random_direction(snake_idx)
    }

    fn seek_wall_direction(&mut self, snake_idx: usize) -> Direction {
        let head = self.snakes[snake_idx].head();
        let current = self.snakes[snake_idx].direction;

        let dist_left = head.x - self.bounds.min_x;
        let dist_right = self.bounds.max_x - 1 - head.x;
        let dist_down = head.y - self.bounds.min_y;
        let dist_up = self.bounds.max_y - 1 - head.y;

        let min_dist = dist_left.min(dist_right).min(dist_down).min(dist_up);
        let preferred = if min_dist == dist_left {
            Direction::Left
        } else if min_dist == dist_right {
            Direction::Right
        } else if min_dist == dist_down {
            Direction::Down
        } else {
            Direction::Up
        };

        if preferred != current.opposite() {
            preferred
        } else {
            self.random_direction(snake_idx)
        }
    }

    /// Run one simulation tick.
    pub fn step(&mut self) {
        self.tick += 1;

        // Arena shrink
        if self.shrink_interval > 0 && self.tick % self.shrink_interval == 0 {
            self.bounds.shrink();
        }

        // Speed increase (simulate: factor *= 0.85, floor at 0.06/0.125 ≈ 0.48)
        if let Some(interval) = self.speed_increase_interval {
            if interval > 0 && self.tick % interval == 0 {
                self.tick_interval_factor = (self.tick_interval_factor * 0.85).max(0.48);
            }
        }

        // Choose directions
        let n = self.snakes.len();
        let mut directions = Vec::with_capacity(n);
        for i in 0..n {
            if self.snakes[i].alive {
                let dir = self.pick_direction(i);
                directions.push(Some(dir));
            } else {
                directions.push(None);
            }
        }

        // Apply directions and move
        for i in 0..n {
            if let Some(dir) = directions[i] {
                self.snakes[i].set_direction(dir);
                self.snakes[i].step();
            }
        }

        // Collision detection
        let mut kills = vec![false; n];
        let mut killers = vec![None; n];

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

            // Body collision with others
            for j in 0..n {
                if i == j || !self.snakes[j].alive {
                    continue;
                }
                if self.snakes[j].body_collision(head) {
                    kills[i] = true;
                    killers[i] = Some(j);
                    self.snakes[j].kills += 1;
                    break;
                }
            }
        }

        // Head-to-head
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
                self.death_log.push(DeathEvent {
                    tick: self.tick,
                    snake_idx: i,
                    killer: killers[i],
                });
            }
        }

        // Food eating
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
        eaten_indices.sort_unstable();
        eaten_indices.dedup();
        for &fi in eaten_indices.iter().rev() {
            self.food.swap_remove(fi);
        }
        for _ in 0..eaten_indices.len() {
            self.spawn_food();
        }

        // Win condition
        if self.alive_count() <= 1 {
            self.winner = self.snakes.iter().position(|s| s.alive);
        }
    }

    /// Run the full simulation until game over or max ticks (no invariant checks).
    #[allow(dead_code)]
    pub fn run(&mut self) -> u32 {
        while self.tick < self.max_ticks && !self.is_over() {
            self.step();
        }
        self.tick
    }

    /// Capture current state for invariant comparison.
    pub fn snapshot(&self) -> TickSnapshot {
        TickSnapshot::capture(&self.snakes, &self.bounds)
    }

    /// Run one tick with all 6 invariant checks.
    pub fn step_with_invariants(&mut self) {
        let before = self.snapshot();
        self.step();
        invariants::check_all(&before, &self.snakes, &self.bounds, self.tick);
    }

    /// Run the full simulation with per-tick invariant checks.
    pub fn run_with_invariants(&mut self) -> u32 {
        while self.tick < self.max_ticks && !self.is_over() {
            self.step_with_invariants();
        }
        self.tick
    }
}
