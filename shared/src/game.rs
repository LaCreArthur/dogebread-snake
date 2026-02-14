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

    pub fn in_bounds(self) -> bool {
        self.x >= 0 && self.x < GRID_WIDTH && self.y >= 0 && self.y < GRID_HEIGHT
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

    /// Predefined distinct snake colors
    pub fn palette(index: u32) -> Self {
        match index % 8 {
            0 => Self::new(0.95, 0.75, 0.1),  // Gold (player default)
            1 => Self::new(0.2, 0.8, 0.4),    // Green
            2 => Self::new(0.9, 0.3, 0.3),    // Red
            3 => Self::new(0.3, 0.5, 0.95),   // Blue
            4 => Self::new(0.9, 0.5, 0.9),    // Pink
            5 => Self::new(0.1, 0.9, 0.9),    // Cyan
            6 => Self::new(0.95, 0.55, 0.1),  // Orange
            7 => Self::new(0.7, 0.7, 0.95),   // Lavender
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
    Playing,
    GameOver,
}

/// Track alive snake count and rankings
#[derive(Resource, Default)]
pub struct MatchState {
    pub alive_count: u32,
    pub total_snakes: u32,
}

/// Timer for dead snake body cleanup
#[derive(Component)]
pub struct DeathTimer {
    pub timer: bevy::time::Timer,
}
