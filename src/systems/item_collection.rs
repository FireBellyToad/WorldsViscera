use hecs::Entity;

use crate::{
    components::{
        actions::WantsItem,
        combat::CombatStats,
        common::{GameLog, Hates, MyTurn, Named, Position},
        items::{InBackback, Item, Perishable, ToBeHarvested},
        monster::Small,
    },
    constants::{
        MAX_ITEMS_IN_BACKPACK, MAX_ITEMS_IN_BACKPACK_FOR_SMALL, OPTION_TO_CHAR_MAP,
        STARTING_ROT_COUNTER,
    },
    engine::state::GameState,
    maps::zone::Zone,
    utils::{common::Utils, roll::Roll},
};

pub struct ItemCollection {}

impl ItemCollection {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let mut item_owner_list: Vec<(Entity, Entity, char, i32)> = Vec::new();
        let mut failed_pick_upper: Vec<Entity> = Vec::new();
        let mut harvested_list: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut collectors = ecs_world
                .query::<(&WantsItem, &CombatStats, &Position, Option<&Small>, &Named)>()
                .with::<&MyTurn>();

            //Items in all backpacks
            let mut items_in_backpacks = ecs_world.query::<(&Item, &InBackback)>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");

            for (collector, (wants_item, stats, position, small, named_collector)) in
                &mut collectors
            {
                let mut last_assigned_char = ' ';
                for &item in &wants_item.items {
                    let mut char_to_assign = last_assigned_char;

                    // All the currently assigned chars of the item carried by the owner
                    let all_currently_assigned_chars: Vec<char> = items_in_backpacks
                        .iter()
                        .filter(|(_, (_, b))| b.owner.id() == collector.id())
                        .map(|(_, (_, b))| b.assigned_char)
                        .collect();

                    // Small monster can only pick up 3 items
                    if all_currently_assigned_chars.len() == MAX_ITEMS_IN_BACKPACK
                        || (small.is_some()
                            && all_currently_assigned_chars.len()
                                == MAX_ITEMS_IN_BACKPACK_FOR_SMALL)
                    {
                        if player_id == collector.id() {
                            game_log
                                .entries
                                .push("You cannot carry anymore!".to_string());
                            failed_pick_upper.push(collector);
                        }
                    } else {
                        // Assign the first "available" char to picked up item
                        // Keep in mind the last assigned char, which is not in the backpack right now
                        // so we can avoid assigning the same char to another item
                        let mut index = 0;
                        while all_currently_assigned_chars.contains(&char_to_assign)
                            || char_to_assign == last_assigned_char
                        {
                            char_to_assign = OPTION_TO_CHAR_MAP[index];
                            index += 1;
                        }
                        last_assigned_char = char_to_assign;

                        // Show appropriate log messages
                        let named_item =
                            ecs_world.get::<&Named>(item).expect("Entity is not Named");

                        if player_id == collector.id() {
                            game_log
                                .entries
                                .push(format!("You pick up the {}", named_item.name));
                        } else if zone.visible_tiles
                            [Zone::get_index_from_xy(&position.x, &position.y)]
                        {
                            // Log NPC  only if visible
                            game_log.entries.push(format!(
                                "The {} picks up the {}",
                                named_collector.name, named_item.name
                            ));
                        }

                        // If needs to be on ground but not starting to rot (usually plants or mushroom)
                        if ecs_world.satisfies::<&ToBeHarvested>(item).unwrap_or(false) {
                            harvested_list.push(item);
                        }

                        // Pick up and keep track of the owner
                        item_owner_list.push((item, collector, char_to_assign, stats.speed));

                        // Check if the item is being stolen from a shop
                        if let Some(owner) =
                            Utils::get_item_owner_by_position(ecs_world, &position.x, &position.y)
                        {
                            let mut shop_owner_query = ecs_world
                                .query_one::<(&mut Hates, &Named)>(owner)
                                .expect("owner must be named and hate");
                            if let Some((hates, named_owner)) = shop_owner_query.get() {
                                if collector.id() == player_id {
                                    game_log.entries.push(format!(
                                        "You stole the {}! The {} gets angry!",
                                        named_item.name, named_owner.name
                                    ));
                                } else if zone.visible_tiles
                                    [Zone::get_index_from_xy(&position.x, &position.y)]
                                {
                                    game_log.entries.push(format!(
                                        "The {} eats the stolen {}! The {} gets angry!",
                                        named_collector.name, named_item.name, named_owner.name
                                    ));
                                }

                                hates.list.insert(collector.id());
                            }
                        }
                    }
                }
            }
        }

        for (item, owner, to_grab, speed) in item_owner_list {
            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsItem>(owner);

            // Remove item from zone and register that now item is in "wants_item" entity backpack
            let _ = ecs_world.exchange_one::<Position, InBackback>(
                item,
                InBackback {
                    owner,
                    assigned_char: to_grab,
                },
            );

            Utils::wait_after_action(ecs_world, owner, speed);
        }

        for entity in failed_pick_upper {
            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsItem>(entity);
        }

        for item in harvested_list {
            // Start to perish
            let _ = ecs_world.exchange_one::<ToBeHarvested, Perishable>(
                item,
                Perishable {
                    rot_counter: STARTING_ROT_COUNTER + Roll::d20(),
                },
            );
        }
    }
}
