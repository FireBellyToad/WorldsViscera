use std::{cmp::max, collections::HashMap};

use hecs::Entity;

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, MyTurn, Named, Position},
        health::{Cured, DiseaseType, Diseased, Hunger},
    },
    constants::{MAX_DISEASE_TICK_COUNTER, VERY_LONG_ACTION_MULTIPLIER},
    engine::state::GameState,
    maps::zone::{DecalType, Zone},
    systems::hunger_check::HungerStatus,
    utils::{common::Utils, roll::Roll},
};

pub struct HealthManager {}

/// Checking hunger status
impl HealthManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let mut healed_entities: Vec<(Entity, DiseaseType, bool)> = Vec::new();
        let mut dizzy_entities_list: Vec<(Entity, i32)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut diseased_entities = ecs_world
                .query::<(
                    &mut Diseased,
                    &CombatStats,
                    &mut SufferingDamage,
                    &mut Hunger,
                    &Named,
                    &Position,
                    Option<&Cured>,
                )>()
                .with::<&MyTurn>();

            //Log all the disease checks

            let zone = game_state
                .current_zone
                .as_mut()
                .expect("must have Some Zone");

            for (diseased_entity, (disease, stats, damage, hunger, named, position, cured_opt)) in
                &mut diseased_entities
            {
                for (disease_type, (tick_counter, is_improving)) in disease.tick_counters.iter_mut()
                {
                    // Heal if cured
                    if let Some(cured) = cured_opt
                        && cured.diseases.iter().any(|d| d == disease_type)
                    {
                        healed_entities.push((diseased_entity, disease_type.clone(), true));

                        // TODO refactor log
                        if player_id == diseased_entity.id() {
                            game_state
                                .game_log
                                .entries
                                .push("You feel better".to_string());
                        }
                    }
                    // When clock is depleted, decrease disease status
                    *tick_counter = max(0, *tick_counter - 1);
                    println!(
                        "Entity {} disease tick_counter {}",
                        diseased_entity.id(),
                        tick_counter
                    );

                    if *tick_counter == 0 {
                        // Reset tick counter randomly
                        *tick_counter = MAX_DISEASE_TICK_COUNTER + Roll::d20();
                        println!(
                            "Entity {} disease tick_counter reset to {}",
                            diseased_entity.id(),
                            tick_counter
                        );

                        // If saving throw is successful, improve health status or heal if already improved
                        if Roll::d20() <= stats.current_toughness {
                            if *is_improving {
                                healed_entities.push((
                                    diseased_entity,
                                    disease_type.clone(),
                                    false,
                                ));
                                // TODO refactor log
                                if player_id == diseased_entity.id() {
                                    game_state
                                        .game_log
                                        .entries
                                        .push("You feel better".to_string());
                                }
                            } else {
                                *is_improving = true;
                            }
                        } else {
                            // if failed, randomize consequences
                            *is_improving = false;
                            match disease_type {
                                DiseaseType::FleshRot => {
                                    // Vomit or cough blood
                                    if Roll::d6() > 3 {
                                        damage.toughness_damage_received += Roll::dice(1, 3);
                                        if player_id == diseased_entity.id() {
                                            if Roll::d6() > 3 {
                                                game_state
                                                    .game_log
                                                    .entries
                                                    .push("You cough blood!".to_string());
                                            } else {
                                                game_state
                                                    .game_log
                                                    .entries
                                                    .push("Your skin peels away!".to_string());
                                            }
                                        } else if zone.visible_tiles
                                            [Zone::get_index_from_xy(&position.x, &position.y)]
                                        {
                                            if Roll::d6() > 3 {
                                                game_state
                                                    .game_log
                                                    .entries
                                                    .push(format!("{} coughs blood!", named.name));
                                            } else {
                                                game_state.game_log.entries.push(format!(
                                                    "{}'s skin peels away!",
                                                    named.name
                                                ));
                                            }
                                        }

                                        zone.decals_tiles.insert(
                                            Zone::get_index_from_xy(&position.x, &position.y),
                                            DecalType::Blood,
                                        );
                                    } else {
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

                                        zone.decals_tiles.insert(
                                            Zone::get_index_from_xy(&position.x, &position.y),
                                            DecalType::Vomit,
                                        );

                                        if player_id == diseased_entity.id() {
                                            game_state
                                                .game_log
                                                .entries
                                                .push("You vomit badly!".to_string());
                                        } else if zone.visible_tiles
                                            [Zone::get_index_from_xy(&position.x, &position.y)]
                                        {
                                            game_state
                                                .game_log
                                                .entries
                                                .push(format!("{} vomits badly!", named.name));
                                        }
                                    }
                                }
                                DiseaseType::Fever => {
                                    // You fumble or lose a turn.
                                    if Roll::d6() > 3 {
                                        damage.damage_received += Roll::dice(2, 4);
                                        if player_id == diseased_entity.id() {
                                            game_state
                                                .game_log
                                                .entries
                                                .push("The fever makes you stumble!".to_string());
                                        } else {
                                            game_state
                                                .game_log
                                                .entries
                                                .push(format!("{} stumbles!", named.name));
                                        }
                                    } else if zone.visible_tiles
                                        [Zone::get_index_from_xy(&position.x, &position.y)]
                                    {
                                        dizzy_entities_list.push((diseased_entity, stats.speed));
                                        if player_id == diseased_entity.id() {
                                            game_state.game_log.entries.push(
                                                "The fever makes you feel dizzy for a moment!"
                                                    .to_string(),
                                            );
                                        }
                                    }
                                }
                                DiseaseType::Calcification => {
                                    damage.dexterity_damage_received += Roll::dice(1, 2);
                                    if player_id == diseased_entity.id() {
                                        if Roll::d6() > 3 {
                                            game_state
                                                .game_log
                                                .entries
                                                .push("Your muscles stiffens!".to_string());
                                        } else {
                                            game_state.game_log.entries.push(
                                                "A calcified patch appears on your skin!"
                                                    .to_string(),
                                            );
                                        }
                                    } else if zone.visible_tiles
                                        [Zone::get_index_from_xy(&position.x, &position.y)]
                                    {
                                        if Roll::d6() > 3 {
                                            game_state
                                                .game_log
                                                .entries
                                                .push(format!("{}'s body stiffens!", named.name));
                                        } else {
                                            game_state.game_log.entries.push(format!(
                                                "A calcified patch appears on {}'s skin!",
                                                named.name
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove disease from healed entities. Check if all diseases are cured
        let mut cured_entities: Vec<(Entity, bool)> = Vec::new();
        for (healed, disease_type, from_cure) in healed_entities {
            if let Ok(mut dis) = ecs_world.get::<&mut Diseased>(healed) {
                dis.tick_counters.remove(&disease_type);
                // Check if all diseases are cured
                if dis.tick_counters.is_empty() {
                    cured_entities.push((healed, from_cure));
                }
            }
        }
        // Remove cure component if cured and is from cure
        for (cured, from_cure) in cured_entities {
            if from_cure {
                let _ = ecs_world.remove::<(Diseased, Cured)>(cured);
            } else {
                let _ = ecs_world.remove_one::<Diseased>(cured);
            }
        }

        // Makes entities waiting for a while
        for (waiter, speed) in dizzy_entities_list {
            Utils::wait_after_action(ecs_world, waiter, speed * VERY_LONG_ACTION_MULTIPLIER);
        }
    }
}
