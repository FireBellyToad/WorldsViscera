use std::cmp::{max, min};

use bracket_lib::prelude::{BTerm, VirtualKeyCode};
use specs::prelude::*;
use specs_derive::Component;

use crate::{components::Position, game_state::State, map::{is_tile_passable, TileType, MAP_HEIGHT, MAP_WIDTH}};

// Player module

#[derive(Component)]
pub struct Player {}

// Player functions

/**
 * Try to move player
 */
fn try_move_player(delta_x: i32, delta_y: i32, ecs_world: &mut World) {
    //Get all entities with Position an Player components
    let mut positions = ecs_world.write_storage::<Position>();
    let mut players = ecs_world.write_storage::<Player>();
    let map = ecs_world.fetch::<Vec<TileType>>();

    // For each one that have both of them (only one, the Player), change position if space is free
    for (_player, pos) in (&mut players, &mut positions).join() {

        if is_tile_passable(&map, pos.x + delta_x, pos.y + delta_y) {
            pos.x = min(MAP_WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_HEIGHT - 1, max(0, pos.y + delta_y));
        }
    }
}

/**
 * Get input for player
 */
pub fn player_input(game_state: &mut State, context: &mut BTerm) {
    //Move Player
    match context.key {
        None => {} // Do nothing if none is pressed
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut game_state.ecs_world),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut game_state.ecs_world),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut game_state.ecs_world),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut game_state.ecs_world),
            _ => {} // Do nothing for all other keys
        },
    }
}
