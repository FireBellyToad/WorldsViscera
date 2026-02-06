use crate::components::actions::WantsToTrade;
use crate::components::combat::{SufferingDamage, WantsToDig, WantsToShoot};
use crate::components::common::{Diggable, Named};
use crate::components::items::{DiggingTool, RangedWeapon, ShopOwner};
use crate::utils::common::ItemsInBackpack;
use crate::utils::roll::Roll;
use crate::{components::actions::WantsToInvoke, maps::zone::DecalType};
use hecs::{Entity, World};
use macroquad::input::{
    KeyCode, MouseButton, clear_input_queue, get_char_pressed, get_key_pressed, is_key_down,
    is_mouse_button_down, mouse_position,
};

use crate::{
    components::{
        actions::{WantsItem, WantsToDrink, WantsToSmell},
        combat::{CombatStats, WantsToMelee, WantsToZap},
        common::{GameLog, MyTurn, Position, Viewshed},
        health::CanAutomaticallyHeal,
        items::{Edible, Item, Quaffable},
    },
    constants::{
        MAP_HEIGHT, MAP_WIDTH, MAX_STAMINA_HEAL_TICK_COUNTER, TILE_SIZE_F32, UI_BORDER_F32,
    },
    dialog::DialogAction,
    engine::state::RunState,
    inventory::InventoryAction,
    maps::zone::{TileType, Zone},
    spawning::spawner::Spawn,
    utils::common::Utils,
};

#[derive(PartialEq, Debug, Clone)]
pub enum SpecialViewMode {
    ZapTargeting,
    RangedTargeting,
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
        let mut digger_target: Option<(Entity, Entity, Entity)> = None;

        // Scope for keeping borrow checker quiet
        {
            let mut players = ecs_world
                .query::<(
                    &mut Position,
                    &mut Viewshed,
                    &CombatStats,
                    &mut SufferingDamage,
                )>()
                .with::<&Player>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (player_entity, (position, viewshed, stats, suffering_damage)) in &mut players {
                // Check if player is on slime before moving away
                if let Some(special_tile) = zone
                    .decals_tiles
                    .get(&Zone::get_index_from_xy(&position.x, &position.y))
                    && let DecalType::Slime = special_tile
                {
                    // Do DEX saving or slip on slime!
                    if stats.current_dexterity < Roll::d20() {
                        game_log.entries.push("You slip on the slime!".to_string());

                        return_state = RunState::DoTick;
                        break;
                    }
                }

                let destination_index =
                    Zone::get_index_from_xy(&(position.x + delta_x), &(position.y + delta_y));

                let mut digging_tools_in_backpack =
                    ecs_world.query::<ItemsInBackpack>().with::<&DiggingTool>();

                let player_dig_tool = digging_tools_in_backpack.iter().find_map(
                    |(item, (_, in_backpack, _, _, _, _, _, _, equipped, _))| {
                        if in_backpack.owner.id() == player_entity.id() && equipped.is_some() {
                            Some(item)
                        } else {
                            None
                        }
                    },
                );

                //Search for potential targets (must have CombatStats component)
                for &potential_target in zone.tile_content[destination_index].iter() {
                    let has_combat_stats = ecs_world
                        .satisfies::<&CombatStats>(potential_target)
                        .unwrap_or(false);

                    if has_combat_stats {
                        attacker_target = Some((player_entity, potential_target));
                    }
                    let is_diggable = ecs_world
                        .satisfies::<&Diggable>(potential_target)
                        .unwrap_or(false);

                    if is_diggable && let Some(dig_tool) = player_dig_tool {
                        digger_target = Some((player_entity, dig_tool, potential_target));
                    } else if is_diggable {
                        game_log
                            .entries
                            .push("The crack is too tight to pass through".to_string());
                    }
                }

                // Move if not attacking or destination is not blocked
                if attacker_target.is_none()
                    && digger_target.is_none()
                    && !zone.blocked_tiles[destination_index]
                {
                    zone.blocked_tiles[Zone::get_index_from_xy(&position.x, &position.y)] = false;
                    position.x = (position.x + delta_x).clamp(0, MAP_WIDTH - 1);
                    position.y = (position.y + delta_y).clamp(0, MAP_HEIGHT - 1);
                    viewshed.must_recalculate = true;
                    zone.blocked_tiles[Zone::get_index_from_xy(&position.x, &position.y)] = true;

                    // Check if player has stepped on a acid
                    if let Some(special_tile) = zone
                        .decals_tiles
                        .get(&Zone::get_index_from_xy(&position.x, &position.y))
                        && let DecalType::Acid = special_tile
                    {
                        // Do DEX saving or be damaged!
                        if stats.current_dexterity < Roll::d20() {
                            game_log
                                .entries
                                .push("You burn yourself on the acid!".to_string());
                            suffering_damage.damage_received += Roll::dice(1, 3);
                        }
                    }

                    return_state = RunState::DoTick;
                }
            }
        }

