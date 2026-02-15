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
