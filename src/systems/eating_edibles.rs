use std::collections::{HashMap, hash_map::Entry};

use hecs::Entity;

use crate::{
    components::{
        actions::WantsToEat,
        combat::{CombatStats, SufferingDamage},
        common::{Hates, Named, Position},
        health::{DiseaseType, Diseased, Hunger},
        items::{Deadly, Edible, Poisonous, Rotten},
        monster::DiseaseBearer,
    },
    constants::MAX_DISEASE_TICK_COUNTER,
    engine::state::GameState,
    maps::zone::{DecalType, Zone},
    systems::hunger_check::HungerStatus,
    utils::{common::Utils, roll::Roll},
};

pub struct EatingEdibles {}

impl EatingEdibles {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let mut eater_cleanup_list: Vec<Entity> = Vec::new();
        let mut eaten_eater_list: Vec<(Entity, Entity, i32)> = Vec::new();
        let mut killed_list: Vec<Entity> = Vec::new();
        let mut infected_list: Vec<(Entity, DiseaseType)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut eaters =
                ecs_world.query::<(&WantsToEat, &CombatStats, &mut Hunger, &Position, &Named)>();

            let zone = game_state
                .current_zone
                .as_mut()
                .expect("must have Some Zone");

            //Log all the eating

            for (eater, (wants_to_eat, stats, hunger, position, named_eater)) in &mut eaters {
                let possible_edible = ecs_world.get::<&Edible>(wants_to_eat.item);

                // Keep track of the eater
                if let Ok(edible_nutrition) = possible_edible {
                    // Eat!
                    eaten_eater_list.push((wants_to_eat.item, eater, stats.speed));

                    // Show appropriate log messages
                    let named_edible = ecs_world
                        .get::<&Named>(wants_to_eat.item)
                        .expect("Entity is not Named");
                    if eater.id() == player_id {
                        game_state
                            .game_log
                            .entries
                            .push(format!("You ate a {}", named_edible.name));
                    } else if zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)]
                    {
                        // Log NPC infighting only if visible
                        game_state
                            .game_log
                            .entries
                            .push(format!("{} ate a {}", named_eater.name, named_edible.name));
                    }

                    if ecs_world.get::<&Deadly>(wants_to_eat.item).is_ok() {
                        if eater.id() == player_id {
                            game_state.game_log.entries.push(
                                "You ate a deadly poisonous food! You agonize and die".to_string(),
                            );
                        }
                        killed_list.push(eater);
                        continue;
                    }

                    // inflict disease of diseased corpse (without saving throw)
                    if let Ok(dis_bear_some) = ecs_world.get::<&DiseaseBearer>(wants_to_eat.item) {
                        let disease_type = dis_bear_some.disease_type.clone();
                        // If the target is already infected, worsen its status
                        if let Ok(mut dis) = ecs_world.get::<&mut Diseased>(eater) {
                            // If the target is already infected, worsen its status
                            match dis.tick_counters.entry(disease_type) {
                                Entry::Occupied(mut entry) => {
                                    //worsen its status
                                    entry.insert((0, false));
                                }
                                Entry::Vacant(entry) => {
                                    // Infect the healthy target otherwise
                                    entry.insert((MAX_DISEASE_TICK_COUNTER + Roll::d20(), false));
                                }
                            }
                        } else {
                            // Infect the healthy target otherwise
                            infected_list.push((eater, disease_type));
                            if player_id == eater.id() {
                                game_state
                                    .game_log
                                    .entries
                                    .push("You start to feel ill...".to_string());
                            }
                        }
                    }

                    let is_poisonous = ecs_world
                        .satisfies::<&Poisonous>(wants_to_eat.item)
                        .unwrap_or(false);
                    let is_rotten = ecs_world
                        .satisfies::<&Rotten>(wants_to_eat.item)
                        .unwrap_or(false);
                    let is_unsavoury = is_poisonous || is_rotten;
                    if is_unsavoury {
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
                            if is_rotten {
                                game_state
                                    .game_log
                                    .entries
                                    .push("You ate rotten food! You vomit!".to_string());
                            } else if is_poisonous {
                                game_state
                                    .game_log
                                    .entries
                                    .push("You ate poisonous food! You vomit!".to_string());
                            }
                        } else if zone.visible_tiles
                            [Zone::get_index_from_xy(&position.x, &position.y)]
                        {
                            // Log NPC infighting only if visible
                            game_state
                                .game_log
                                .entries
                                .push(format!("The {} vomits!", named_eater.name));
                        }

                        zone.decals_tiles.insert(
                            Zone::get_index_from_xy(&position.x, &position.y),
                            DecalType::Vomit,
                        );
                    } else {
                        hunger.tick_counter += Roll::dice(
                            edible_nutrition.nutrition_dice_number,
                            edible_nutrition.nutrition_dice_size,
                        ) * 3;
                    }

                    // Check if the item is being stolen from a shop
                    if let Ok(item_pos) = ecs_world.get::<&Position>(wants_to_eat.item)
                        && let Some(owner) =
                            Utils::get_item_owner_by_position(ecs_world, &item_pos.x, &item_pos.y)
                    {
                        let mut shop_owner_query = ecs_world
                            .query_one::<(&mut Hates, &Named)>(owner)
                            .expect("owner must be named and hate");
                        if let Some((hates, named_owner)) = shop_owner_query.get() {
                            if eater.id() == player_id {
                                game_state.game_log.entries.push(format!(
                                    "You eat the stolen {}! The {} gets angry!",
                                    named_edible.name, named_owner.name
                                ));
                            } else if zone.visible_tiles
                                [Zone::get_index_from_xy(&item_pos.x, &item_pos.y)]
                            {
                                game_state.game_log.entries.push(format!(
                                    "The {} eats the stolen {}! The {} gets angry!",
                                    named_eater.name, named_edible.name, named_owner.name
                                ));
                            }

                            hates.list.insert(eater.id());
                        }
                    }
                } else {
                    if eater.id() == player_id {
                        game_state
                            .game_log
                            .entries
                            .push("You can't eat that!".to_string());
                    }
                    eater_cleanup_list.push(eater);
                }
            }
        }

        for (eaten, eater, speed) in eaten_eater_list {
            // Despawn item from World
            let _ = ecs_world.despawn(eaten);
            // Remove owner's will to eat
            let _ = ecs_world.remove_one::<WantsToEat>(eater);
            println!(
                "Entity id {} eats!!---------------------------------",
                eater.id()
            );

            Utils::wait_after_action(ecs_world, eater, speed);
        }

        for to_clean in eater_cleanup_list {
            // Remove owner's will to eat
            let _ = ecs_world.remove_one::<WantsToEat>(to_clean);
        }

        for killed in killed_list {
            let mut damage = ecs_world
                .get::<&mut SufferingDamage>(killed)
                .expect("Entity has no SufferingDamage");
            let stats = ecs_world
                .get::<&mut CombatStats>(killed)
                .expect("Entity has no CombatStats");
            damage.damage_received = stats.current_stamina + stats.current_toughness;
        }

        // Infect the infected
        for (infected, disease_type) in infected_list {
            // Infect the healthy target otherwise
            let mut tick_counters = HashMap::new();
            tick_counters.insert(
                disease_type,
                (MAX_DISEASE_TICK_COUNTER + Roll::d20(), false),
            );
            let _ = ecs_world.insert_one(infected, Diseased { tick_counters });
        }
    }
}
