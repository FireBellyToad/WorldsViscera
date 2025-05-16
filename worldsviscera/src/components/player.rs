use std::cmp::{max, min};

use bracket_lib::prelude::{BTerm, VirtualKeyCode};
use specs::prelude::*;
use specs_derive::Component;

use crate::{
    game_state::{RunState, State},
    map::Map,
};

use super::common::{Position, Viewshed};

/// Player constants
pub const VIEW_RADIUS: i32 = 8;

/// Player module
#[derive(Component, Debug)]
pub struct Player {}

// Player functions

///
/// Try to move player
///
fn try_move_player(delta_x: i32, delta_y: i32, ecs_world: &mut World) {
    //Get all entities with Position an Player components
    let mut positions = ecs_world.write_storage::<Position>();
    let mut players = ecs_world.write_storage::<Player>();
    let mut viewsheds = ecs_world.write_storage::<Viewshed>();
    let map = ecs_world.fetch::<Map>();

    // For each one that have both of them (only one, the Player), change position if space is free
    for (_player, position, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        if map.is_tile_passable(position.x + delta_x, position.y + delta_y) {
            position.x = min(map.width - 1, max(0, position.x + delta_x));
            position.y = min(map.height - 1, max(0, position.y + delta_y));

            viewshed.must_recalculate = true;
        }
    }
}

///
/// Handle player input
///
pub fn player_input(game_state: &mut State, context: &mut BTerm) -> RunState{
    //Move Player
    match context.key {
        None => {return RunState::Paused } // Keep the game paused if none is pressed
        Some(key) => match key {
            //Support Numpad and vi commands (holy shit)
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut game_state.ecs_world)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut game_state.ecs_world)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut game_state.ecs_world)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut game_state.ecs_world)
            }
            _ => { return RunState::Paused } // Keep the game paused for all other keys
        },
    }

    RunState::Running
}
