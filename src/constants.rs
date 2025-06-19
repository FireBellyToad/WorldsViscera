/// Zone related constats
pub const MAP_WIDTH: i32 = 56;
pub const MAP_HEIGHT: i32 = 34;
pub const TILE_SIZE: i32 = 24;
pub const TILE_SIZE_F32: f32 = TILE_SIZE as f32;

/// Hud related constants
pub const HUD_WIDTH: i32 = MAP_WIDTH * TILE_SIZE;
pub const HUD_HEIGHT: i32 = 192 + UI_BORDER;
pub const HUD_BORDER: i32 = 4;

pub const HEADER_HEIGHT: i32 = 24;
pub const HEADER_LEFT_SPAN: i32 = 64;

pub const MAX_MESSAGES_IN_LOG: usize = 4;

/// Inventory related constants
pub const INVENTORY_X: i32 = 64;
pub const INVENTORY_Y: i32 = 128;
pub const INVENTORY_SIZE: i32 = 512;
pub const INVENTORY_FOOTER_WIDTH: i32 = 186;
pub const INVENTORY_LEFT_SPAN: i32 = 20;
pub const INVENTORY_TOP_SPAN: i32 = 48;
pub const OPTION_TO_CHAR_MAP: [char; 52] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
pub const ITEM_INVENTORY_LEFT_SPAN: i32 = 12;
pub const ITEM_INVENTORY_TOP_SPAN: i32 = 10;

/// UI related constants
pub const UI_BORDER: i32 = 8;
pub const UI_BORDER_F32: f32 = UI_BORDER as f32;
pub const WINDOW_WIDTH: i32 = (UI_BORDER * 2) + (MAP_WIDTH * TILE_SIZE);
pub const WINDOW_HEIGHT: i32 = (UI_BORDER * 2) + (MAP_HEIGHT * TILE_SIZE) + HUD_HEIGHT;
pub const FONT_SIZE: f32 = 32.0;
pub const LETTER_SIZE: f32 = 15.0;

/// Timing related constants
pub const SECONDS_TO_WAIT: f32 = 0.1;
pub const MAX_ACTION_SPEED: i32 = 4;

/// Spawning related constants
pub const MAX_MONSTERS_ON_ROOM_START: i32 = 5;
pub const MAX_ITEMS_ON_ROOM_START: i32 = 2;
pub const MAX_SPAWN_TENTANTIVES: i32 = 10;

/// Player related constants
pub const BASE_VIEW_RADIUS: i32 = 6;
pub const MAX_STAMINA_HEAL_TICK_COUNTER: i32 = 4;
pub const MAX_HUNGER_TICK_COUNTER: i32 = 151;
pub const MAX_THIRST_TICK_COUNTER: i32 = 151;

/// Drunken Walk related constants
pub const DRUNKEN_WALK_LIFE_MAX: i32 = 50;
pub const DRUNKEN_WALK_MAX_ITERATIONS: i32 = 50;

/// Item related constants
pub const STARTING_ROT_COUNTER: i32 = 51;
pub const LANTERN_RADIUS: i32 = 6;

/// Monsters related constats
pub const BASE_MONSTER_VIEW_RADIUS: i32 = 8;