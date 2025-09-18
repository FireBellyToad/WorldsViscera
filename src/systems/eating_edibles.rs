use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToEat,
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, Named, Position},
        health::Hunger,
        items::{Deadly, Edible, Unsavoury},
        player::Player,
    },
    maps::zone::{DecalType, Zone},
    systems::hunger_check::HungerStatus,
    utils::roll::Roll,
};

pub struct EatingEdibles {}

impl EatingEdibles {
    pub fn run(ecs_world: &mut World) {
        let mut eater_cleanup_list: Vec<Entity> = Vec::new();
        let mut eaten_eater_list: Vec<(Entity, Entity)> = Vec::new();
        let mut killed_list: Vec<Entity> = Vec::new();
        let player_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut eaters = ecs_world.query::<(&WantsToEat, &mut Hunger, &Position)>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (eater, (wants_to_eat, hunger, position)) in &mut eaters {
                let possible_edible = ecs_world.get::<&Edible>(wants_to_eat.item);

                // Keep track of the eater
                if let Ok(edible_nutrition) = possible_edible {
                    // Eat!
                    eaten_eater_list.push((wants_to_eat.item, eater));

                    // Show appropriate log messages
                    let named_edible = ecs_world.get::<&Named>(wants_to_eat.item).expect("Entity is not Named");
                    if eater.id() == player_id {
                        game_log
                            .entries
                            .push(format!("You ate a {}", named_edible.name));
                    } else {
                        let named_eater = ecs_world.get::<&Named>(eater).expect("Entity is not Named");

                        game_log
                            .entries
                            .push(format!("{} ate a {}", named_eater.name, named_edible.name));
                    }

                    if ecs_world.get::<&Deadly>(wants_to_eat.item).is_ok() {
                        if eater.id() == player_id {
                            game_log.entries.push(
                                "You ate a deadly poisonous food! You agonize and die".
                            to_string());
                        }
                        killed_list.push(eater);
                        continue;
                    }

                    // Is it unsavoury? Then vomit badly
                    if let Ok(unsavoury_component) = ecs_world.get::<&Unsavoury>(wants_to_eat.item)
                    {
                        hunger.tick_counter -= Roll::dice(3, 10);
                        match hunger.current_status {
                            HungerStatus::Satiated => {
                                hunger.current_status = HungerStatus::Normal;
                            }
                            HungerStatus::Normal => {
                                hunger.current_status = HungerStatus::Hungry;
                            }
                            HungerStatus::Hungry => {
                                hunger.current_status = HungerStatus::Starved;
                            }
                            HungerStatus::Starved => {}
                        }

                        if eater.id() == player_id {
                            game_log.entries.push(format!(
                                "You ate {} food! You vomit!",
                                unsavoury_component.game_log
                            ));
                        }

                        zone.decals_tiles.insert(
                            Zone::get_index_from_xy(position.x, position.y),
                            DecalType::Vomit,
                        );
                    } else {
                        hunger.tick_counter += Roll::dice(
                            edible_nutrition.nutrition_dice_number,
                            edible_nutrition.nutrition_dice_size,
                        );
                    }
                } else {
                    if eater.id() == player_id {
                        game_log.entries.push("You can't eat that!".to_string());
                    }
                    eater_cleanup_list.push(eater);
                }
            }
        }

        for (eaten, eater) in eaten_eater_list {
            // Despawn item from World
            let _ = ecs_world.despawn(eaten);
            // Remove owner's will to eat
            let _ = ecs_world.remove_one::<WantsToEat>(eater);
            if player_id == eater.id() {
                Player::wait_after_action(ecs_world);
            }
        }

        for to_clean in eater_cleanup_list {
            // Remove owner's will to eat
            let _ = ecs_world.remove_one::<WantsToEat>(to_clean);
        }

        for killed in killed_list {
            let mut damage = ecs_world.get::<&mut SufferingDamage>(killed).expect("Entity has no SufferingDamage");
            let stats = ecs_world.get::<&mut CombatStats>(killed).expect("Entity has no CombatStats");
            damage.damage_received = stats.current_stamina + stats.current_toughness;
        }
    }
}
