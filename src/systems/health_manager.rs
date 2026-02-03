use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, MyTurn, Named, Position},
        health::{DiseaseType, Diseased, Hunger},
        player::Player,
    },
    constants::MAX_DISEASE_TICK_COUNTER,
    maps::zone::{DecalType, Zone},
    systems::hunger_check::HungerStatus,
    utils::{common::Utils, roll::Roll},
};

pub struct HealthManager {}

/// Checking hunger status
impl HealthManager {
    pub fn run(ecs_world: &mut World) {
        let mut healed_entities: Vec<Entity> = Vec::new();
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
                )>()
                .with::<&MyTurn>();

            let player_id = Player::get_entity_id(ecs_world);

            //Log all the disease checks
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            for (diseased_entity, (disease, stats, damage, hunger, named, position)) in
                &mut diseased_entities
            {
                // When clock is depleted, decrease disease status
                disease.tick_counter = max(0, disease.tick_counter - 1);
                println!(
                    "Entity {} disease tick_counter {}",
                    diseased_entity.id(),
                    disease.tick_counter
                );

                if disease.tick_counter == 0 {
                    // Reset tick counter randomly
                    disease.tick_counter = MAX_DISEASE_TICK_COUNTER + Roll::d20();
                    println!(
                        "Entity {} disease tick_counter reset to {}",
                        diseased_entity.id(),
                        disease.tick_counter
                    );

                    // If saving throw is successful, improve health status or heal if already improved
                    if Roll::d20() <= stats.current_toughness {
                        if disease.is_improving {
                            healed_entities.push(diseased_entity);
                            println!("Entity {} is healed", diseased_entity.id())
                        } else {
                            disease.is_improving = true;
                            println!("Entity {} is_improving", diseased_entity.id())
                        }
                    } else {
                        // if failed, randomize consequences
                        disease.is_improving = false;
                        match disease.disease_type {
                            DiseaseType::FleshRot => {
                                // Vomit or cough blood
                                if Roll::d6() > 3 {
                                    damage.toughness_damage_received += Roll::dice(1, 3);
                                    if player_id == diseased_entity.id() {
                                        if Roll::d6() > 3 {
                                            game_log.entries.push("You cough blood!".to_string());
                                        } else {
                                            game_log
                                                .entries
                                                .push("Your skin peels away!".to_string());
                                        }
                                    } else if zone.visible_tiles
                                        [Zone::get_index_from_xy(&position.x, &position.y)]
                                    {
                                        if Roll::d6() > 3 {
                                            game_log
                                                .entries
                                                .push(format!("{} coughs blood!", named.name));
                                        } else {
                                            game_log
                                                .entries
                                                .push(format!("{}'s skin peels away!", named.name));
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
                                        game_log.entries.push("You vomit badly!".to_string());
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
                                        game_log
                                            .entries
                                            .push("Your head spins and you stumble!".to_string());
                                    } else {
                                        game_log.entries.push(format!("{} stumbles!", named.name));
                                    }
                                } else if zone.visible_tiles
                                    [Zone::get_index_from_xy(&position.x, &position.y)]
                                {
                                    dizzy_entities_list.push((diseased_entity, stats.speed));
                                    if player_id == diseased_entity.id() {
                                        game_log
                                            .entries
                                            .push("You feel dizzy for a moment!".to_string());
                                    }
                                }
                            }
                            DiseaseType::Calcification => {
                                damage.dexterity_damage_received += Roll::dice(1, 2);
                                if player_id == diseased_entity.id() {
                                    if Roll::d6() > 3 {
                                        game_log.entries.push("Your muscles stiffens!".to_string());
                                    } else {
                                        game_log.entries.push(
                                            "A calcified patch appears on your skin!".to_string(),
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
                                        game_log.entries.push(format!(
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

        // Remove disease from healed entities
        for healed in healed_entities {
            let _ = ecs_world.remove_one::<Diseased>(healed);
        }
        // TODO make sure this works
        // for (waiter, speed) in dizzy_entities_list {
        //     Utils::wait_after_action(ecs_world, waiter, speed);
        // }
    }
}
