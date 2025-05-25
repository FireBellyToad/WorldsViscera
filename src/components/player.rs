use std::cmp::{max, min};

use hecs::World;
use macroquad::input::{get_key_pressed, get_keys_down, get_keys_pressed, get_last_key_pressed, KeyCode};

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    map::{Map, TileType},
};

use super::common::Position;

/// Player constants
pub const VIEW_RADIUS: i32 = 8;

/// Player struct
pub struct Player {}

///
/// Try to move player
///
fn try_move_player(delta_x: i32, delta_y: i32, ecs_world: &World) {
    let mut players = ecs_world.query::<(&Player, &mut Position)>();
    let mut maps = ecs_world.query::<&Map>();

    for (_entity, (_players, position)) in &mut players {
        for (_entity, map) in &mut maps {
            let index = map.get_index_from_xy(position.x + delta_x, position.y + delta_y);
            if map.tiles[index] != TileType::Wall {
                position.x = min(MAP_WIDTH - 1, max(0, position.x + delta_x));
                position.y = min(MAP_HEIGHT - 1, max(0, position.y + delta_y));
            }
        }
    }
}

///
/// Handle player input
///
pub fn player_input(ecs_world: &World) {
    // Player movement
    match get_key_pressed() {
        None => {} // Nothing happened
        Some(key) => match key {
            KeyCode::Left => try_move_player(-1, 0, &ecs_world),
            KeyCode::Right => try_move_player(1, 0, &ecs_world),
            KeyCode::Up => try_move_player(0, -1, &ecs_world),
            KeyCode::Down => try_move_player(0, 1, &ecs_world),
            _ => {}
        },
    }
    
}
