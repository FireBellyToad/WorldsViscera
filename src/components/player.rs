use crate::components::actions::{WantsToDig, WantsToTrade};
use crate::components::combat::{Grappled, SufferingDamage, WantsToShoot};
use crate::components::common::{
    Diggable, Hates, Immunity, ImmunityTypeEnum, Inspectable, Named, WillChat,
};
use crate::components::items::{RangedWeapon, ShopOwner};
use crate::constants::{ACID_DECAL_DAMAGE_DICE, NEXT_TO_DISTANCE, STANDARD_ACTION_MULTIPLIER};
use crate::engine::state::GameState;
use crate::utils::common::ItemsInBackpack;
use crate::utils::roll::Roll;
use crate::{components::actions::WantsToInvoke, maps::zone::DecalType};
use hecs::{Component, Entity, World};
use macroquad::input::{
    KeyCode, MouseButton, clear_input_queue, get_char_pressed, get_key_pressed, is_key_down,
    is_mouse_button_down, mouse_position,
};

use crate::{
    components::{
        actions::{WantsItem, WantsToDrink, WantsToSmell},
        combat::{CombatStats, WantsToMelee, WantsToZap},
        common::{MyTurn, Position, Viewshed},
        items::{Edible, Item, Quaffable},
    },
    constants::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE_F32, UI_BORDER_F32},
    engine::state::RunState,
    inventory::InventoryAction,
    maps::zone::{TileType, Zone},
    spawning::spawner::Spawn,
    utils::common::Utils,
    utils::dialog::DialogAction,
};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SpecialViewMode {
    ZapTargeting,
    RangedTargeting,
    Smell,
    Inspecting,
}

/// Player struct
pub struct Player {}