        // if return_state == RunState::DoTick here, than is moving, needs to wait!
        if return_state == RunState::DoTick {
            Player::wait_after_action(ecs_world);
        }

        // Attack if needed
        if let Some((attacker, target)) = attacker_target {
            let _ = ecs_world.insert_one(attacker, WantsToMelee { target });
            return_state = RunState::DoTick;
        }

        // Dig if needed
        if let Some((digger, tool, target)) = digger_target {
            let _ = ecs_world.insert_one(digger, WantsToDig { target, tool });
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

                        //Smell (whiff) action
                        'w' => {
                            clear_input_queue();
                            run_state = RunState::MouseTargeting(SpecialViewMode::Smell);
                        }

                        //Shoot
                        's' => {
                            clear_input_queue();
                            // TODO change with equipped ranged weapon, so check if it is ranged and equipped before
                            // entering in target mode

                            run_state = Player::try_shoot(ecs_world, player_entity);
                        }

                        //Trade item to shop owner
                        't' => {
                            run_state = Player::try_trade(ecs_world, player_entity);
                        }

                        _ => {}
                    }
                }
            }
        }

        // Reset heal counter if the player did not wait through space or . key
        if run_state == RunState::DoTick && !is_actively_waiting {
            Player::reset_heal_counter(ecs_world);
        }

        run_state
    }

    /// Checks mouse input
    pub fn checks_input_for_targeting(
        ecs_world: &mut World,
        special_view_mode: SpecialViewMode,
    ) -> RunState {
        let player_entity = Player::get_entity(ecs_world);
        // ESC for escaping targeting without using Invokable
        if is_key_down(KeyCode::Escape) {
            // Remove components linked to view mode to avoid bugs
            match special_view_mode {
                SpecialViewMode::ZapTargeting => {
                    let _ = ecs_world.remove_one::<WantsToInvoke>(player_entity);
                }
                SpecialViewMode::RangedTargeting => {
                    let _ = ecs_world.remove_one::<WantsToShoot>(player_entity);
                }
                _ => {}
            }
            return RunState::WaitingPlayerInput;
        } else if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();

            let rounded_x = (((mouse_x - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;
            let rounded_y = (((mouse_y - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;

            match special_view_mode {
                SpecialViewMode::ZapTargeting | SpecialViewMode::RangedTargeting => {
                    let mut is_valid_tile = false;
                    // Scope for keeping borrow checker quiet
                    {
                        let mut zone_query = ecs_world.query::<&Zone>();
                        let (_, zone) = zone_query
                            .iter()
                            .last()
                            .expect("Zone is not in hecs::World");
                        // Make sure that we are targeting a valid tile
                        let index = Zone::get_index_from_xy(&rounded_x, &rounded_y);
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

        if let Some(item) = Player::take_from_map(ecs_world) {
            // Check if the item is being stolen from a shop
            if Utils::get_item_owner(ecs_world, item).is_some() {
                //Show Dialog
                RunState::ShowDialog(DialogAction::Steal_Pick(item))
            } else {
                // Reset heal counter if the player did pick up something
                let _ = ecs_world.insert_one(player_entity, WantsItem { items: vec![item] });
                Player::reset_heal_counter(ecs_world);

                RunState::DoTick
            }
        } else {
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");
            game_log
                .entries
                .push("There is nothing here to pick up".to_string());

            RunState::WaitingPlayerInput
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

        let mut items = ecs_world.query::<(&Item, &Position)>();
        // Get item
        for (item_entity, (_tem, item_position)) in &mut items {
            if position.x == item_position.x && position.y == item_position.y {
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
            // Check if the item is being stolen from a shop
            if Utils::get_item_owner(ecs_world, item).is_some() {
                //Show Theft Dialog
                return RunState::ShowDialog(DialogAction::Steal_Eat(item));
            } else {
                //Show eat Dialog
                return RunState::ShowDialog(DialogAction::Eat(item));
            }
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
                zone.water_tiles[Zone::get_index_from_xy(&player_position.x, &player_position.y)];
        }

        if there_is_river_here {
            let river_entity = Spawn::river_water_entity(ecs_world);
            let _ = ecs_world.insert_one(player_entity, WantsToDrink { item: river_entity });

            return RunState::DoTick;
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
        let standing_on_tile;

        //Scope to keep borrow checker quiet
        {
            let mut player_query = ecs_world.query::<&Position>().with::<&Player>();
            let (_, position) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");

            let mut zone_query = ecs_world.query::<&Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");
            standing_on_tile = &zone.tiles[Zone::get_index_from_xy(&position.x, &position.y)];

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

    /// Try to shoot a ranged weapon: checks if the player has a ranged weapon equipped
    fn try_shoot(ecs_world: &mut World, player_entity: Entity) -> RunState {
        let mut weapon_opt: Option<Entity> = None;
        //Scope to keep borrow checker quiet
        {
            let mut ranged_weapons_in_backpacks_query = ecs_world.query::<ItemsInBackpack>();

            let player_ranged_weapons: Vec<(Entity, ItemsInBackpack)> =
                ranged_weapons_in_backpacks_query
                    .iter()
                    .filter(
                        |(_, (_, in_backpack, _, _, _, _, _, _, equipped, ranged))| {
                            in_backpack.owner.id() == player_entity.id()
                                && equipped.is_some()
                                && ranged.is_some()
                        },
                    )
                    .collect();

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            if player_ranged_weapons.is_empty() {
                game_log
                    .entries
                    .push("You don't have a ranged weapon equipped".to_string());
            } else {
                // Should be one, anyway
                weapon_opt = Some(player_ranged_weapons[0].0);
            }
        }

        // if has weapon, go check ammo or go to target mode
        if let Some(weapon) = weapon_opt {
            //Scope to keep borrow checker quiet
            {
                // Check if the Player has ammo available
                let weapon_stats = ecs_world
                    .get::<&RangedWeapon>(weapon)
                    .expect("Entity has no RangedWeapon"); // TODO maybe refactor this with InflictsDamage component;

                // If no ammo available, abort without advancing to next tick
                if weapon_stats.ammo_count_total == 0 {
                    let mut game_log_query = ecs_world.query::<&mut GameLog>();
                    let (_, game_log) = game_log_query
                        .iter()
                        .last()
                        .expect("Game log is not in hecs::World");

                    let weapon_named = ecs_world
                        .get::<&Named>(weapon)
                        .expect("Entity has no Named");
                    game_log.entries.push(format!(
                        "You don't have any ammunition for your {}",
                        weapon_named.name
                    ));
                    return RunState::WaitingPlayerInput;
                }
            }

            // Ready to shoot
            let _ = ecs_world.insert_one(player_entity, WantsToShoot { weapon });
            return RunState::MouseTargeting(SpecialViewMode::RangedTargeting);
        }

        // No ranged weapon equipped, abort without advancing to next tick
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

    /// Try to trade an item to a potetial shop owner
    pub fn try_trade(ecs_world: &mut World, _player: Entity) -> RunState {
        let mut owner_entity: Option<Entity> = None;
        // Scope for keeping borrow checker quiet
        {
            let mut zone_query = ecs_world.query::<&Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            let mut player_query = ecs_world
                .query::<(&Viewshed, &Position)>()
                .with::<&Player>();
            let (_, (viewshed, player_pos)) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for &index in &viewshed.visible_tiles {
                for &entity in &zone.tile_content[index] {
                    if ecs_world.satisfies::<&ShopOwner>(entity).unwrap_or(false) {
                        let pos = ecs_world
                            .get::<&Position>(entity)
                            .expect("Entity does not have a Position");

                        if Utils::distance(&player_pos.x, &pos.x, &player_pos.y, &pos.y) <= 1.5 {
                            owner_entity = Some(entity);
                            break;
                        } else {
                            game_log.entries.push(
                                "You see someone who may trade, but it's too far away".to_string(),
                            );
                            //We must guarantee only one shop owner per zone
                            return RunState::WaitingPlayerInput;
                        }
                    }
                }

                // Avoid unnecessary iterations
                if owner_entity.is_some() {
                    break;
                }
            }

            game_log
                .entries
                .push("You can't see anyone willing to trade".to_string());
        }

        // If we found a shop owner, offer them an item
        if let Some(target) = owner_entity {
            let player_entity = Player::get_entity(ecs_world);
            let _ = ecs_world.insert_one(player_entity, WantsToTrade { target, item: None });
            return RunState::ShowInventory(InventoryAction::Trade);
        }

        RunState::WaitingPlayerInput
    }
}
