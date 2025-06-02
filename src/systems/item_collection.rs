
use hecs::{Entity, World};

use crate::components::{
    common::{GameLog, Named, Position},
    items::{InBackback, WantsItem},
};

pub struct ItemCollection {}

impl ItemCollection {
    pub fn run(ecs_world: &mut World) {

        let mut item_owner_list: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut collectors = ecs_world.query::<&WantsItem>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (collector, wants_item) in &mut collectors {
                // Pick up and keep track of the owner
                item_owner_list.push((wants_item.item, collector));

                // Show appropriate log messages
                let named_owner = ecs_world.get::<&Named>(collector).unwrap();
                let named_item = ecs_world.get::<&Named>(wants_item.item).unwrap();

                game_log.entries.push(format!(
                    "{} picks up the {}",
                    named_owner.name, named_item.name
                ));
            }
        }

        for (item, owner) in item_owner_list {
            // Remove item from map
            let _ = ecs_world.remove_one::<Position>(item);

            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsItem>(owner);

            // Register that now item is in "wants_item" entity backpack
            let _ = ecs_world.insert_one(item, InBackback { owner });
        }
    }
}
