use hecs::{Entity, World};

use crate::components::{
    actions::WantsToDrop,
    common::{GameLog, Named, Position},
    items::{Equipped, InBackback},
};

pub struct ItemDropping {}

impl ItemDropping {
    pub fn run(ecs_world: &mut World) {
        let mut item_drop_position_list: Vec<(Entity, Entity, (i32, i32), bool)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to drop items
            let mut items_to_drop = ecs_world.query::<&WantsToDrop>();

            //Log all the drop downs
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (dropper, wants_item) in &mut items_to_drop {
                // Show appropriate log messages
                let named_dropper = ecs_world.get::<&Named>(dropper).unwrap();
                let named_item = ecs_world.get::<&Named>(wants_item.item).unwrap();
                let drop: hecs::Ref<'_, Position> = ecs_world.get::<&Position>(dropper).unwrap();
                let is_equipped = ecs_world.get::<&Equipped>(wants_item.item).is_ok();

                // Drop item and keep track of the drop Position
                item_drop_position_list.push((
                    wants_item.item,
                    dropper,
                    (drop.x, drop.y),
                    is_equipped,
                ));

                game_log.entries.push(format!(
                    "{} drops up the {}",
                    named_dropper.name, named_item.name
                ));
            }
        }

        for (item, dropper, (drop_x, drop_y), is_equipped) in item_drop_position_list {
            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsToDrop>(dropper);

            // TODO improve, this action must be longer!
            if is_equipped {
                let _ = ecs_world.remove_one::<Equipped>(item);
            }

            // Remove item from back pack Register that now item is in "wants_item" entity backpack
            let _ = ecs_world.exchange_one::<InBackback, Position>(
                item,
                Position {
                    x: drop_x,
                    y: drop_y,
                },
            );
        }
    }
}
