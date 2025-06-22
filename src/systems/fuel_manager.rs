use hecs::{Entity, World};

use crate::components::{
    common::{GameLog, Named, Viewshed},
    items::{Fuel, InBackback, Refill, WantsToFuel},
    player::Player,
};

pub struct FuelManager {}

impl FuelManager {
    pub fn check_fuel(ecs_world: &mut World) {
        // List of light producers with fuel
        let mut lighters = ecs_world.query::<&mut Fuel>().without::<&Refill>();

        let player_entity = Player::get_player_entity(ecs_world);

        let mut game_log_query = ecs_world.query::<&mut GameLog>();
        let (_e, game_log) = game_log_query
            .iter()
            .last()
            .expect("Game log is not in hecs::World");

        for (lighter, fuel) in &mut lighters {
            // Log fuel change for lantern used by player
            let entity_in_backpack = ecs_world.get::<&InBackback>(lighter);

            if entity_in_backpack.is_ok() {
                let in_backback = entity_in_backpack.unwrap();
                let named = ecs_world.get::<&Named>(lighter).unwrap();
                // Log messages for fuel status
                if player_entity.id() == in_backback.owner.id() {
                    match fuel.counter {
                        25 => {
                            game_log
                                .entries
                                .push(format!("Your {} is flickering", named.name));
                        }
                        1 => {
                            game_log
                                .entries
                                .push(format!("Your {} goes out", named.name));
                        }
                        _ => {}
                    }

                    //show immediately new vision
                    let mut player_viewshed =
                        ecs_world.get::<&mut Viewshed>(player_entity).unwrap();
                    player_viewshed.must_recalculate = true;
                }
            }

            //If fuel is less then 1, the lighter will not produce light
            if fuel.counter > 0 {
                fuel.counter -= 1;
            }
        }
    }

    pub fn do_refills(ecs_world: &mut World) {
        let mut refillers_and_items_used: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of light producers with fuel
            let mut query = ecs_world.query::<&WantsToFuel>();
            let wants_to_refill_list: Vec<(Entity, &WantsToFuel)> =
                query.iter().filter(|(_e, w)| w.with.is_some()).collect();

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (refiller, wants_to_refill) in wants_to_refill_list {
                let target = wants_to_refill.item;
                let item_used = wants_to_refill.with.unwrap();

                let item_used_fuel = ecs_world.get::<&Fuel>(item_used).unwrap();
                let target_fuel_optional = ecs_world.get::<&mut Fuel>(target);

                if target_fuel_optional.is_ok() {
                    let mut target_fuel = target_fuel_optional.unwrap();

                    // Refill!
                    target_fuel.counter = item_used_fuel.counter;

                    // Show appropriate log messages
                    let named_dropper = ecs_world.get::<&Named>(refiller).unwrap();
                    let named_target = ecs_world.get::<&Named>(target).unwrap();
                    let named_item_used = ecs_world.get::<&Named>(item_used).unwrap();

                    game_log.entries.push(format!(
                        "{} refills the {} with the {}",
                        named_dropper.name, named_target.name, named_item_used.name
                    ));
                } else {
                    game_log
                        .entries
                        .push(format!("This item cannot be refilled!"));
                }

                // Remember refiller and what has been used to cleanup later
                refillers_and_items_used.push((refiller, item_used));
            }
        }

        //Cleanup
        for (refiller, item_used) in refillers_and_items_used {
            let _ = ecs_world.remove_one::<WantsToFuel>(refiller);
            let _ = ecs_world.despawn(item_used);
        }
    }
}
