/// Map related constats
pub const MAP_WIDTH: i32 = 40;
pub const MAP_HEIGHT: i32 = 25;
pub const TILE_SIZE: i32 = 32;

/// Hud related constants
pub const HUD_WIDTH: i32 = MAP_WIDTH * TILE_SIZE;
pub const HUD_HEIGHT: i32 = 192 + UI_BORDER;
pub const HUD_BORDER: i32 = 4;

pub const FONT_SIZE: f32 = 32.0;

/// UI related constants
pub const UI_BORDER: i32 = 8;
pub const WINDOW_WIDTH: i32 = (UI_BORDER * 2) + (MAP_WIDTH * TILE_SIZE);
pub const WINDOW_HEIGHT: i32 = (UI_BORDER * 2) + (MAP_HEIGHT * TILE_SIZE) + HUD_HEIGHT;
pub const HEADER_WIDTH: i32 = 512;
pub const HEADER_HEIGHT: i32 = 24;
pub const HEADER_LEFT_SPAN: i32 = 64;
pub const MAX_MESSAGES_IN_LOG: usize = 4;

/// Timing related constants
pub const SECONDS_TO_WAIT: f32 = 0.1;
