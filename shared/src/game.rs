use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::constants::*;

/// Cardinal direction for snake movement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn delta(self) -> IVec2 {
        match self {
            Direction::Up => IVec2::new(0, 1),
            Direction::Down => IVec2::new(0, -1),
            Direction::Left => IVec2::new(-1, 0),
            Direction::Right => IVec2::new(1, 0),
        }
    }

    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
}

/// Grid position (cell coordinates, not pixels)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert grid position to world (pixel) coordinates, centered on the grid
    pub fn to_world(self) -> Vec2 {
        let offset_x = (GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
        let offset_y = (GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;
        Vec2::new(
            self.x as f32 * CELL_SIZE - offset_x + CELL_SIZE / 2.0,
            self.y as f32 * CELL_SIZE - offset_y + CELL_SIZE / 2.0,
        )
    }

    /// Check if position is within playable area (inside the border walls)
    pub fn in_bounds(self) -> bool {
        self.x >= 1 && self.x < GRID_WIDTH - 1 && self.y >= 1 && self.y < GRID_HEIGHT - 1
    }

    /// Manhattan distance to another position
    pub fn distance(self, other: GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

/// Unique snake identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct SnakeId(pub u32);

/// Snake color (head color — body is slightly darker)
#[derive(Debug, Clone, Copy, Component)]
pub struct SnakeColor {
    pub head: Color,
    pub body: Color,
}

impl SnakeColor {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self {
            head: Color::srgb(r, g, b),
            body: Color::srgb(r * 0.7, g * 0.7, b * 0.7),
        }
    }

    /// Doge-themed snake colors
    pub fn palette(index: u32) -> Self {
        match index % 8 {
            0 => Self::new(0.91, 0.69, 0.29),  // Doge Gold (player)
            1 => Self::new(0.4, 0.85, 0.5),    // Lime Green
            2 => Self::new(0.95, 0.35, 0.35),  // Hot Red
            3 => Self::new(0.35, 0.55, 0.98),  // Electric Blue
            4 => Self::new(0.95, 0.45, 0.85),  // Hot Pink
            5 => Self::new(0.2, 0.95, 0.95),   // Bright Cyan
            6 => Self::new(0.98, 0.60, 0.15),  // Bright Orange
            7 => Self::new(0.75, 0.65, 0.98),  // Soft Purple
            _ => unreachable!(),
        }
    }
}

/// Marker: this snake is controlled by the local player
#[derive(Component)]
pub struct PlayerControlled;

/// Marker: this snake is AI-controlled
#[derive(Component)]
pub struct AiControlled;

/// The snake: head position + body segments
#[derive(Debug, Clone, Component)]
pub struct Snake {
    pub segments: VecDeque<GridPos>,
    pub direction: Direction,
    pub next_direction: Direction,
    pub alive: bool,
    pub grow_pending: usize,
    pub score: u32,
    pub kills: u32,
}

impl Snake {
    pub fn new(head_x: i32, head_y: i32, direction: Direction) -> Self {
        let delta = direction.opposite().delta();
        let mut segments = VecDeque::new();
        for i in 0..INITIAL_SNAKE_LENGTH {
            segments.push_back(GridPos::new(
                head_x + delta.x * i as i32,
                head_y + delta.y * i as i32,
            ));
        }
        Self {
            segments,
            direction,
            next_direction: direction,
            alive: true,
            grow_pending: 0,
            score: 0,
            kills: 0,
        }
    }

    pub fn head(&self) -> GridPos {
        self.segments[0]
    }

    /// Try to change direction (prevents 180-degree turns)
    pub fn set_direction(&mut self, dir: Direction) {
        if dir != self.direction.opposite() {
            self.next_direction = dir;
        }
    }

    /// Advance the snake one step. Returns the new head position.
    pub fn step(&mut self) -> GridPos {
        self.direction = self.next_direction;
        let delta = self.direction.delta();
        let new_head = GridPos::new(self.head().x + delta.x, self.head().y + delta.y);
        self.segments.push_front(new_head);

        if self.grow_pending > 0 {
            self.grow_pending -= 1;
        } else {
            self.segments.pop_back();
        }

        new_head
    }

    pub fn self_collision(&self) -> bool {
        let head = self.head();
        self.segments.iter().skip(1).any(|s| *s == head)
    }

    /// Check if a position collides with any body segment (not head)
    pub fn body_collision(&self, pos: GridPos) -> bool {
        self.segments.iter().skip(1).any(|s| *s == pos)
    }

    pub fn occupies(&self, pos: GridPos) -> bool {
        self.segments.iter().any(|s| *s == pos)
    }
}

/// Food item on the grid
#[derive(Debug, Clone, Copy, Component)]
pub struct Food {
    pub pos: GridPos,
}

impl Food {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            pos: GridPos::new(x, y),
        }
    }
}

