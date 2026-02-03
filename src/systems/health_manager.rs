use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, MyTurn, Named, Position},
        health::{Diseased, Hunger},
        player::Player,
    },
    constants::{MAX_DISEASE_TICK_COUNTER, MAX_HUNGER_TICK_COUNTER},
    maps::zone::{DecalType, Zone},
    systems::hunger_check::HungerStatus,
    utils::roll::Roll,
};

pub struct HealthManager {}

/// Checking hunger status
impl HealthManager {
    pub fn run(ecs_world: &mut World) {
        let mut healed_entities: Vec<Entity> = Vec::new();

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
                        match Roll::d6() {
                            1 => {
                                damage.toughness_damage_received += Roll::dice(1, 2);
                                if player_id == diseased_entity.id() {
                                    game_log.entries.push("You cough blood!".to_string());
                                } else {
                                    game_log
                                        .entries
                                        .push(format!("{} coughs blood!", named.name));
                                }

                                zone.decals_tiles.insert(
                                    Zone::get_index_from_xy(&position.x, &position.y),
                                    DecalType::Blood,
                                );
                            }
                            2 => {
                                damage.dexterity_damage_received += Roll::dice(1, 2);
                                if player_id == diseased_entity.id() {
                                    game_log.entries.push("Your muscles stiffen!".to_string());
                                } else {
                                    game_log
                                        .entries
                                        .push(format!("{}'s body is stiffening!", named.name));
                                }
                            }
                            3 => {
                                damage.damage_received += Roll::dice(2, 4);
                                if player_id == diseased_entity.id() {
                                    game_log
                                        .entries
                                        .push("Your head spins and you stumble!".to_string());
                                } else {
                                    game_log.entries.push(format!("{} stumbles!", named.name));
                                }
                            }
                            4 => {
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
                                } else {
                                    game_log
                                        .entries
                                        .push(format!("{} vomits badly!", named.name));
                                }
                            }
                            5 => {
                                if player_id == diseased_entity.id() {
                                    game_log
                                        .entries
                                        .push("You feel dizzy for a moment!".to_string());
                                }
                            }
                            6 => {
                                damage.toughness_damage_received += Roll::dice(1, 2);
                                damage.dexterity_damage_received += Roll::dice(1, 2);
                                if player_id == diseased_entity.id() {
                                    game_log.entries.push(
                                        "You cough blood and your body stiffens!".to_string(),
                                    );
                                } else {
                                    game_log.entries.push(format!(
                                        "{} coughs blood and its body stiffens!",
                                        named.name
                                    ));
                                }
                                zone.decals_tiles.insert(
                                    Zone::get_index_from_xy(&position.x, &position.y),
                                    DecalType::Blood,
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Remove disease from healed entities
        for healed in healed_entities {
            let _ = ecs_world.remove_one::<Diseased>(healed);
        }
    }
}
