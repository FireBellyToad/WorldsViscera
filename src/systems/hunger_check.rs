use std::cmp::{max, min};

use hecs::World;

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::GameLog,
        health::Hunger,
        player::Player,
    },
    constants::MAX_HUNGER_TICK_COUNTER,
    utils::roll::Roll,
};

/// Hunger status enum
#[derive(Debug, PartialEq)]
pub enum HungerStatus {
    Satiated,
    Normal,
    Hungry,
    Starved,
}

pub struct HungerCheck {}

/// Checking hunger status
impl HungerCheck {
    pub fn run(ecs_world: &mut World) {
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut hungry_entities = ecs_world.query::<(&mut Hunger, &CombatStats)>();

            let player_id = Player::get_player_id(ecs_world);

            //Log all the hunger checks
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (hungry_entity, (hunger, stats)) in &mut hungry_entities {
                // When clock is depleted, decrease fed status
                // TODO Calculate penalties
                hunger.tick_counter = max(0, hunger.tick_counter - 1);

                if hunger.tick_counter <= MAX_HUNGER_TICK_COUNTER && hunger.tick_counter == 0 {
                    match hunger.current_status {
                        HungerStatus::Satiated => {
                            hunger.tick_counter = MAX_HUNGER_TICK_COUNTER;
                            hunger.current_status = HungerStatus::Normal;
                        }
                        HungerStatus::Normal => {
                            hunger.tick_counter = MAX_HUNGER_TICK_COUNTER;
                            hunger.current_status = HungerStatus::Hungry;
                        }
                        HungerStatus::Hungry => {
                            hunger.tick_counter = MAX_HUNGER_TICK_COUNTER;
                            hunger.current_status = HungerStatus::Starved;

                            if hungry_entity.id() == player_id {
                                game_log.entries.push(format!("You are starving!"));
                            }
                        }
                        HungerStatus::Starved => {
                            // 33% of chance to be damaged by hunger
                            if Roll::d6() <= 2 {
                                let damage_starving_entity =
                                    ecs_world.get::<&mut SufferingDamage>(hungry_entity);

                                // if can starve, damage the entity
                                if damage_starving_entity.is_ok() {
                                    damage_starving_entity.unwrap().damage_received += 1;
                                    game_log
                                        .entries
                                        .push(format!("Starvation wastes you away!"));
                                }
                            }
                        }
                    }
                } else if hunger.tick_counter > MAX_HUNGER_TICK_COUNTER {
                    // If eating something, keep delta and increase status
                    // Do not do a double step (FIXME think about it)
                    hunger.tick_counter = min(
                        MAX_HUNGER_TICK_COUNTER,
                        hunger.tick_counter - MAX_HUNGER_TICK_COUNTER,
                    );
                    match hunger.current_status {
                        HungerStatus::Satiated => {
                            if Roll::d20() <= stats.current_toughness {
                                hunger.tick_counter = MAX_HUNGER_TICK_COUNTER;
                                if hungry_entity.id() == player_id {
                                    game_log.entries.push(format!(
                                        "You ate too much and feel slightly nauseous"
                                    ));
                                }
                            } else {
                                hunger.tick_counter = MAX_HUNGER_TICK_COUNTER - Roll::dice(3, 10);
                                hunger.current_status = HungerStatus::Normal;
                                if hungry_entity.id() == player_id {
                                    game_log
                                        .entries
                                        .push(format!("You ate too much and vomit!"));
                                }
                            }
                        }
                        HungerStatus::Normal => {
                            hunger.current_status = HungerStatus::Satiated;
                        }
                        HungerStatus::Hungry => {
                            hunger.current_status = HungerStatus::Normal;
                        }
                        HungerStatus::Starved => {
                            hunger.current_status = HungerStatus::Hungry;
                            if hungry_entity.id() == player_id {
                                game_log.entries.push(format!("You are no longer starved"));
                            }
                        }
                    }
                }
            }
        }
    }
}
