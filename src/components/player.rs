use std::cmp::{max, min};

use hecs::World;
use macroquad::input::{KeyCode, get_key_pressed};

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    map::{get_index_from_xy, Map, TileType},
};

use super::common::{Position, Viewshed};

/// Player constants
pub const VIEW_RADIUS: i32 = 8;

/// Player struct
pub struct Player {}

///
/// Try to move player
///
fn try_move_player(delta_x: i32, delta_y: i32, ecs_world: &World) {
    let mut players = ecs_world.query::<(&Player, &mut Position, &mut Viewshed)>();
    let mut map_query = ecs_world.query::<&Map>();
    let (_e, map) = map_query
        .iter()
        .last()
        .expect("Map is not in hecs::World");

    for (entity, (_players, position, viewshed)) in &mut players {
        println!("Player: {:?}",entity);
        let index = get_index_from_xy(position.x + delta_x, position.y + delta_y);
        if map.tiles[index] != TileType::Wall {
            position.x = min(MAP_WIDTH - 1, max(0, position.x + delta_x));
            position.y = min(MAP_HEIGHT - 1, max(0, position.y + delta_y));
            viewshed.must_recalculate = true;
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
            KeyCode::Kp4 | KeyCode::Left => try_move_player(-1, 0, &ecs_world),
            KeyCode::Kp6 | KeyCode::Right => try_move_player(1, 0, &ecs_world),
            KeyCode::Kp8 | KeyCode::Up => try_move_player(0, -1, &ecs_world),
            KeyCode::Kp2 | KeyCode::Down => try_move_player(0, 1, &ecs_world),
            _ => {}
        },
    }
}
