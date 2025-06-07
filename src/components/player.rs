use std::cmp::{max, min};

use hecs::{Entity, World};
use macroquad::input::{
    KeyCode, MouseButton, clear_input_queue, get_key_pressed, is_key_down, is_mouse_button_down,
    mouse_position,
};

use crate::{
    components::{
        combat::WantsToZap,
        health::CanAutomaticallyHeal,
        map::{Map, get_index_from_xy},
    },
    constants::*,
    engine::state::RunState,
};

use super::{
    combat::{CombatStats, WantsToMelee},
    common::GameLog,
    items::WantsItem,
};
use super::{
    common::{Position, Viewshed},
    items::Item,
};

/// Player struct
pub struct Player {}

impl Player {
    ///
    /// Try to move player
    ///
    fn try_move_player(delta_x: i32, delta_y: i32, ecs_world: &mut World) {
        let mut attacker_target: Option<(Entity, Entity)> = None;

        // Scope for keeping borrow checker quiet
        {
            let mut players = ecs_world.query::<(&Player, &mut Position, &mut Viewshed)>();

            let mut map_query = ecs_world.query::<&Map>();
            let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

            for (player_entity, (_p, position, viewshed)) in &mut players {
                let destination_index =
                    get_index_from_xy(position.x + delta_x, position.y + delta_y);

                //Search for potential targets (must have CombatStats component)
                for &potential_target in map.tile_content[destination_index].iter() {
                    let has_combat_stats = ecs_world
                        .satisfies::<&CombatStats>(potential_target)
                        .unwrap();

                    if has_combat_stats {
                        attacker_target = Some((player_entity, potential_target));
                    }
                }

                // Move if not attacking or destination is not blocked
                if attacker_target.is_none() && !map.blocked_tiles[destination_index] {
                    position.x = min(MAP_WIDTH - 1, max(0, position.x + delta_x));
                    position.y = min(MAP_HEIGHT - 1, max(0, position.y + delta_y));
                    viewshed.must_recalculate = true;
                }
            }
        }

        // Attack if needed
        if attacker_target.is_some() {
            let (attacker, target) = attacker_target.unwrap();
            let _ = ecs_world.insert_one(attacker, WantsToMelee { target });
        }
    }

    ///
    /// Handle player input
    ///
    pub fn checks_keyboard_input(ecs_world: &mut World) -> RunState {
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

                // Skip turn doing nothing, so you can heal
                KeyCode::Period | KeyCode::Space => return RunState::MonsterTurn,

                //Pick up
                KeyCode::P => {
                    Self::pick_up(ecs_world);
                    clear_input_queue();
                }

                //Eat item
                KeyCode::E => {
                    clear_input_queue();
                    return RunState::ShowEatInventory;
                }

                //Drop item
                KeyCode::D => {
                    clear_input_queue();
                    return RunState::ShowDropInventory;
                }

                //Invoke item
                KeyCode::I => {
                    clear_input_queue();
                    return RunState::ShowInvokeInventory;
                }

                _ => return RunState::WaitingPlayerInput,
            },
        }

        RunState::PlayerTurn
    }

    /// Checks mouse input
    pub fn checks_input_for_targeting(ecs_world: &mut World) -> RunState {
        // ESC for escaping targeting without using Invokable
        if is_key_down(KeyCode::Escape) {
            return RunState::WaitingPlayerInput;
        } else if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();

            let rounded_x = (((mouse_x - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;
            let rounded_y = (((mouse_y - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;

            let player_entity = Self::get_player_entity(&ecs_world);

            let _ = ecs_world.insert_one(
                player_entity,
                WantsToZap {
                    target: (rounded_x, rounded_y),
                },
            );
            return RunState::PlayerTurn;
        }

        RunState::MouseTargeting
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
                let _ = ecs_world.insert_one(player_entity, WantsItem { item: item });
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

    /// Extract Player's entity from world and return it with copy
    pub fn get_player_entity(ecs_world: &World) -> Entity {
        let mut player_query = ecs_world.query::<&Player>();
        let (player_entity, _p) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        player_entity
    }
    /// Extract Player's entity id from world and return it with copy
    pub fn get_player_id(ecs_world: &World) -> u32 {
        Player::get_player_entity(ecs_world).id()
    }

    /// Reset heal counter. Usually when the player did anything but wait
    pub fn reset_heal_counter(ecs_world: &World) {
        let mut players =
            ecs_world.query::<(&Player, &mut CombatStats, &mut CanAutomaticallyHeal)>();
        for (_e, (_p, stats, can_heal)) in &mut players {
            if stats.current_stamina < stats.max_stamina {
                can_heal.tick_counter = MAX_STAMINA_HEAL_TICK_COUNTER
            }
        }
    }
}
