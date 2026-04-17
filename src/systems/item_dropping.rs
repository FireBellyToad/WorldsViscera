use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToDrop,
        combat::CombatStats,
        common::{MyTurn, Named, Position},
        items::{Corpse, Equipped, InBackback},
    },
    engine::state::GameState,
    utils::common::{ItemsInBackpack, Utils},
};

pub struct ItemDropping {}

impl ItemDropping {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let mut item_drop_position_list: Vec<(Entity, Entity, (i32, i32), i32)> = Vec::new();
        let mut item_drop_nothing: Vec<Entity> = Vec::new();
        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to drop items
            let mut droppers_of_item = ecs_world
                .query::<(&WantsToDrop, &CombatStats, &Position)>()
                .with::<&MyTurn>();

            //Log all the drop downs

            for (dropper, (wants_to_drop, stats, drop_position)) in &mut droppers_of_item {
                let mut dropped_item_query = ecs_world
                    .query_one::<(&InBackback, Option<&Equipped>, Option<&Corpse>)>(
                        wants_to_drop.item,
                    )
                    .expect("No entity {:?} found with InBackback component");
                let (_, equipped_opt, corpse_opt) =
                    dropped_item_query.get().expect("Must have Equipped");
                let corpse_text = Utils::get_corpse_string(corpse_opt.is_some());

                if equipped_opt.is_some() {
                    if player_id == dropper.id() {
                        game_state
                            .game_log
                            .add_entry("You cannot drop an equipped item");
                    }

                    item_drop_nothing.push(dropper);
                } else {
                    let named_item = ecs_world
                        .get::<&Named>(wants_to_drop.item)
                        .expect("Entity is not Named");

                    // Drop item and keep track of the drop Position
                    item_drop_position_list.push((
                        wants_to_drop.item,
                        dropper,
                        (drop_position.x, drop_position.y),
                        stats.speed,
                    ));

                    if player_id == dropper.id() {
                        game_state.game_log.add_entry(&format!(
                            "You drop up a {}{}",
                            named_item.name, corpse_text
                        ));
                    } else {
                        let named_dropper = ecs_world
                            .get::<&Named>(dropper)
                            .expect("Entity is not Named");
                        game_state.game_log.add_entry(&format!(
                            "{} drops up a {}{}",
                            named_dropper.name, named_item.name, corpse_text
                        ));
                    }
                }
            }
        }

        for dropper in item_drop_nothing {
            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsToDrop>(dropper);
        }

        for (item, dropper, (drop_x, drop_y), speed) in item_drop_position_list {
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

            Utils::wait_after_action(ecs_world, dropper, speed);
        }
    }

    /// Drop all items of entity
    pub fn drop_all_of(ent: Entity, ecs_world: &mut World, drop_x: i32, drop_y: i32) {
        let items_to_drop: Vec<Entity>;

        //Drop items
        items_to_drop = ecs_world
            .query::<ItemsInBackpack>()
            .iter()
            .filter_map(|(e, (_, in_backpack, ..))| {
                if in_backpack.owner.id() == ent.id() {
                    Some(e)
                } else {
                    None
                }
            })
            .collect();

        for item in items_to_drop {
            // Remove item from back pack Register that now item is in "wants_item" entity backpack
            let _ = ecs_world.exchange_one::<InBackback, Position>(
                item,
                Position {
                    x: drop_x,
                    y: drop_y,
                },
            );

            let _ = ecs_world.remove_one::<Equipped>(item);
        }
    }
}
