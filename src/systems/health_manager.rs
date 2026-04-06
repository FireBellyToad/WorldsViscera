use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, Immunity, ImmunityTypeEnum, MyTurn, Named, Position},
        health::{Cured, DiseaseType, Diseased, Hunger, Stunned},
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

        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");

        HealthManager::handle_diseases(ecs_world, player_id, zone, &mut game_state.game_log);
        HealthManager::handle_stun(ecs_world, player_id, &mut game_state.game_log);
    }

    pub fn handle_diseases(
        ecs_world: &mut World,
        player_id: u32,
        zone: &mut Zone,
        game_log: &mut GameLog,
    ) {
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

            for (diseased_entity, (disease, stats, damage, hunger, named, position, cured_opt)) in
                &mut diseased_entities
            {
                for (disease_type, (tick_counter, is_improving)) in disease.tick_counters.iter_mut()
                {
                    // Heal if cured
                    if let Some(cured) = cured_opt
                        && cured.diseases.iter().any(|d| d == disease_type)
                    {
                        healed_entities.push((diseased_entity, *disease_type, true));

                        // TODO refactor log
                        if player_id == diseased_entity.id() {
                            game_log.add_entry("You feel better");
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
                                healed_entities.push((diseased_entity, *disease_type, false));
                                // TODO refactor log
                                if player_id == diseased_entity.id() {
                                    game_log.add_entry("You feel better");
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
                                                game_log.add_entry("You cough blood!");
                                            } else {
                                                game_log.add_entry("Your skin peels away!");
                                            }
                                        } else if zone.visible_tiles
                                            [Zone::get_index_from_xy(&position.x, &position.y)]
                                        {
                                            if Roll::d6() > 3 {
                                                game_log
                                                    .entries
                                                    .push(format!("{} coughs blood!", named.name));
                                            } else {
                                                game_log.add_entry(&format!(
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
                                            game_log.add_entry("You vomit badly!");
                                        } else if zone.visible_tiles
                                            [Zone::get_index_from_xy(&position.x, &position.y)]
                                        {
                                            game_log
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
                                            game_log.add_entry("The fever makes you stumble!");
                                        } else {
                                            game_log
                                                .entries
                                                .push(format!("{} stumbles!", named.name));
                                        }
                                    } else if zone.visible_tiles
                                        [Zone::get_index_from_xy(&position.x, &position.y)]
                                    {
                                        dizzy_entities_list.push((diseased_entity, stats.speed));
                                        if player_id == diseased_entity.id() {
                                            game_log.add_entry(
                                                "The fever makes you feel dizzy for a moment!",
                                            );
                                        }
                                    }
                                }
                                DiseaseType::Calcification => {
                                    damage.dexterity_damage_received += Roll::dice(1, 2);
                                    if player_id == diseased_entity.id() {
                                        if Roll::d6() > 3 {
                                            game_log.add_entry("Your muscles stiffens!");
                                        } else {
                                            game_log.add_entry(
                                                "A calcified patch appears on your skin!",
                                            );
                                        }
                                    } else if zone.visible_tiles
                                        [Zone::get_index_from_xy(&position.x, &position.y)]
                                    {
                                        if Roll::d6() > 3 {
                                            game_log
                                                .entries
                                                .push(format!("{}'s body stiffens!", named.name));
                                        } else {
                                            game_log.add_entry(&format!(
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
        let mut gained_immunity: Vec<(Entity, ImmunityTypeEnum)> = Vec::new();
        for (healed, disease_type, from_cure) in healed_entities {
            if let Ok(mut dis) = ecs_world.get::<&mut Diseased>(healed) {
                dis.tick_counters.remove(&disease_type);

                // 50% of the time, grant immunity to the healed disease
                if Roll::d6() > 4 {
                    gained_immunity.push((healed, ImmunityTypeEnum::Disease(disease_type)));
                }

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

        // Add immunity to entities that gained it
        for (healed, immunity_type) in gained_immunity {
            if let Ok(mut immunity) = ecs_world.get::<&mut Immunity>(healed) {
                immunity
                    .to
                    .entry(immunity_type)
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
            } else {
                panic!("Immunity component Missing!!!")
            }
        }

        // Makes entities waiting for a while
        for (waiter, speed) in dizzy_entities_list {
            Utils::wait_after_action(ecs_world, waiter, speed * VERY_LONG_ACTION_MULTIPLIER);
        }
    }

    pub fn handle_stun(ecs_world: &mut World, player_id: u32, game_log: &mut GameLog) {
        let mut unstunned_entities: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of perishable entities
            let mut stunned = ecs_world.query::<&mut Stunned>().with::<&MyTurn>();

            for (entity, stunned) in &mut stunned {
                if stunned.tick_counter > 0 {
                    stunned.tick_counter -= 1;
                    println!(
                        "entity {} stunned.tick_counter {}",
                        entity.id(),
                        stunned.tick_counter
                    );
                } else {
                    if entity.id() == player_id {
                        game_log.add_entry("You are not stunned anymore");
                    }
                    unstunned_entities.push(entity);
                }
            }
        }

        // Despawn completely rotted edibles
        for entity in unstunned_entities {
            let _ = ecs_world.remove_one::<Stunned>(entity);
        }
    }
}
