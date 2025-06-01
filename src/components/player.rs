use std::cmp::{max, min};

use hecs::{Entity, World};
use macroquad::input::{KeyCode, get_key_pressed};

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    engine::state::RunState,
    map::{Map, get_index_from_xy},
    utils::random_util::RandomUtils,
};

use super::{
    combat::{CombatStats, Damageable},
    common::GameLog,
    items::WantsItem,
};
use super::{
    common::{Named, Position, Viewshed},
    items::Item,
};

/// Player constants
pub const VIEW_RADIUS: i32 = 8;

/// Player struct
pub struct Player {}

impl Player {
    ///
    /// Try to move player
    ///
    fn try_move_player(delta_x: i32, delta_y: i32, ecs_world: &World) {
        let mut players =
            ecs_world.query::<(&Player, &mut Position, &mut Viewshed, &CombatStats)>();

        let mut map_query = ecs_world.query::<&Map>();
        let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

        let mut game_log_query = ecs_world.query::<&mut GameLog>();
        let (_e, game_log) = game_log_query
            .iter()
            .last()
            .expect("Game log is not in hecs::World");

        for (_e, (_p, position, viewshed, player_stats)) in &mut players {
            let destination_index = get_index_from_xy(position.x + delta_x, position.y + delta_y);

            //Search for potential targets
            for &potential_target in map.tile_content[destination_index].iter() {
                // Has potential target the CombatStats Component?
                let mut target = ecs_world
                    .query_one::<(&mut Damageable, &CombatStats, &Named)>(potential_target)
                    .unwrap();

                // Possibily attack, impeding movement
                match target.get() {
                    None => {}
                    Some((target, target_stats, target_name)) => {
                        // Attack it
                        let damage = max(
                            0,
                            RandomUtils::dice(1, player_stats.unarmed_attack_dice)
                                - target_stats.base_armor,
                        );
                        game_log.entries.push(format!(
                            "You punch {} for {} damage",
                            target_name.name, damage
                        ));
                        target.damage_received += damage;
                        return;
                    }
                }
            }

            // Move if destination is not blocked
            if !map.blocked_tiles[destination_index] {
                position.x = min(MAP_WIDTH - 1, max(0, position.x + delta_x));
                position.y = min(MAP_HEIGHT - 1, max(0, position.y + delta_y));
                viewshed.must_recalculate = true;
            }
        }
    }

    ///
    /// Handle player input
    ///
    pub fn player_input(ecs_world: &mut World) -> RunState {
        // Player movement
        match get_key_pressed() {
            None => return RunState::WaitingPlayerInput, // Nothing happened
            Some(key) => match key {
                KeyCode::Kp4 | KeyCode::Left => Self::try_move_player(-1, 0, ecs_world),
                KeyCode::Kp6 | KeyCode::Right => Self::try_move_player(1, 0, ecs_world),
                KeyCode::Kp8 | KeyCode::Up => Self::try_move_player(0, -1, ecs_world),
                KeyCode::Kp2 | KeyCode::Down => Self::try_move_player(0, 1, ecs_world),

                // Diagonals
                KeyCode::Kp9 => Self::try_move_player(1, -1, ecs_world),
                KeyCode::Kp7 => Self::try_move_player(-1, -1, ecs_world),
                KeyCode::Kp3 => Self::try_move_player(1, 1, ecs_world),
                KeyCode::Kp1 => Self::try_move_player(-1, 1, ecs_world),

                //Pick up
                KeyCode::P => Self::pick_up(ecs_world),

                //Show Inventory
                KeyCode::I => return RunState::ShowInventory,

                _ => return RunState::WaitingPlayerInput,
            },
        }

        RunState::PlayerTurn
    }

    fn pick_up(ecs_world: &mut World) {
        let (player_position, player_entity);
        let mut target_item: Option<Entity> = None;

        // Scope for keeping borrow checker quiet
        {
            let mut player_query = ecs_world.query::<(&Player, &Position)>();
            let (player, (_p, position)) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");
            player_position = position;
            player_entity = player;

            let mut items = ecs_world.query::<(&Item, &Position)>();
            // Get item
            for (item_entity, (_item, item_position)) in &mut items {
                if player_position.x == item_position.x && player_position.y == item_position.y {
                    target_item = Some(item_entity)
                }
            }
        }

        let mut picked_something = false;
        match target_item {
            None => {}
            Some(item) => {
                picked_something = true;
                let _ = ecs_world.insert_one(
                    player_entity,
                    WantsItem {
                        collected_by: player_entity,
                        item: item,
                    },
                );
            }
        }

        if !picked_something {
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            game_log
                .entries
                .push(String::from("There is nothing here to pick up"));
        }
    }
}
