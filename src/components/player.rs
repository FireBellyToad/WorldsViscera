use std::cmp::{max, min};

use hecs::{Entity, World};
use macroquad::input::{
    KeyCode, MouseButton, clear_input_queue, get_char_pressed, get_key_pressed, is_key_down,
    is_mouse_button_down, mouse_position,
};

use crate::{
    components::{
        combat::WantsToZap,
        common::{MyTurn, WaitingToAct},
        health::CanAutomaticallyHeal,
    },
    constants::*,
    engine::state::RunState,
    inventory::InventoryAction,
    maps::zone::{TileType, Zone},
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
    fn try_move(delta_x: i32, delta_y: i32, ecs_world: &mut World) -> RunState {
        let mut return_state = RunState::WaitingPlayerInput;
        let mut attacker_target: Option<(Entity, Entity)> = None;

        // Scope for keeping borrow checker quiet
        {
            let mut players = ecs_world.query::<(&Player, &mut Position, &mut Viewshed)>();

            let mut zone_query = ecs_world.query::<&Zone>();
            let (_e, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            for (player_entity, (_p, position, viewshed)) in &mut players {
                let destination_index =
                    Zone::get_index_from_xy(position.x + delta_x, position.y + delta_y);

                //Search for potential targets (must have CombatStats component)
                for &potential_target in zone.tile_content[destination_index].iter() {
                    let has_combat_stats = ecs_world
                        .satisfies::<&CombatStats>(potential_target)
                        .unwrap();

                    if has_combat_stats {
                        attacker_target = Some((player_entity, potential_target));
                    }
                }

                // Move if not attacking or destination is not blocked
                if attacker_target.is_none() {
                    if !zone.blocked_tiles[destination_index] {
                        position.x = min(MAP_WIDTH - 1, max(0, position.x + delta_x));
                        position.y = min(MAP_HEIGHT - 1, max(0, position.y + delta_y));
                        viewshed.must_recalculate = true;
                        return_state = RunState::DoTick;
                    }
                }
            }
        }

        // Attack if needed
        if attacker_target.is_some() {
            let (attacker, target) = attacker_target.unwrap();
            let _ = ecs_world.insert_one(attacker, WantsToMelee { target });

            return_state = RunState::DoTick;
        }

        return_state
    }

    ///
    /// Handle player input
    ///
    pub fn checks_keyboard_input(ecs_world: &mut World) -> RunState {
        let mut run_state = RunState::WaitingPlayerInput;
        let mut check_chars_pressed = false;
        let mut is_actively_waiting = false;
        // Player movement
        match get_key_pressed() {
            None => run_state = RunState::WaitingPlayerInput, // Nothing happened
            Some(key) => match key {
                KeyCode::Kp4 | KeyCode::Left => run_state = Player::try_move(-1, 0, ecs_world),
                KeyCode::Kp6 | KeyCode::Right => run_state = Player::try_move(1, 0, ecs_world),
                KeyCode::Kp8 | KeyCode::Up => run_state = Player::try_move(0, -1, ecs_world),
                KeyCode::Kp2 | KeyCode::Down => run_state = Player::try_move(0, 1, ecs_world),

                // Diagonals
                KeyCode::Kp9 => run_state = Player::try_move(1, -1, ecs_world),
                KeyCode::Kp7 => run_state = Player::try_move(-1, -1, ecs_world),
                KeyCode::Kp3 => run_state = Player::try_move(1, 1, ecs_world),
                KeyCode::Kp1 => run_state = Player::try_move(-1, 1, ecs_world),

                // Skip turn doing nothing, so you can heal
                KeyCode::Space => {
                    run_state = RunState::DoTick;
                    is_actively_waiting = true;
                }

                // Something was pressed but is not in this match?
                // Check for characters pressed
                _ => check_chars_pressed = true,
            },
        }

        // Player commands. Handed with characters to manage different keyboards layout
        // Do it only if no keys were pressed or else Arrow keys and space will not work properly
        if check_chars_pressed {
            match get_char_pressed() {
                None => run_state = RunState::WaitingPlayerInput, // Nothing happened
                Some(char) => {
                    match char {
                        // Skip turn doing nothing, so you can heal
                        '.' => {
                            run_state = RunState::DoTick;
                            is_actively_waiting = true;
                        }

                        //Pick up
                        'p' => {
                            Player::pick_up(ecs_world);
                            run_state = RunState::DoTick;
                        }

                        //Eat item
                        'e' => {
                            clear_input_queue();
                            run_state = RunState::ShowInventory(InventoryAction::Eat);
                        }

                        //DEBUG ONLY KILL
                        'k' => {
                            run_state = RunState::GameOver;
                        }

                        '<' | '>' => {
                            run_state = Player::try_next_level(ecs_world, char);
                        }

                        //Drop item
                        'd' => {
                            clear_input_queue();
                            run_state = RunState::ShowInventory(InventoryAction::Drop);
                        }

                        //Invoke item
                        'i' => {
                            clear_input_queue();
                            run_state = RunState::ShowInventory(InventoryAction::Invoke);
                        }

                        //Quaff item
                        'q' => {
                            clear_input_queue();
                            run_state = RunState::ShowInventory(InventoryAction::Quaff);
                        }

                        _ => {}
                    }
                }
            }
        }

        // Wait if any real action is taken
        if run_state != RunState::WaitingPlayerInput {
            // Reset heal counter if the player did not wait through space or . key
            if !is_actively_waiting {
                Player::reset_heal_counter(ecs_world);
            }
            Player::wait_after_action(ecs_world);
        }

        run_state
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

            let mut is_valid_tile = false;
            // Scope for keeping borrow checker quiet
            {
                let mut zone_query = ecs_world.query::<&Zone>();
                let (_e, zone) = zone_query
                    .iter()
                    .last()
                    .expect("Zone is not in hecs::World");
                // Make sure that we are targeting a valid tile
                let index = Zone::get_index_from_xy(rounded_x, rounded_y);
                if index < zone.visible_tiles.len() {
                    is_valid_tile = zone.visible_tiles[index];
                }
            }

            let player_entity = Player::get_player_entity(&ecs_world);

            if is_valid_tile {
                let _ = ecs_world.insert_one(
                    player_entity,
                    WantsToZap {
                        target: (rounded_x, rounded_y),
                    },
                );
                // Reset heal counter if the player did not wait
                Player::reset_heal_counter(ecs_world);
                Player::wait_after_action(ecs_world);
                return RunState::DoTick;
            }
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
        } else {
            // Reset heal counter if the player did pick up something
            Player::reset_heal_counter(ecs_world);
            Player::wait_after_action(ecs_world);
        }
    }

    fn try_next_level(ecs_world: &mut World, char_pressed: char) -> RunState {
        let player_position;
        let standing_on_tile;
        
        //Scope to keep borrow checker quiet
        {
            let mut player_query = ecs_world.query::<(&Player, &Position)>();
            let (_e, (_p, position)) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");
            player_position = position;

            let mut zone_query = ecs_world.query::<&Zone>();
            let (_e, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");
            standing_on_tile =
                &zone.tiles[Zone::get_index_from_xy(player_position.x, player_position.y)];

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            game_log
                .entries
                .push(String::from("There is nothing here to pick up"));

            //TODO skill check
            match standing_on_tile {
                TileType::DownPassage => {
                    if char_pressed == '>' {
                        game_log.entries.push(format!("You climb down..."));
                        return RunState::GoToNextZone;
                    }
                }
                TileType::UpPassage => {
                    if char_pressed == '<' {
                        game_log.entries.push(format!("You climb up..."));
                        return RunState::GoToNextZone;
                    }
                }
                _ => {
                    if char_pressed == '>' {
                        game_log.entries.push(format!("You can't go down here"));
                    } else if char_pressed == '<' {
                        game_log.entries.push(format!("You can't go up here"));
                    }
                }
            }
        }

        return RunState::WaitingPlayerInput;
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
    /// Extract Player's entity from world and return it with copy
    pub fn can_act(ecs_world: &World) -> bool {
        let mut player_query = ecs_world.query::<(&Player, &MyTurn)>();

        player_query.iter().len() > 0
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

    /// Wait some ticks after action is taken
    pub fn wait_after_action(ecs_world: &mut World) {
        let player = Player::get_player_entity(ecs_world);
        let speed;

        // Scope for keeping borrow checker quiet
        {
            speed = ecs_world.get::<&CombatStats>(player).unwrap().speed;
        }
        // TODO account speed penalties
        let _ = ecs_world.exchange_one::<MyTurn, WaitingToAct>(
            player,
            WaitingToAct {
                tick_countdown: max(1, MAX_ACTION_SPEED - speed),
            },
        );
    }
}
