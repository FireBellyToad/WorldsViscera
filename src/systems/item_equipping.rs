use hecs::{Entity, World};

use crate::components::{
    actions::WantsToEquip,
    common::{GameLog, Named},
    items::{BodyLocation, Equipped},
};

pub struct ItemEquipping {}

impl ItemEquipping {
    pub fn run(ecs_world: &mut World) {
        let mut item_unequip_list: Vec<(Entity, BodyLocation, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to equip items
            let mut items_to_equip = ecs_world.query::<&WantsToEquip>();

            //Log all the drop downs
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (equipper, wants_item) in &mut items_to_equip {
                // Show appropriate log messages
                let named_dropper = ecs_world.get::<&Named>(equipper).unwrap();
                let named_item = ecs_world.get::<&Named>(wants_item.item).unwrap();

                //TODO check if wants_item.body_location is already taken?

                // Drop item and keep track of the drop Position
                item_unequip_list.push((wants_item.item, wants_item.body_location.clone(), equipper));

                game_log.entries.push(format!(
                    "{} equips up the {}",
                    named_dropper.name, named_item.name
                ));
            }
        }

        for (item, body_location, equipper) in item_unequip_list {
            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsToEquip>(equipper);

            // Equip at specified location
            let _ = ecs_world.insert_one(item, Equipped{
                owner: equipper,
                body_location,
            });
        }
    }
}
