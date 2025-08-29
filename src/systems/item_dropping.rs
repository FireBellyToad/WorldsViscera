use hecs::{Entity, World};

use crate::components::{
    actions::WantsToDrop,
    common::{GameLog, Named, Position},
    items::{Equipped, InBackback},
    player::Player,
};

pub struct ItemDropping {}

impl ItemDropping {
    pub fn run(ecs_world: &mut World) {
        let mut item_drop_position_list: Vec<(Entity, Entity, (i32, i32))> = Vec::new();
        let mut item_drop_nothing: Vec<Entity> = Vec::new();

        let player_id = Player::get_entity_id(ecs_world);

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
                let is_equipped = ecs_world.get::<&Equipped>(wants_item.item).is_ok();

                if is_equipped {
                    if player_id == dropper.id() {
                        game_log
                            .entries
                            .push(format!("You cannot drop an equipped item"));
                    }

                    item_drop_nothing.push(dropper);
                } else {
                    let named_item = ecs_world.get::<&Named>(wants_item.item).unwrap();
                    let drop: hecs::Ref<'_, Position> =
                        ecs_world.get::<&Position>(dropper).unwrap();

                    // Drop item and keep track of the drop Position
                    item_drop_position_list.push((wants_item.item, dropper, (drop.x, drop.y)));

                    if player_id == dropper.id() {
                        game_log
                            .entries
                            .push(format!("You drop up the {}", named_item.name));
                    } else {
                        let named_dropper = ecs_world.get::<&Named>(dropper).unwrap();
                        game_log.entries.push(format!(
                            "{} drops up a {}",
                            named_dropper.name, named_item.name
                        ));
                    }
                }
            }
        }

        for dropper in item_drop_nothing {
            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsToDrop>(dropper);
        }

        for (item, dropper, (drop_x, drop_y)) in item_drop_position_list {
            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsToDrop>(dropper);

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
