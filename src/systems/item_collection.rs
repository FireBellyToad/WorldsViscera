use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsItem,
        common::{GameLog, Named, Position},
        items::{InBackback, Item, Perishable, ToBeHarvested},
        player::Player,
    },
    constants::{MAX_ITEMS_IN_BACKPACK, OPTION_TO_CHAR_MAP, STARTING_ROT_COUNTER}, utils::roll::Roll,
};

pub struct ItemCollection {}

impl ItemCollection {
    pub fn run(ecs_world: &mut World) {
        let mut item_owner_list: Vec<(Entity, Entity, char)> = Vec::new();
        let mut failed_pick_upper: Vec<Entity> = Vec::new();
        let mut harvested_list: Vec<Entity> = Vec::new();
        let player_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut collectors = ecs_world.query::<&WantsItem>();

            //Items in all backpacks
            let mut items_in_backpacks = ecs_world.query::<(&Item, &InBackback)>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (collector, wants_item) in &mut collectors {
                let mut char_to_assign = OPTION_TO_CHAR_MAP[0];

                // All the currently assigned chars of the item carried by the owner
                let all_currently_assigned_chars: Vec<char> = items_in_backpacks
                    .iter()
                    .filter(|(_e, (_i, b))| b.owner.id() == collector.id())
                    .map(|(_e, (_i, b))| b.assigned_char)
                    .collect();

                let named_owner = ecs_world
                    .get::<&Named>(collector)
                    .expect("Entity is not Named");
                if all_currently_assigned_chars.len() == MAX_ITEMS_IN_BACKPACK {
                    if player_id == collector.id() {
                        game_log
                            .entries
                            .push("You cannot carry anymore!".to_string());
                        failed_pick_upper.push(collector);
                    }
                } else {
                    // Assign the first "available" char to picked up item
                    let mut index = 0;
                    while all_currently_assigned_chars.contains(&char_to_assign) {
                        char_to_assign = OPTION_TO_CHAR_MAP[index];
                        index += 1;
                    }

                    // Show appropriate log messages
                    let named_item = ecs_world
                        .get::<&Named>(wants_item.item)
                        .expect("Entity is not Named");

                    if player_id == collector.id() {
                        game_log
                            .entries
                            .push(format!("You pick up the {}", named_item.name));
                    } else {
                        game_log.entries.push(format!(
                            "{} picks up the {}",
                            named_owner.name, named_item.name
                        ));
                    }

                    // If needs to be on ground but not starting to rot (usually plants or mushroom)
                    if ecs_world.get::<&ToBeHarvested>(wants_item.item).is_ok() {
                        harvested_list.push(wants_item.item);
                    }

                    // Pick up and keep track of the owner
                    item_owner_list.push((wants_item.item, collector, char_to_assign));
                }
            }
        }

        for (item, owner, to_grab) in item_owner_list {
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