/// Game state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameState {
    #[default]
    WaitingToStart,
    Countdown,
    Playing,
    GameOver,
}

/// Track alive snake count and rankings
#[derive(Resource, Default)]
pub struct MatchState {
    pub alive_count: u32,
    pub total_snakes: u32,
}

/// Dynamic arena bounds (shrinks over time in battle royale)
#[derive(Resource, Clone, Copy)]
pub struct ArenaBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32, // exclusive
    pub max_y: i32, // exclusive
}

impl Default for ArenaBounds {
    fn default() -> Self {
        Self {
            min_x: 1,
            min_y: 1,
            max_x: GRID_WIDTH - 1,
            max_y: GRID_HEIGHT - 1,
        }
    }
}

impl ArenaBounds {
    pub fn contains(&self, pos: GridPos) -> bool {
        pos.x >= self.min_x && pos.x < self.max_x && pos.y >= self.min_y && pos.y < self.max_y
    }

    /// Minimum size the arena can shrink to
    pub fn can_shrink(&self) -> bool {
        (self.max_x - self.min_x) > 6 && (self.max_y - self.min_y) > 6
    }

    pub fn shrink(&mut self) {
        if self.can_shrink() {
            self.min_x += 1;
            self.min_y += 1;
            self.max_x -= 1;
            self.max_y -= 1;
        }
    }

    /// Distance from position to nearest arena wall (0 = on the wall)
    pub fn wall_distance(&self, pos: GridPos) -> i32 {
        let dx = (pos.x - self.min_x).min(self.max_x - 1 - pos.x);
        let dy = (pos.y - self.min_y).min(self.max_y - 1 - pos.y);
        dx.min(dy)
    }
}

/// Timer for dead snake body cleanup
#[derive(Component)]
pub struct DeathTimer {
    pub timer: bevy::time::Timer,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Direction ──────────────────────────────────────────────────────

