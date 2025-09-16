/// Zone related constats
pub const MAP_WIDTH: i32 = 56;
pub const MAP_HEIGHT: i32 = 32;
pub const MAP_WIDTH_F32: f32 = MAP_WIDTH as f32;
pub const MAP_HEIGHT_F32: f32 = MAP_HEIGHT as f32;
pub const TILE_SIZE: i32 = 24;
pub const TILE_SIZE_F32: f32 = TILE_SIZE as f32;
pub const BRAZIER_RADIUS: i32 = 10;
pub const MAX_BRAZIER_IN_ZONE: i32 = 3;
pub const MAX_RIVERS_IN_ZONE: i32 = 6;

/// Hud related constants
pub const HUD_WIDTH: i32 = MAP_WIDTH * TILE_SIZE;
pub const HUD_HEIGHT: i32 = 192 + UI_BORDER;
pub const HUD_BORDER: i32 = 4;

pub const HEADER_HEIGHT: i32 = 24;
pub const HEADER_LEFT_SPAN: i32 = 64;

pub const MAX_MESSAGES_IN_LOG: usize = 4;

/// Inventory related constants
pub const INVENTORY_X: i32 = (WINDOW_WIDTH/2) - INVENTORY_SIZE/2;
pub const INVENTORY_Y: i32 = (WINDOW_HEIGHT/3) - INVENTORY_SIZE/2;
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
pub const MAX_ITEMS_IN_BACKPACK: usize = 10;

/// Dialog related constants
pub const DIALOG_X: i32 = (WINDOW_WIDTH/2) - DIALOG_SIZE/2; 
pub const DIALOG_Y: i32 = (WINDOW_HEIGHT/3) - DIALOG_SIZE/2; 
pub const DIALOG_SIZE: i32 = 512;
pub const DIALOG_FOOTER_WIDTH: i32 = 186;
pub const DIALOG_LEFT_SPAN: i32 = DIALOG_FOOTER_WIDTH/4;
pub const DIALOG_TOP_SPAN: i32 = DIALOG_SIZE/6;

/// UI related constants
pub const UI_BORDER: i32 = 8;
pub const UI_BORDER_F32: f32 = UI_BORDER as f32;
pub const WINDOW_WIDTH: i32 = (UI_BORDER * 2) + (MAP_WIDTH * TILE_SIZE);
pub const WINDOW_HEIGHT: i32 = (UI_BORDER * 2) + (MAP_HEIGHT * TILE_SIZE) + HUD_HEIGHT;
pub const FONT_SIZE: f32 = 32.0;
pub const LETTER_SIZE: f32 = 15.0;

/// Timing related constants
pub const SECONDS_TO_WAIT: f32 = 0.1;
pub const SLOW: i32 = 1;
pub const NORMAL: i32 = 2;
pub const FAST: i32 = 3;
pub const MAX_ACTION_SPEED: i32 = 4;

/// Spawning related constants
pub const MAX_MONSTERS_IN_ZONE: i32 = 5;
pub const MAX_ITEMS_IN_ZONE: i32 = 10;
pub const MAX_SPAWN_TENTANTIVES: i32 = 10;

/// Player related constants
pub const BASE_VIEW_RADIUS: i32 = 6;
pub const MAX_STAMINA_HEAL_TICK_COUNTER: i32 = 4;
pub const MAX_HUNGER_TICK_COUNTER: i32 = 151;
pub const MAX_THIRST_TICK_COUNTER: i32 = 151;
pub const PLAYER_SMELL_RADIUS: f32 = 16.0;
pub const PLAYER_LISTEN_RADIUS: f32 = 20.0;

/// Drunken Walk related constants
pub const DRUNKEN_WALK_LIFE_MAX: i32 = 50;
pub const DRUNKEN_WALK_MAX_ITERATIONS: i32 = 50;

/// Item related constants
pub const STARTING_ROT_COUNTER: i32 = 100;
pub const LANTERN_RADIUS: i32 = 6;
pub const STARTING_FUEL: i32 = 400;
pub const STARTING_WET_COUNTER: i32 = 50;

pub const MUSHROOM_EXCELLENT: i32 = 0;
pub const MUSHROOM_POISONOUS: i32 = 1;
pub const MUSHROOM_MEDIOCRE: i32 = 2;
pub const MUSHROOM_DEADLY: i32 = 3;
pub const MUSHROOM_LUMINESCENT: i32 = 4;
pub const MUSHROOM_LIGHT_RADIUS: i32 = 2;

pub const MUSHROOM_SPAWN_MAP: [i32; 10] = [
    MUSHROOM_EXCELLENT,
    MUSHROOM_MEDIOCRE,
    MUSHROOM_MEDIOCRE,
    MUSHROOM_MEDIOCRE,
    MUSHROOM_MEDIOCRE,
    MUSHROOM_POISONOUS,
    MUSHROOM_POISONOUS,
    MUSHROOM_DEADLY,
    MUSHROOM_LUMINESCENT,
    MUSHROOM_LUMINESCENT,
];

/// Monsters related constats
pub const BASE_MONSTER_VIEW_RADIUS: i32 = 8;
pub const MAX_HIDDEN_TURNS: i32 = 9;

/// Saving Throw related constants
pub const AUTOFAIL_SAVING_THROW: i32 = 999;

pub const LISTEN_COOLDOWN_START:i32 = 19;

#[derive(PartialEq, Debug)]
pub enum UIAction {
    Eat,
    Drop,
    Invoke,
    Quaff,
    RefillWhat,
    RefillWith,
    Equip,
}