impl Player {
    ///
    /// Try to move player
    ///
    fn try_move(delta_x: i32, delta_y: i32, game_state: &mut GameState) {
        game_state.run_state = RunState::WaitingPlayerInput;
        let mut attacker_target: Option<(Entity, Entity)> = None;
        let mut digger_target: Option<(Entity, Entity, Entity)> = None;
        let mut waiter_speed_list: Vec<(Entity, i32)> = Vec::new();
        let mut remove_grappled: bool = false;

        // Scope for keeping borrow checker quiet
        {
            let ecs_world = &mut game_state.ecs_world;
            let mut players = ecs_world
                .query::<(
                    &mut Position,
                    &mut Viewshed,
                    &CombatStats,
                    &mut SufferingDamage,
                    Option<&Grappled>,
                    &Immunity,
                )>()
                .with::<&Player>();

            let zone = game_state
                .current_zone
                .as_mut()
                .expect("must have Some Zone");

            for (
                player_entity,
                (position, viewshed, stats, suffering_damage, grappled_opt, immunity),
            ) in &mut players
            {
                // Check if player is on slime before moving away and is not immune to slip
                if let Some(special_tile) = zone
                    .decals_tiles
                    .get(&Zone::get_index_from_xy(&position.x, &position.y))
                    && let DecalType::Slime = special_tile
                    && !immunity.to.contains_key(&ImmunityTypeEnum::Slipping)
                {
                    // Do DEX saving or slip on slime!
                    if stats.current_dexterity < Roll::d20() {
                        game_state.game_log.add_entry("You slip on the slime!");

                        game_state.run_state = RunState::DoTick;
                        break;
                    }
                }

                let destination_index =
                    Zone::get_index_from_xy(&(position.x + delta_x), &(position.y + delta_y));

                let player_dig_tool = ecs_world.query::<ItemsInBackpack>().iter().find_map(
                    |(item, (_, in_backpack, .., equipped, _, _, dig_tool))| {
                        if in_backpack.owner.id() == player_entity.id()
                            && equipped.is_some()
                            && dig_tool.is_some()
                        {
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
                        game_state
                            .game_log
                            .add_entry("You have no digging tool to use on this wall");
                    }
                }

                // Move if not attacking or destination is not blocked
                if attacker_target.is_none()
                    && digger_target.is_none()
                    && !zone.blocked_tiles[destination_index]
                {
                    // Check if player is grappled and try to escape.
                    // Placed if here so Player can still attack grappler
                    if let Some(grappled) = grappled_opt
                        && let Ok(mut g_query) =
                            ecs_world.query_one::<(&Named, &CombatStats)>(grappled.by)
                    {
                        remove_grappled = true;

                        let (grappler_name, grappler_stats) =
                            g_query.get().expect("g_query must have result");
                        // Try to escape grapple
                        if Roll::d20() <= stats.current_dexterity {
                            game_state.game_log.add_entry(&format!(
                                "You free yourself from the {}'s grasp!",
                                grappler_name.name
                            ));

                            // Grappler lose turn
                            waiter_speed_list.push((grappled.by, grappler_stats.speed));
                        } else {
                            game_state.game_log.add_entry(&format!(
                                "You cant' escape the {}'s grasp!",
                                grappler_name.name
                            ));
                            game_state.run_state = RunState::DoTick;
                            break;
                        }
                    } else {
                        zone.blocked_tiles[Zone::get_index_from_xy(&position.x, &position.y)] =
                            false;
                        position.x = (position.x + delta_x).clamp(0, MAP_WIDTH - 1);
                        position.y = (position.y + delta_y).clamp(0, MAP_HEIGHT - 1);
                        viewshed.must_recalculate = true;
                        zone.blocked_tiles[Zone::get_index_from_xy(&position.x, &position.y)] =
                            true;

                        // Check if player has stepped on a acid
                        if let Some(special_tile) = zone
                            .decals_tiles
                            .get(&Zone::get_index_from_xy(&position.x, &position.y))
                            && let DecalType::Acid = special_tile
                            && !immunity.to.contains_key(&ImmunityTypeEnum::DamagingFloor)
                        {
                            // Do DEX saving or be damaged!
                            if stats.current_dexterity < Roll::d20() {
                                game_state
                                    .game_log
                                    .add_entry("You burn yourself on the acid!");
                                suffering_damage.damage_received +=
                                    Roll::dice(1, ACID_DECAL_DAMAGE_DICE);
                            }
                        }
                    }

                    game_state.run_state = RunState::DoTick;
                }
            }
        }

        // if return_state == RunState::DoTick here, than is moving, needs to wait!
        if game_state.run_state == RunState::DoTick {
            Player::wait_after_action(game_state, STANDARD_ACTION_MULTIPLIER);
        }

        // Attack if needed
        if let Some((attacker, target)) = attacker_target {
            let _ = game_state
                .ecs_world
                .insert_one(attacker, WantsToMelee { target });
            game_state.run_state = RunState::DoTick;
        }

        // Dig if needed
        if let Some((digger, tool, target)) = digger_target {
            let _ = game_state
                .ecs_world
                .insert_one(digger, WantsToDig { target, tool });
            game_state.run_state = RunState::DoTick;
        }

        // Remove grapple if player was grappled
        if remove_grappled {
            let _ = game_state.ecs_world.remove_one::<Grappled>(
                game_state.current_player_entity.expect("must have player"),
            );
        }

        // TODO account speed penalties
        for (waiter, speed) in waiter_speed_list {
            Utils::wait_after_action(&mut game_state.ecs_world, waiter, speed);
        }
    }

    ///
    /// Handle player input
    ///
    pub fn checks_keyboard_input(game_state: &mut GameState) {
        game_state.run_state = RunState::WaitingPlayerInput;
        let mut check_chars_pressed = false;

        // Player movement
        match get_key_pressed() {
            None => game_state.run_state = RunState::WaitingPlayerInput, // Nothing happened
            Some(key) => match key {
                KeyCode::Kp4 | KeyCode::Left => Player::try_move(-1, 0, game_state),
                KeyCode::Kp6 | KeyCode::Right => Player::try_move(1, 0, game_state),
                KeyCode::Kp8 | KeyCode::Up => Player::try_move(0, -1, game_state),
                KeyCode::Kp2 | KeyCode::Down => Player::try_move(0, 1, game_state),

                // Diagonals
                KeyCode::Kp9 => Player::try_move(1, -1, game_state),
                KeyCode::Kp7 => Player::try_move(-1, -1, game_state),
                KeyCode::Kp3 => Player::try_move(1, 1, game_state),
                KeyCode::Kp1 => Player::try_move(-1, 1, game_state),

                // Skip turn doing nothing, so you can heal
                KeyCode::Space => {
                    Player::wait_after_action(game_state, STANDARD_ACTION_MULTIPLIER);
                    game_state.run_state = RunState::DoTick;
                }

                // Something was pressed but is not in this match?
                // Check for characters pressed
                _ => check_chars_pressed = true,
            },
        }

        // Player commands. Handed with characters to manage different keyboards layout
        // Do it only if no keys were pressed or else Arrow keys and space will not work properly
        // IMPORTANT: call clear_input_queue() on action that opens inventory
        if check_chars_pressed {
            match get_char_pressed() {
                None => game_state.run_state = RunState::WaitingPlayerInput, // Nothing happened
                Some(char) => {
                    match char {
                        // Skip turn doing nothing, so you can heal
                        '.' => {
                            game_state.run_state = RunState::DoTick;
                            Player::wait_after_action(game_state, STANDARD_ACTION_MULTIPLIER);
                        }

                        //Pick up
                        'p' => {
                            Player::pick_up(game_state);
                        }

                        //Eat item
                        'e' => {
                            Player::try_eat(game_state);
                        }

                        //Apply item
                        'a' => {
                            clear_input_queue();
                            game_state.run_state = RunState::ShowInventory(InventoryAction::Apply);
                        }

                        //Kill himself in debug mode only
                        'k' if game_state.debug_mode => {
                            game_state.run_state = RunState::GameOver;
                        }

                        '>' => {
                            Player::try_next_level(game_state);
                        }

                        //Drop item
                        'd' => {
                            clear_input_queue();
                            game_state.run_state = RunState::ShowInventory(InventoryAction::Drop);
                        }

                        //Equip item
                        'f' => {
                            clear_input_queue();
                            game_state.run_state = RunState::ShowInventory(InventoryAction::Equip);
                        }

                        //Quaff item
                        'q' => {
                            clear_input_queue();

                            // Drink from river
                            Player::try_drink(game_state);
                        }

                        //Smell (whiff) action
                        'w' => {
                            clear_input_queue();
                            game_state.run_state = RunState::MouseTargeting(SpecialViewMode::Smell);
                        }

                        //Shoot
                        's' => {
                            clear_input_queue();
                            // TODO change with equipped ranged weapon, so check if it is ranged and equipped before
                            // entering in target mode
                            Player::try_shoot(game_state);
                        }

                        //Trade item to shop owner
                        't' => {
                            Player::try_trade(game_state);
                        }

                        //Trade item to shop owner
                        'c' => {
                            Player::try_chat(game_state);
                        }

                        'l' => {
                            Player::try_inspect(game_state);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Checks mouse input
    pub fn checks_input_for_targeting(
        game_state: &mut GameState,
        special_view_mode: SpecialViewMode,
    ) {
        let player_entity = game_state.current_player_entity.expect("must be Some");
        let mut to_despawn: Option<(Entity, &str)> = None;
        // Keep RunState to MouseTargeting running while player is targeting
        game_state.run_state = RunState::MouseTargeting(special_view_mode);
        // ESC for escaping targeting without using Invokable
        if is_key_down(KeyCode::Escape) {
            // Remove components linked to view mode to avoid bugs
            match special_view_mode {
                SpecialViewMode::ZapTargeting => {
                    let _ = game_state
                        .ecs_world
                        .remove_one::<WantsToInvoke>(player_entity);
                }
                SpecialViewMode::RangedTargeting => {
                    let _ = game_state
                        .ecs_world
                        .remove_one::<WantsToShoot>(player_entity);
                }
                _ => {}
            }
            game_state.run_state = RunState::WaitingPlayerInput;
        } else if is_mouse_button_down(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();

            let rounded_x = (((mouse_x - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;
            let rounded_y = (((mouse_y - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;

            let zone = game_state
                .current_zone
                .as_mut()
                .expect("must have Some Zone");

            match special_view_mode {
                SpecialViewMode::ZapTargeting | SpecialViewMode::RangedTargeting => {
                    let mut is_valid_tile = false;
                    // Scope for keeping borrow checker quiet
                    {
                        // Make sure that we are targeting a valid tile
                        let index = Zone::get_index_from_xy(&rounded_x, &rounded_y);
                        if index < zone.visible_tiles.len() {
                            is_valid_tile = zone.visible_tiles[index];
                        }
                    }

                    if is_valid_tile {
                        let _ = game_state.ecs_world.insert_one(
                            player_entity,
                            WantsToZap {
                                target: (rounded_x, rounded_y),
                            },
                        );

                        game_state.run_state = RunState::DoTick;
                    }
                }
                SpecialViewMode::Inspecting => {
                    let mut is_valid_tile = false;
                    // Scope for keeping borrow checker quiet
                    {
                        // Make sure that we are targeting a valid tile
                        let index = Zone::get_index_from_xy(&rounded_x, &rounded_y);
                        if index < zone.visible_tiles.len() {
                            is_valid_tile = zone.visible_tiles[index];
                        }
                    }

                    // Inspect the first entity found in tile and
                    // query Inspectable and Named components
                    let first_ent_fount =
                        zone.tile_content[Zone::get_index_from_xy(&rounded_x, &rounded_y)].first();
                    if is_valid_tile
                        && let Some(ent) = first_ent_fount
                        && let Ok(mut q) = game_state
                            .ecs_world
                            .query_one::<(&Inspectable, &Named)>(*ent)
                        && let Some((inspectable, named)) = q.get()
                    {
                        // Despawn the entity if it should be despawned
                        if inspectable.despawn_on_inspect {
                            to_despawn = Some((*ent, named.name));
                        }
                        game_state.run_state = RunState::ShowDialog(DialogAction::ShowMessage(
                            inspectable.description,
                        ));
                    } else {
                        game_state.game_log.add_entry("Nothing to inspect here");
                        game_state.run_state = RunState::WaitingPlayerInput;
                    }
                }
                SpecialViewMode::Smell => {
                    let _ = game_state.ecs_world.insert_one(
                        player_entity,
                        WantsToSmell {
                            target: (rounded_x, rounded_y),
                        },
                    );
                    game_state.run_state = RunState::WaitingPlayerInput;
                }
            }

            // Despawn any entity that should be despawned
            // and reset the tile to floor and log
            if let Some((e, name)) = to_despawn {
                let _ = game_state.ecs_world.despawn(e);
                zone.tiles[Zone::get_index_from_xy(&rounded_x, &rounded_y)] = TileType::Floor;
                game_state
                    .game_log
                    .add_entry(&format!("The {} vanishes in the darkness", name));
            }
        }
    }

    /// Picks up something to store in backpack
    fn pick_up(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state.current_player_entity.expect("Must be Some");

        if let Some(item) = Player::take_from_map::<Item>(ecs_world, player_entity) {
            // Check if the item is being stolen from a shop
            if Utils::get_item_owner(ecs_world, item).is_some() {
                //Show Dialog
                game_state.run_state = RunState::ShowDialog(DialogAction::StealPick(item));
            } else {
                // Reset heal counter if the player did pick up something
                let _ = ecs_world.insert_one(
                    player_entity,
                    WantsItem {
                        items: vec![item],
                        was_bought: false,
                    },
                );

                game_state.run_state = RunState::DoTick;
            }
        } else {
            game_state
                .game_log
                .add_entry("There is nothing here to pick up");

            game_state.run_state = RunState::WaitingPlayerInput;
        }
    }

    /// Takes something from map.
    fn take_from_map<T: Component>(ecs_world: &mut World, player_entity: Entity) -> Option<Entity> {
        let mut target_item: Option<Entity> = None;

        let position = ecs_world
            .get::<&Position>(player_entity)
            .expect("Player must have a position");

        let mut items = ecs_world.query::<(&Item, &Position)>();
        // Get item
        for (item_entity, (_tem, item_position)) in &mut items {
            if position.x == item_position.x
                && position.y == item_position.y
                && ecs_world.satisfies::<&T>(item_entity).unwrap_or(false)
            {
                target_item = Some(item_entity);
            }
        }

        target_item
    }

    /// Try to eat. Return new Runstate
    fn try_eat(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state.current_player_entity.expect("Must be Some");

        clear_input_queue();
        let item_on_ground = Player::take_from_map::<Edible>(ecs_world, player_entity);

        // Is really Edible?
        if let Some(item) = item_on_ground
            && ecs_world.satisfies::<&Edible>(item).unwrap_or(false)
        {
            // Check if the item is being stolen from a shop
            if Utils::get_item_owner(ecs_world, item).is_some() {
                //Show Theft Dialog

                game_state.run_state = RunState::ShowDialog(DialogAction::StealEat(item));
            } else {
                //Show eat Dialog

                game_state.run_state = RunState::ShowDialog(DialogAction::Eat(item));
            }
        } else {
            game_state.run_state = RunState::ShowInventory(InventoryAction::Eat)
        }
    }

    /// Try to drink. Return new Runstate and true if it can heal
    fn try_drink(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state
            .current_player_entity
            .expect("must have some player");
        let there_is_river_here: bool;

        // Show quaffable items in inventory
        game_state.run_state = RunState::ShowInventory(InventoryAction::Quaff);
        // Scope for keeping borrow checker quiet
        {
            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");
            let position = ecs_world
                .get::<&Position>(player_entity)
                .expect("Player must have a position");
            let player_position = position;

            // Get Water from river
            there_is_river_here =
                zone.water_tiles[Zone::get_index_from_xy(&player_position.x, &player_position.y)];
        }

        if there_is_river_here {
            let river_entity = Spawn::river_water_entity(ecs_world);
            let _ = ecs_world.insert_one(player_entity, WantsToDrink { item: river_entity });

            game_state.run_state = RunState::DoTick;
        } else {
            // Drink a quaffable item on ground
            let item_on_ground = Player::take_from_map::<Quaffable>(ecs_world, player_entity);

            // Is really Quaffable?
            if let Some(item) = item_on_ground
                && ecs_world.satisfies::<&Quaffable>(item).unwrap_or(false)
            {
                game_state.run_state = RunState::ShowDialog(DialogAction::Quaff(item));
            }
        }
    }

    fn try_next_level(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let standing_on_tile;

        game_state.run_state = RunState::WaitingPlayerInput;
        let player_entity = game_state
            .current_player_entity
            .expect("Must have player entity");
        //Scope to keep borrow checker quiet
        {
            let position = ecs_world
                .get::<&Position>(player_entity)
                .expect("Player must have a position");

            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");
            standing_on_tile = &zone.tiles[Zone::get_index_from_xy(&position.x, &position.y)];

            game_state
                .game_log
                .add_entry("There is nothing here to pick up");

            //TODO skill check
            if standing_on_tile == &TileType::DownPassage {
                game_state.game_log.add_entry("You climb down...");
                game_state.run_state = RunState::GoToNextZone;
            } else {
                game_state.game_log.add_entry("You can't go down here");
            }
        }
    }

    /// Try to shoot a ranged weapon: checks if the player has a ranged weapon equipped
    fn try_shoot(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state.current_player_entity.expect("must be some");
        let mut weapon_opt: Option<Entity> = None;
        //Scope to keep borrow checker quiet
        {
            // Must move the query in ranged_weapons_in_backpacks_query
            // because we need to check if .is_empty() later,
            // and can't be dropped before of that
            let mut ranged_weapons_in_backpacks_query = ecs_world.query::<ItemsInBackpack>();

            let player_ranged_weapon: Option<(Entity, ItemsInBackpack)> =
                ranged_weapons_in_backpacks_query.iter().find(
                    |(_, (_, in_backpack, .., equipped, ranged, _))| {
                        in_backpack.owner.id() == player_entity.id()
                            && equipped.is_some()
                            && ranged.is_some()
                    },
                );

            if let Some(weapon) = player_ranged_weapon {
                weapon_opt = Some(weapon.0);
            } else {
                game_state
                    .game_log
                    .add_entry("You don't have a ranged weapon equipped");
            }
        }

        // if has weapon, go check ammo or go to target mode
        if let Some(weapon) = weapon_opt {
            //Scope to keep borrow checker quiet
            {
                // Check if the Player has ammo available
                let mut q = ecs_world
                    .query_one::<(&RangedWeapon, &Named)>(weapon)
                    .unwrap_or_else(|_| panic!("No Named RangedWeapon for entity {:?}", weapon)); // TODO maybe refactor this with InflictsDamage component;
                let (weapon_stats, weapon_named) = q.get().expect("Entity has no RangedWeapon");

                // If no ammo available, abort without advancing to next tick
                if weapon_stats.ammo_count_total == 0 {
                    game_state.game_log.add_entry(&format!(
                        "You don't have any ammunition for your {}",
                        weapon_named.name
                    ));
                    game_state.run_state = RunState::WaitingPlayerInput;
                    return;
                }
            }

            // Ready to shoot
            let _ = ecs_world.insert_one(player_entity, WantsToShoot { weapon });
            game_state.run_state = RunState::MouseTargeting(SpecialViewMode::RangedTargeting);
        } else {
            // No ranged weapon equipped, abort without advancing to next tick
            game_state.run_state = RunState::WaitingPlayerInput;
        }
    }

    /// Extract Player's entity from world and return it with copy
    pub fn can_act(ecs_world: &World, player_entity: Entity) -> bool {
        ecs_world
            .satisfies::<&MyTurn>(player_entity)
            .unwrap_or(false)
    }

    /// Wait some ticks after action is taken
    pub fn wait_after_action(game_state: &mut GameState, multiplier: i32) {
        let ecs_world = &mut game_state.ecs_world;
        let player = game_state
            .current_player_entity
            .expect("Player should be set");
        let speed;

        // Scope for keeping borrow checker quiet
        {
            speed = ecs_world
                .get::<&CombatStats>(player)
                .expect("Entity has no CombatStats")
                .speed;
        }

        Utils::wait_after_action(&mut game_state.ecs_world, player, speed * multiplier);
    }

    /// Utility method for FOV forced recalculation
    pub fn force_view_recalculation(game_state: &mut GameState) {
        let mut player_viewshed = game_state
            .ecs_world
            .get::<&mut Viewshed>(
                game_state
                    .current_player_entity
                    .expect("must be some entity"),
            )
            .expect("Player entity does not have a Viewshed");
        player_viewshed.must_recalculate = true;
    }

    /// Try to trade an item to a potetial shop owner
    pub fn try_trade(game_state: &mut GameState) {
        game_state.run_state = RunState::WaitingPlayerInput;
        let mut owner_entity: Option<Entity> = None;
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state
            .current_player_entity
            .expect("must be some entity");
        // Scope for keeping borrow checker quiet
        {
            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");
            let mut player_query = ecs_world
                .query_one::<(&Viewshed, &Position)>(player_entity)
                .expect("Player is not in hecs::World");
            let (viewshed, player_pos) = player_query.get().expect("Player is not in hecs::World");

            // Search for visibile shop owners in the visibile tiles
            for &index in &viewshed.visible_tiles {
                for &entity in &zone.tile_content[index] {
                    // If entity is a non-angered shop owner, try to trade
                    if let Ok(mut res) =
                        ecs_world.query_one::<(&ShopOwner, &Hates, &Position)>(entity)
                        && let Some((_, hates, pos)) = res.get()
                        && !hates.list.contains(&player_entity.id())
                    {
                        if Utils::distance(&player_pos.x, &pos.x, &player_pos.y, &pos.y)
                            <= NEXT_TO_DISTANCE
                        {
                            owner_entity = Some(entity);
                            break;
                        } else {
                            game_state
                                .game_log
                                .add_entry("You see someone who may trade, but it's too far away");
                            //We must guarantee only one shop owner per zone
                            game_state.run_state = RunState::WaitingPlayerInput;
                            return; //abort control flow
                        }
                    }
                }

                // Avoid unnecessary iterations
                if owner_entity.is_some() {
                    break;
                }
            }

            if owner_entity.is_none() {
                game_state
                    .game_log
                    .add_entry("You can't see anyone willing to trade");
            }
        }

        // If we found a shop owner, offer them an item
        if let Some(target) = owner_entity {
            let _ = ecs_world.insert_one(player_entity, WantsToTrade { target, item: None });
            game_state.run_state = RunState::ShowInventory(InventoryAction::Trade);
        }
    }

    /// Try to chat an item to a potetial shop owner
    pub fn try_chat(game_state: &mut GameState) {
        game_state.run_state = RunState::WaitingPlayerInput;
        let mut chatter_entity: Option<&'static str> = None;
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state
            .current_player_entity
            .expect("must be some entity");
        // Scope for keeping borrow checker quiet
        {
            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");
            let mut player_query = ecs_world
                .query_one::<(&Viewshed, &Position)>(player_entity)
                .expect("Player is not in hecs::World");
            let (viewshed, player_pos) = player_query.get().expect("Player is not in hecs::World");

            // Search for visibile shop owners in the visibile tiles
            for &index in &viewshed.visible_tiles {
                for &entity in &zone.tile_content[index] {
                    // If is a non-angered chattable, try to chat with it
                    if let Ok(mut res) =
                        ecs_world.query_one::<(&WillChat, &Hates, &Position)>(entity)
                        && let Some((will_chat, hates, pos)) = res.get()
                        && !hates.list.contains(&player_entity.id())
                    {
                        // must be adjacent to player
                        if Utils::distance(&player_pos.x, &pos.x, &player_pos.y, &pos.y)
                            <= NEXT_TO_DISTANCE
                        {
                            let diag_len = will_chat.dialogues.len();
                            chatter_entity = Some(
                                will_chat.dialogues[Roll::dice(1, diag_len as i32) as usize - 1],
                            );
                            break;
                        } else {
                            game_state
                                .game_log
                                .add_entry("You see someone who may chat, but it's too far away");
                            //We must guarantee only one shop owner per zone
                            game_state.run_state = RunState::WaitingPlayerInput;
                            return; //abort control flow
                        }
                    }
                }

                // Avoid unnecessary iterations
                if chatter_entity.is_some() {
                    break;
                }
            }

            if chatter_entity.is_none() {
                game_state
                    .game_log
                    .add_entry("You can't see anyone willing to chat");
            }
        }

        // If we found a shop owner, offer them an item
        if let Some(message) = chatter_entity {
            Player::wait_after_action(game_state, STANDARD_ACTION_MULTIPLIER);
            game_state.run_state = RunState::ShowDialog(DialogAction::ShowMessage(message));
        }
    }

    pub fn try_inspect(game_state: &mut GameState) {
        game_state.run_state = RunState::MouseTargeting(SpecialViewMode::Inspecting);
    }
}
