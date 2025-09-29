use std::cmp::max;

use hecs::{Entity, World};
use macroquad::input::{
    KeyCode, MouseButton, clear_input_queue, get_char_pressed, get_key_pressed, is_key_down,
    is_mouse_button_down, mouse_position,
};

use crate::{
    components::{
        actions::{WantsItem, WantsToDrink, WantsToSmell},
        combat::{CombatStats, WantsToMelee, WantsToZap},
        common::{GameLog, MyTurn, Position, Viewshed, WaitingToAct},
        health::CanAutomaticallyHeal,
        items::{Edible, Item, Quaffable},
    },
    constants::{
        MAP_HEIGHT, MAP_WIDTH, MAX_ACTION_SPEED, MAX_STAMINA_HEAL_TICK_COUNTER, TILE_SIZE_F32,
        UI_BORDER_F32,
    },
    dialog::DialogAction,
    engine::state::RunState,
    inventory::InventoryAction,
    maps::zone::{TileType, Zone},
    spawning::spawner::Spawn, utils::common::Utils,
};

#[derive(PartialEq, Debug)]
pub enum SpecialViewMode {
    ZapTargeting,
    Smell,
}

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
            let mut players = ecs_world
                .query::<(&mut Position, &mut Viewshed)>()
                .with::<&Player>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            for (player_entity, (position, viewshed)) in &mut players {
                let destination_index =
                    Zone::get_index_from_xy(position.x + delta_x, position.y + delta_y);

                //Search for potential targets (must have CombatStats component)
                for &potential_target in zone.tile_content[destination_index].iter() {
                    let has_combat_stats = ecs_world
                        .satisfies::<&CombatStats>(potential_target)
                        .expect("Entity has no CombatStats");

                    if has_combat_stats {
                        attacker_target = Some((player_entity, potential_target));
                    }
                }

                // Move if not attacking or destination is not blocked
                if attacker_target.is_none() && !zone.blocked_tiles[destination_index] {
                    position.x = (position.x + delta_x).clamp(0, MAP_WIDTH - 1);
                    position.y = (position.y + delta_y).clamp(0, MAP_HEIGHT - 1);
                    viewshed.must_recalculate = true;
                    zone.blocked_tiles[Zone::get_index_from_xy(position.x, position.y)] = true;
                    return_state = RunState::DoTick;
                }
            }
        }

        // Attack if needed
        if let Some((attacker, target)) = attacker_target {
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
            let player_entity = Player::get_entity(ecs_world);
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
                            run_state = Player::pick_up(ecs_world);
                        }

                        //Eat item
                        'e' => {
                            run_state = Player::try_eat(ecs_world, player_entity);
                        }

                        //Apply item
                        'a' => {
                            clear_input_queue();
                            run_state = RunState::ShowInventory(InventoryAction::Apply);
                        }

                        //DEBUG ONLY KILL
                        'k' => {
                            run_state = RunState::GameOver;
                        }

                        '>' => {
                            run_state = Player::try_next_level(ecs_world);
                        }

                        //Drop item
                        'd' => {
                            clear_input_queue();
                            run_state = RunState::ShowInventory(InventoryAction::Drop);
                        }

                        //Equip item
                        'f' => {
                            clear_input_queue();
                            run_state = RunState::ShowInventory(InventoryAction::Equip);
                        }

                        //Invoke item
                        'i' => {
                            clear_input_queue();
                            run_state = RunState::ShowInventory(InventoryAction::Invoke);
                        }

                        //Quaff item
                        'q' => {
                            clear_input_queue();

                            // Drink from river
                            run_state = Player::try_drink(ecs_world, player_entity);
                        }

                        //Smell action
                        's' => {
                            clear_input_queue();
                            run_state = RunState::MouseTargeting(SpecialViewMode::Smell);
                        }

                        _ => {}
                    }
                }
            }
        }

        // Wait if any real action is taken
        if run_state == RunState::DoTick {
            // Reset heal counter if the player did not wait through space or . key
            if !is_actively_waiting {
                Player::reset_heal_counter(ecs_world);
            }
            Player::wait_after_action(ecs_world);
        }

        run_state
    }

    /// Checks mouse input
    pub fn checks_input_for_targeting(
        ecs_world: &mut World,
        special_view_mode: SpecialViewMode,
    ) -> RunState {
        // ESC for escaping targeting without using Invokable
        if is_key_down(KeyCode::Escape) {
            return RunState::WaitingPlayerInput;
        } else if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();

            let rounded_x = (((mouse_x - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;
            let rounded_y = (((mouse_y - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;

            let player_entity = Player::get_entity(ecs_world);

            match special_view_mode {
                SpecialViewMode::ZapTargeting => {
                    let mut is_valid_tile = false;
                    // Scope for keeping borrow checker quiet
                    {
                        let mut zone_query = ecs_world.query::<&Zone>();
                        let (_, zone) = zone_query
                            .iter()
                            .last()
                            .expect("Zone is not in hecs::World");
                        // Make sure that we are targeting a valid tile
                        let index = Zone::get_index_from_xy(rounded_x, rounded_y);
                        if index < zone.visible_tiles.len() {
                            is_valid_tile = zone.visible_tiles[index];
                        }
                    }

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
                SpecialViewMode::Smell => {
                    let _ = ecs_world.insert_one(
                        player_entity,
                        WantsToSmell {
                            target: (rounded_x, rounded_y),
                        },
                    );
                    return RunState::WaitingPlayerInput;
                }
            }
        }

        RunState::MouseTargeting(special_view_mode)
    }

    /// Picks up something to store in backpack
    fn pick_up(ecs_world: &mut World) -> RunState {
        let player_entity = Player::get_entity(ecs_world);

        let mut picked_something = false;
        match Player::take_from_map(ecs_world) {
            None => {}
            Some(item) => {
                picked_something = true;
                println!("pick_up {:?}", item);
                let _ = ecs_world.insert_one(player_entity, WantsItem { item });
            }
        }

        if !picked_something {
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            game_log
                .entries
                .push("There is nothing here to pick up".to_string());

            RunState::WaitingPlayerInput
        } else {
            // Reset heal counter if the player did pick up something
            Player::reset_heal_counter(ecs_world);

            RunState::DoTick
        }
    }

    /// Takes something from map.
    fn take_from_map(ecs_world: &mut World) -> Option<Entity> {
        let mut target_item: Option<Entity> = None;

        let mut player_query = ecs_world.query::<&Position>().with::<&Player>();
        let (_, position) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");
        let player_position = position;

        let mut items = ecs_world.query::<(&Item, &Position)>();
        // Get item
        for (item_entity, (_tem, item_position)) in &mut items {
            if player_position.x == item_position.x && player_position.y == item_position.y {
                target_item = Some(item_entity);
            }
        }

        target_item
    }

    /// Try to eat. Return new Runstate
    fn try_eat(ecs_world: &mut World, _player_entity: Entity) -> RunState {
        clear_input_queue();
        let item_on_ground = Player::take_from_map(ecs_world);

        // Is really Edible?
        if let Some(item) = item_on_ground
            && ecs_world.satisfies::<&Edible>(item).unwrap_or(false)
        {
            return RunState::ShowDialog(DialogAction::Eat(item));
        }

        RunState::ShowInventory(InventoryAction::Eat)
    }

    /// Try to drink. Return new Runstate and true if it can heal
    fn try_drink(ecs_world: &mut World, player_entity: Entity) -> RunState {
        let there_is_river_here: bool;

        // Scope for keeping borrow checker quiet
        {
            let mut zone_query = ecs_world.query::<&Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            let mut player_query = ecs_world.query::<&Position>().with::<&Player>();
            let (_, position) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");
            let player_position = position;

            // Get Water from river
            there_is_river_here =
                zone.water_tiles[Zone::get_index_from_xy(player_position.x, player_position.y)];
        }

        if there_is_river_here {
            let river_entity = Spawn::river_water_entity(ecs_world);
            let _ = ecs_world.insert_one(player_entity, WantsToDrink { item: river_entity });

           return RunState::DoTick
        } else {
            // Drink a quaffable item on ground
            let item_on_ground = Player::take_from_map(ecs_world);

            // Is really Quaffable?
            if let Some(item) = item_on_ground
                && ecs_world.satisfies::<&Quaffable>(item).unwrap_or(false)
            {
                return RunState::ShowDialog(DialogAction::Quaff(item));
            }
        }

        // Show quaffable items in inventory
        RunState::ShowInventory(InventoryAction::Quaff)
    }

    fn try_next_level(ecs_world: &mut World) -> RunState {
        let player_position;
        let standing_on_tile;

        //Scope to keep borrow checker quiet
        {
            let mut player_query = ecs_world.query::<&Position>().with::<&Player>();
            let (_, position) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");
            player_position = position;

            let mut zone_query = ecs_world.query::<&Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");
            standing_on_tile =
                &zone.tiles[Zone::get_index_from_xy(player_position.x, player_position.y)];

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            game_log
                .entries
                .push("There is nothing here to pick up".to_string());

            //TODO skill check
            if standing_on_tile == &TileType::DownPassage {
                game_log.entries.push("You climb down...".to_string());
                return RunState::GoToNextZone;
            } else {
                game_log.entries.push("You can't go down here".to_string());
            }
        }

        RunState::WaitingPlayerInput
    }

    /// Extract Player's entity from world and return it with copy
    pub fn get_entity(ecs_world: &World) -> Entity {
        let mut player_query = ecs_world.query::<&Player>();
        let (player_entity, _) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        player_entity
    }
    /// Extract Player's entity id from world and return it with copy
    pub fn get_entity_id(ecs_world: &World) -> u32 {
        Player::get_entity(ecs_world).id()
    }
    /// Extract Player's entity from world and return it with copy
    pub fn can_act(ecs_world: &World) -> bool {
        let mut player_query = ecs_world.query::<(&Player, &MyTurn)>();

        player_query.iter().len() > 0
    }

    /// Reset heal counter. Usually when the player did anything but wait
    pub fn reset_heal_counter(ecs_world: &World) {
        let mut players = ecs_world
            .query::<(&mut CombatStats, &mut CanAutomaticallyHeal)>()
            .with::<&Player>();
        for (_, (stats, can_heal)) in &mut players {
            if stats.current_stamina < stats.max_stamina {
                can_heal.tick_counter = MAX_STAMINA_HEAL_TICK_COUNTER
            }
        }
    }

    /// Wait some ticks after action is taken
    // TODO this is only for player, but must be made common to all entities or else turn system does not work!
    pub fn wait_after_action(ecs_world: &mut World) {
        let player = Player::get_entity(ecs_world);
        let speed;

        // Scope for keeping borrow checker quiet
        {
            speed = ecs_world
                .get::<&CombatStats>(player)
                .expect("Entity has no CombatStats")
                .speed;
        }
        
        Utils::wait_after_action(ecs_world, player, speed);
    }

    /// Utility method for FOV forced recalculation
    pub fn force_view_recalculation(ecs_world: &World) {
        let mut player_viewshed = ecs_world
            .get::<&mut Viewshed>(Player::get_entity(ecs_world))
            .expect("Player entity does not have a Viewshed");
        player_viewshed.must_recalculate = true;
    }
}
