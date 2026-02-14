/// Grid dimensions (cells)
pub const GRID_WIDTH: i32 = 40;
pub const GRID_HEIGHT: i32 = 40;

/// Size of each cell in pixels
pub const CELL_SIZE: f32 = 16.0;

/// Game tick rate (movements per second)
pub const TICK_RATE: f64 = 8.0;

/// How many ticks between each movement
pub const TICK_INTERVAL: f64 = 1.0 / TICK_RATE;

/// Initial snake length
pub const INITIAL_SNAKE_LENGTH: usize = 5;

/// Respawn delay in seconds
pub const RESPAWN_DELAY: f32 = 3.0;

/// Points per food eaten
pub const POINTS_PER_FOOD: i32 = 1;

/// Points for killing another snake
pub const POINTS_PER_KILL: i32 = 3;

/// Score to win
pub const WINNING_SCORE: i32 = 20;

/// Game duration in seconds
pub const GAME_DURATION: f32 = 180.0;