    #[test]
    fn direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Down.opposite(), Direction::Up);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::Right.opposite(), Direction::Left);
    }

    #[test]
    fn direction_opposite_is_involution() {
        for d in Direction::ALL {
            assert_eq!(d.opposite().opposite(), d);
        }
    }

    #[test]
    fn direction_delta() {
        assert_eq!(Direction::Up.delta(), IVec2::new(0, 1));
        assert_eq!(Direction::Down.delta(), IVec2::new(0, -1));
        assert_eq!(Direction::Left.delta(), IVec2::new(-1, 0));
        assert_eq!(Direction::Right.delta(), IVec2::new(1, 0));
    }

    #[test]
    fn direction_all_contains_four() {
        assert_eq!(Direction::ALL.len(), 4);
        assert!(Direction::ALL.contains(&Direction::Up));
        assert!(Direction::ALL.contains(&Direction::Down));
        assert!(Direction::ALL.contains(&Direction::Left));
        assert!(Direction::ALL.contains(&Direction::Right));
    }

    // ── GridPos ───────────────────────────────────────────────────────

    #[test]
    fn gridpos_new() {
        let p = GridPos::new(10, 20);
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 20);
    }

    #[test]
    fn gridpos_in_bounds_corners() {
        // Just inside each corner
        assert!(GridPos::new(1, 1).in_bounds());
        assert!(GridPos::new(GRID_WIDTH - 2, 1).in_bounds());
        assert!(GridPos::new(1, GRID_HEIGHT - 2).in_bounds());
        assert!(GridPos::new(GRID_WIDTH - 2, GRID_HEIGHT - 2).in_bounds());
    }

    #[test]
    fn gridpos_in_bounds_walls_are_out() {
        // On the wall (border row/col) — out of bounds
        assert!(!GridPos::new(0, 0).in_bounds());
        assert!(!GridPos::new(0, 30).in_bounds());
        assert!(!GridPos::new(30, 0).in_bounds());
        assert!(!GridPos::new(GRID_WIDTH - 1, 30).in_bounds());
        assert!(!GridPos::new(30, GRID_HEIGHT - 1).in_bounds());
    }

    #[test]
    fn gridpos_in_bounds_negative() {
        assert!(!GridPos::new(-1, 30).in_bounds());
        assert!(!GridPos::new(30, -1).in_bounds());
        assert!(!GridPos::new(-5, -5).in_bounds());
    }

    #[test]
    fn gridpos_in_bounds_just_outside() {
        assert!(!GridPos::new(GRID_WIDTH, 30).in_bounds());
        assert!(!GridPos::new(30, GRID_HEIGHT).in_bounds());
    }

    #[test]
    fn gridpos_distance_same_point() {
        let p = GridPos::new(5, 5);
        assert_eq!(p.distance(p), 0);
    }

    #[test]
    fn gridpos_distance_adjacent() {
        let a = GridPos::new(5, 5);
        assert_eq!(a.distance(GridPos::new(6, 5)), 1);
        assert_eq!(a.distance(GridPos::new(5, 6)), 1);
        assert_eq!(a.distance(GridPos::new(4, 5)), 1);
        assert_eq!(a.distance(GridPos::new(5, 4)), 1);
    }

    #[test]
    fn gridpos_distance_diagonal() {
        let a = GridPos::new(0, 0);
        let b = GridPos::new(3, 4);
        assert_eq!(a.distance(b), 7); // Manhattan: |3| + |4|
    }

    #[test]
    fn gridpos_distance_large() {
        let a = GridPos::new(0, 0);
        let b = GridPos::new(59, 59);
        assert_eq!(a.distance(b), 118);
    }

    #[test]
    fn gridpos_distance_symmetric() {
        let a = GridPos::new(3, 7);
        let b = GridPos::new(10, 2);
        assert_eq!(a.distance(b), b.distance(a));
    }

    #[test]
    fn gridpos_to_world_origin() {
        // GridPos(0,0) should map to bottom-left area
        let w = GridPos::new(0, 0).to_world();
        let offset_x = (GRID_WIDTH as f32 * CELL_SIZE) / 2.0;
        let offset_y = (GRID_HEIGHT as f32 * CELL_SIZE) / 2.0;
        let expected_x = 0.0 * CELL_SIZE - offset_x + CELL_SIZE / 2.0;
        let expected_y = 0.0 * CELL_SIZE - offset_y + CELL_SIZE / 2.0;
        assert!((w.x - expected_x).abs() < f32::EPSILON);
        assert!((w.y - expected_y).abs() < f32::EPSILON);
    }

    #[test]
    fn gridpos_to_world_center() {
        let center = GridPos::new(GRID_WIDTH / 2, GRID_HEIGHT / 2).to_world();
        // Center cell should be near pixel (0,0) + half-cell offset
        // (30 * 12 - 360 + 6, 30 * 12 - 360 + 6) = (6, 6)
        assert!((center.x - 6.0).abs() < f32::EPSILON);
        assert!((center.y - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gridpos_to_world_cell_spacing() {
        // Adjacent cells should differ by exactly CELL_SIZE
        let a = GridPos::new(5, 5).to_world();
        let b = GridPos::new(6, 5).to_world();
        assert!((b.x - a.x - CELL_SIZE).abs() < f32::EPSILON);
        assert!((b.y - a.y).abs() < f32::EPSILON);
    }

    // ── Snake ─────────────────────────────────────────────────────────

    #[test]
    fn snake_new_length() {
        let s = Snake::new(30, 30, Direction::Right);
        assert_eq!(s.segments.len(), INITIAL_SNAKE_LENGTH);
    }

    #[test]
    fn snake_new_head_position() {
        let s = Snake::new(30, 30, Direction::Right);
        assert_eq!(s.head(), GridPos::new(30, 30));
    }

    #[test]
    fn snake_new_segments_extend_opposite() {
        // Facing right → segments extend left from head
        let s = Snake::new(30, 30, Direction::Right);
        assert_eq!(s.segments[0], GridPos::new(30, 30));
        assert_eq!(s.segments[1], GridPos::new(29, 30));
        assert_eq!(s.segments[2], GridPos::new(28, 30));
        assert_eq!(s.segments[3], GridPos::new(27, 30));
        assert_eq!(s.segments[4], GridPos::new(26, 30));
    }

    #[test]
    fn snake_new_facing_up() {
        let s = Snake::new(10, 10, Direction::Up);
        // Segments extend downward
        for i in 0..INITIAL_SNAKE_LENGTH {
            assert_eq!(s.segments[i], GridPos::new(10, 10 - i as i32));
        }
    }

    #[test]
    fn snake_new_defaults() {
        let s = Snake::new(30, 30, Direction::Right);
        assert!(s.alive);
        assert_eq!(s.grow_pending, 0);
        assert_eq!(s.score, 0);
        assert_eq!(s.kills, 0);
        assert_eq!(s.direction, Direction::Right);
        assert_eq!(s.next_direction, Direction::Right);
    }

    #[test]
    fn snake_step_moves_forward() {
        let mut s = Snake::new(30, 30, Direction::Right);
        let new_head = s.step();
        assert_eq!(new_head, GridPos::new(31, 30));
        assert_eq!(s.head(), GridPos::new(31, 30));
    }

    #[test]
    fn snake_step_preserves_length() {
        let mut s = Snake::new(30, 30, Direction::Right);
        let len_before = s.segments.len();
        s.step();
        assert_eq!(s.segments.len(), len_before);
    }

    #[test]
    fn snake_step_tail_follows() {
        let mut s = Snake::new(30, 30, Direction::Right);
        // Before: [30,29,28,27,26]
        s.step();
        // After: [31,30,29,28,27] — tail (26) dropped, new head (31) added
        assert_eq!(s.segments[0], GridPos::new(31, 30));
        assert_eq!(s.segments[4], GridPos::new(27, 30));
        assert!(!s.occupies(GridPos::new(26, 30)));
    }

    #[test]
    fn snake_step_with_grow_pending() {
        let mut s = Snake::new(30, 30, Direction::Right);
        s.grow_pending = 2;
        let len_before = s.segments.len();

        s.step();
        assert_eq!(s.segments.len(), len_before + 1);
        assert_eq!(s.grow_pending, 1);

        s.step();
        assert_eq!(s.segments.len(), len_before + 2);
        assert_eq!(s.grow_pending, 0);

        // No more growth
        s.step();
        assert_eq!(s.segments.len(), len_before + 2);
    }

    #[test]
    fn snake_step_updates_direction_from_next() {
        let mut s = Snake::new(30, 30, Direction::Right);
        s.set_direction(Direction::Up);
        assert_eq!(s.direction, Direction::Right); // not yet
        assert_eq!(s.next_direction, Direction::Up);

        s.step();
        assert_eq!(s.direction, Direction::Up);
        assert_eq!(s.head(), GridPos::new(30, 31));
    }

    #[test]
    fn snake_set_direction_accepts_perpendicular() {
        let mut s = Snake::new(30, 30, Direction::Right);
        s.set_direction(Direction::Up);
        assert_eq!(s.next_direction, Direction::Up);

        s.set_direction(Direction::Down);
        assert_eq!(s.next_direction, Direction::Down);
    }

    #[test]
    fn snake_set_direction_rejects_180() {
        let mut s = Snake::new(30, 30, Direction::Right);
        s.set_direction(Direction::Left); // opposite
        assert_eq!(s.next_direction, Direction::Right); // unchanged
    }

    #[test]
    fn snake_set_direction_rejects_180_all_directions() {
        for dir in Direction::ALL {
            let mut s = Snake::new(30, 30, dir);
            s.set_direction(dir.opposite());
            assert_eq!(s.next_direction, dir, "Should reject 180° for {:?}", dir);
        }
    }

    #[test]
    fn snake_set_direction_accepts_same() {
        let mut s = Snake::new(30, 30, Direction::Right);
        s.set_direction(Direction::Right);
        assert_eq!(s.next_direction, Direction::Right);
    }

    #[test]
    fn snake_self_collision_none_initially() {
        let s = Snake::new(30, 30, Direction::Right);
        assert!(!s.self_collision());
    }

    #[test]
    fn snake_self_collision_circle_back() {
        let mut s = Snake::new(30, 30, Direction::Right);
        // Grow the snake long enough to circle
        s.grow_pending = 10;
        // Move in a square: right, up, left, down
        s.step(); // 31,30
        s.set_direction(Direction::Up);
        s.step(); // 31,31
        s.set_direction(Direction::Left);
        s.step(); // 30,31
        s.set_direction(Direction::Down);
        s.step(); // 30,30 — head is on old body position
        assert!(s.self_collision());
    }

    #[test]
    fn snake_body_collision_detects_body() {
        let s = Snake::new(30, 30, Direction::Right);
        // Body segments (not head)
        assert!(s.body_collision(GridPos::new(29, 30)));
        assert!(s.body_collision(GridPos::new(28, 30)));
    }

    #[test]
    fn snake_body_collision_ignores_head() {
        let s = Snake::new(30, 30, Direction::Right);
        assert!(!s.body_collision(GridPos::new(30, 30))); // head
    }

    #[test]
    fn snake_body_collision_misses_empty() {
        let s = Snake::new(30, 30, Direction::Right);
        assert!(!s.body_collision(GridPos::new(50, 50)));
    }

    #[test]
    fn snake_occupies_head() {
        let s = Snake::new(30, 30, Direction::Right);
        assert!(s.occupies(GridPos::new(30, 30)));
    }

    #[test]
    fn snake_occupies_body() {
        let s = Snake::new(30, 30, Direction::Right);
        assert!(s.occupies(GridPos::new(28, 30)));
    }

    #[test]
    fn snake_occupies_empty() {
        let s = Snake::new(30, 30, Direction::Right);
        assert!(!s.occupies(GridPos::new(50, 50)));
    }

    #[test]
    fn snake_head_returns_first_segment() {
        let s = Snake::new(15, 20, Direction::Up);
        assert_eq!(s.head(), s.segments[0]);
        assert_eq!(s.head(), GridPos::new(15, 20));
    }

    // ── Edge: snake at grid boundary ──────────────────────────────────

    #[test]
    fn snake_step_out_of_bounds() {
        // Snake at right edge, facing right — step goes out of bounds
        let mut s = Snake::new(GRID_WIDTH - 2, 30, Direction::Right);
        let new_head = s.step();
        assert_eq!(new_head, GridPos::new(GRID_WIDTH - 1, 30));
        assert!(!new_head.in_bounds());
    }

    // ── Edge: two snakes head-to-head ─────────────────────────────────

    #[test]
    fn two_snakes_head_to_head_collision() {
        let mut s1 = Snake::new(30, 30, Direction::Right);
        let mut s2 = Snake::new(32, 30, Direction::Left);
        s1.step(); // head at 31,30
        s2.step(); // head at 31,30
        // Both heads at the same position
        assert_eq!(s1.head(), s2.head());
        // Each head collides with the other's body
        assert!(s1.occupies(s2.head()));
        assert!(s2.occupies(s1.head()));
    }

    // ── ArenaBounds ───────────────────────────────────────────────────

    #[test]
    fn arena_default() {
        let a = ArenaBounds::default();
        assert_eq!(a.min_x, 1);
        assert_eq!(a.min_y, 1);
        assert_eq!(a.max_x, GRID_WIDTH - 1);
        assert_eq!(a.max_y, GRID_HEIGHT - 1);
    }

    #[test]
    fn arena_contains_inside() {
        let a = ArenaBounds::default();
        assert!(a.contains(GridPos::new(30, 30)));
        assert!(a.contains(GridPos::new(1, 1))); // inclusive min
    }

    #[test]
    fn arena_contains_boundary_exclusive_max() {
        let a = ArenaBounds::default();
        // Max is exclusive
        assert!(!a.contains(GridPos::new(GRID_WIDTH - 1, 30)));
        assert!(!a.contains(GridPos::new(30, GRID_HEIGHT - 1)));
    }

    #[test]
    fn arena_contains_outside() {
        let a = ArenaBounds::default();
        assert!(!a.contains(GridPos::new(0, 30)));
        assert!(!a.contains(GridPos::new(30, 0)));
        assert!(!a.contains(GridPos::new(-1, -1)));
        assert!(!a.contains(GridPos::new(GRID_WIDTH, GRID_HEIGHT)));
    }

    #[test]
    fn arena_can_shrink_initially() {
        let a = ArenaBounds::default();
        // Default arena is 58x58, well above 6x6
        assert!(a.can_shrink());
    }

    #[test]
    fn arena_can_shrink_at_minimum() {
        let a = ArenaBounds {
            min_x: 27,
            min_y: 27,
            max_x: 33,
            max_y: 33,
        };
        // 33 - 27 = 6, exactly at minimum
        assert!(!a.can_shrink());
    }

    #[test]
    fn arena_can_shrink_just_above_minimum() {
        let a = ArenaBounds {
            min_x: 26,
            min_y: 26,
            max_x: 33,
            max_y: 33,
        };
        // 33 - 26 = 7, above minimum
        assert!(a.can_shrink());
    }

    #[test]
    fn arena_shrink_decreases_bounds() {
        let mut a = ArenaBounds::default();
        let orig = a;
        a.shrink();
        assert_eq!(a.min_x, orig.min_x + 1);
        assert_eq!(a.min_y, orig.min_y + 1);
        assert_eq!(a.max_x, orig.max_x - 1);
        assert_eq!(a.max_y, orig.max_y - 1);
    }

    #[test]
    fn arena_shrink_stops_at_minimum() {
        let mut a = ArenaBounds {
            min_x: 27,
            min_y: 27,
            max_x: 33,
            max_y: 33,
        };
        let before = a;
        a.shrink(); // should be a no-op
        assert_eq!(a.min_x, before.min_x);
        assert_eq!(a.max_x, before.max_x);
    }

    #[test]
    fn arena_shrink_repeated_to_minimum() {
        let mut a = ArenaBounds::default();
        // Shrink repeatedly until minimum
        for _ in 0..100 {
            a.shrink();
        }
        assert!(!a.can_shrink());
        assert_eq!(a.max_x - a.min_x, 6);
        assert_eq!(a.max_y - a.min_y, 6);
    }

    #[test]
    fn arena_wall_distance_center() {
        let a = ArenaBounds::default();
        let d = a.wall_distance(GridPos::new(30, 30));
        // min of (30-1, 58-30, 30-1, 58-30) = min(29, 28, 29, 28) = 28
        assert_eq!(d, 28);
    }

    #[test]
    fn arena_wall_distance_corner() {
        let a = ArenaBounds::default();
        // min_x=1, min_y=1 corner
        let d = a.wall_distance(GridPos::new(1, 1));
        // min of (1-1, 58-1, 1-1, 58-1) = min(0, 57, 0, 57) = 0
        assert_eq!(d, 0);
    }

    #[test]
    fn arena_wall_distance_edge() {
        let a = ArenaBounds::default();
        // On min_x edge, somewhere in the middle vertically
        let d = a.wall_distance(GridPos::new(1, 30));
        assert_eq!(d, 0);
    }

    #[test]
    fn arena_wall_distance_one_in() {
        let a = ArenaBounds::default();
        let d = a.wall_distance(GridPos::new(2, 30));
        assert_eq!(d, 1);
    }

    // ── Edge: snake on new boundary after shrink ──────────────────────

    #[test]
    fn arena_shrink_snake_on_new_boundary() {
        let mut a = ArenaBounds::default();
        // Snake at min edge
        let snake_pos = GridPos::new(a.min_x, 30);
        assert!(a.contains(snake_pos));
        a.shrink();
        // After shrink, the old min_x is now outside
        assert!(!a.contains(snake_pos));
    }

    // ── SnakeColor ────────────────────────────────────────────────────

    #[test]
    fn snake_color_palette_returns_valid_colors() {
        for i in 0..8 {
            let c = SnakeColor::palette(i);
            // Just verify they don't panic and produce distinct head/body
            match (c.head, c.body) {
                (Color::Srgba(h), Color::Srgba(b)) => {
                    // Body should be darker (70% of head)
                    assert!((b.red - h.red * 0.7).abs() < 0.01);
                    assert!((b.green - h.green * 0.7).abs() < 0.01);
                    assert!((b.blue - h.blue * 0.7).abs() < 0.01);
                }
                _ => panic!("Expected Srgba colors"),
            }
        }
    }

    #[test]
    fn snake_color_palette_wraps_at_8() {
        for i in 0..8 {
            let a = SnakeColor::palette(i);
            let b = SnakeColor::palette(i + 8);
            match (a.head, b.head) {
                (Color::Srgba(ah), Color::Srgba(bh)) => {
                    assert!((ah.red - bh.red).abs() < f32::EPSILON);
                    assert!((ah.green - bh.green).abs() < f32::EPSILON);
                    assert!((ah.blue - bh.blue).abs() < f32::EPSILON);
                }
                _ => panic!("Expected Srgba colors"),
            }
        }
    }

    #[test]
    fn snake_color_new_body_is_70_percent() {
        let c = SnakeColor::new(1.0, 0.5, 0.8);
        match (c.head, c.body) {
            (Color::Srgba(h), Color::Srgba(b)) => {
                assert!((h.red - 1.0).abs() < f32::EPSILON);
                assert!((h.green - 0.5).abs() < f32::EPSILON);
                assert!((h.blue - 0.8).abs() < f32::EPSILON);
                assert!((b.red - 0.7).abs() < f32::EPSILON);
                assert!((b.green - 0.35).abs() < f32::EPSILON);
                assert!((b.blue - 0.56).abs() < f32::EPSILON);
            }
            _ => panic!("Expected Srgba colors"),
        }
    }

    // ── Food ──────────────────────────────────────────────────────────

    #[test]
    fn food_new() {
        let f = Food::new(10, 20);
        assert_eq!(f.pos, GridPos::new(10, 20));
    }

    // ── GridPos (0,0) is out of bounds (wall) ─────────────────────────

    #[test]
    fn gridpos_origin_is_wall() {
        assert!(!GridPos::new(0, 0).in_bounds());
    }
}
