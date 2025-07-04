use hecs::{Entity, World};

use crate::{
    components::{
        common::{GameLog, Named, Position},
        items::{InBackback, Item, WantsItem},
    },
    constants::{MAX_ITEMS_IN_BACKPACK, OPTION_TO_CHAR_MAP},
};

pub struct ItemCollection {}

impl ItemCollection {
    pub fn run(ecs_world: &mut World) {
        let mut item_owner_list: Vec<(Entity, Entity, char)> = Vec::new();
        let mut failed_pick_upper: Vec<Entity> = Vec::new();

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

                let named_owner = ecs_world.get::<&Named>(collector).unwrap();
                if all_currently_assigned_chars.len() == MAX_ITEMS_IN_BACKPACK {
                    game_log
                        .entries
                        .push(format!("{} cannot carry anymore!", named_owner.name));
                    failed_pick_upper.push(collector);
                } else {
                    // Assign the first "available" char to picked up item
                    let mut index = 0;
                    while all_currently_assigned_chars.contains(&char_to_assign) {
                        char_to_assign = OPTION_TO_CHAR_MAP[index];
                        index += 1;
                    }

                    // Show appropriate log messages
                    let named_item = ecs_world.get::<&Named>(wants_item.item).unwrap();

                    game_log.entries.push(format!(
                        "{} picks up the {}",
                        named_owner.name, named_item.name
                    ));

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
    }
}
