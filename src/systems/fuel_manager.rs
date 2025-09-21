use hecs::{Entity, World};

use crate::components::{
    actions::WantsToFuel,
    common::{GameLog, Named, Viewshed},
    items::{InBackback, MustBeFueled, Refiller, TurnedOn},
    player::Player,
};

pub struct FuelManager {}

impl FuelManager {
    pub fn check_fuel(ecs_world: &mut World) {
        // List of light producers that use fuel
        let mut turned_on_lighters = ecs_world
            .query::<(&mut MustBeFueled, &Named, Option<&InBackback>)>()
            .with::<&TurnedOn>()
            .without::<&Refiller>();

        let player_entity = Player::get_entity(ecs_world);

        let mut game_log_query = ecs_world.query::<&mut GameLog>();
        let (_, game_log) = game_log_query
            .iter()
            .last()
            .expect("Game log is not in hecs::World");

        for (_, (fuel, named, entity_in_backpack)) in &mut turned_on_lighters {
            
            // Log fuel change for lantern used by player
            if let Some(in_backback) = entity_in_backpack {
                // Log messages for fuel status
                if player_entity.id() == in_backback.owner.id() {
                    match fuel.fuel_counter {
                        30 => {
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
                    Player::force_view_recalculation(ecs_world);
                }
            }

            //If fuel is less then 1, the lighter will not produce light
            if fuel.fuel_counter > 0 {
                fuel.fuel_counter -= 1;
            }
        }
    }

    pub fn do_refills(ecs_world: &mut World) {
        let mut refillers_and_items_used: Vec<(Entity, Entity)> = Vec::new();
        let player_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            // List of light producers with fuel
            let mut query = ecs_world.query::<&WantsToFuel>();
            let wants_to_refill_list: Vec<(Entity, &WantsToFuel)> =
                query.iter().filter(|(_, w)| w.item.is_some()).collect();

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (refiller, wants_to_refill) in wants_to_refill_list {
                let target = wants_to_refill.item.expect("item to refill must not be None");
                let item_used = wants_to_refill.with;

                let item_used_fuel = ecs_world
                    .get::<&MustBeFueled>(item_used)
                    .expect("Entity has not MustBeFueled");
                let target_fuel_optional = ecs_world.get::<&mut MustBeFueled>(target);

                match target_fuel_optional {
                    Ok(mut target_fuel) => {
                        // Refill!
                        target_fuel.fuel_counter = item_used_fuel.fuel_counter;

                        // Show appropriate log messages
                        let named_dropper = ecs_world
                            .get::<&Named>(refiller)
                            .expect("Entity is not Named");
                        let named_target = ecs_world
                            .get::<&Named>(target)
                            .expect("Entity is not Named");
                        let named_item_used = ecs_world
                            .get::<&Named>(item_used)
                            .expect("Entity is not Named");

                        game_log.entries.push(format!(
                            "{} refills the {} with the {}",
                            named_dropper.name, named_target.name, named_item_used.name
                        ));
                    }
                    Err(_) => {
                        game_log
                            .entries
                            .push("This item cannot be refilled!".to_string());
                    }
                }

                // Remember refiller and what has been used to cleanup later
                refillers_and_items_used.push((refiller, item_used));
            }
        }

        //Cleanup
        for (refiller, item_used) in refillers_and_items_used {
            let _ = ecs_world.remove_one::<WantsToFuel>(refiller);
            let _ = ecs_world.despawn(item_used);

            if player_id == refiller.id() {
                Player::wait_after_action(ecs_world);
            }
        }
    }
}
