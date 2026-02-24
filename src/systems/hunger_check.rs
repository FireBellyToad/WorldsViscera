use std::cmp::{max, min};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{MyTurn, Position, Species, SpeciesEnum},
        health::Hunger,
    },
    constants::MAX_HUNGER_TICK_COUNTER,
    engine::state::GameState,
    maps::zone::{DecalType, Zone},
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

impl HungerStatus {
    pub fn to_string(&self) -> &'static str {
        match *self {
            HungerStatus::Satiated => "3/3",
            HungerStatus::Normal => "2/3",
            HungerStatus::Hungry => "1/3",
            HungerStatus::Starved => "0/3",
        }
    }
}

pub struct HungerCheck {}

/// Checking hunger status
impl HungerCheck {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");

        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut hungry_entities = ecs_world
                .query::<(&mut Hunger, &CombatStats, &Position, &Species)>()
                .with::<&MyTurn>();

            //Log all the hunger checks

            for (hungry_entity, (hunger, stats, position, species)) in &mut hungry_entities {
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
                                game_state
                                    .game_log
                                    .entries
                                    .push("You are starving!".to_string());
                            }
                        }
                        HungerStatus::Starved => {
                            // 33% of chance to be damaged by hunger
                            // Undead feel hunger but are immune to starvation damage
                            if species.value != SpeciesEnum::Undead && Roll::d6() <= 2 {
                                // if can starve and can be damaged, damage the entity
                                if let Ok(mut damage_starving_entity) =
                                    ecs_world.get::<&mut SufferingDamage>(hungry_entity)
                                {
                                    damage_starving_entity.damage_received += 1;

                                    if hungry_entity.id() == player_id {
                                        game_state
                                            .game_log
                                            .entries
                                            .push("Starvation wastes you away!".to_string());
                                    }
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
                                    game_state.game_log.entries.push(
                                        "You ate too much and feel slightly nauseous".to_string(),
                                    );
                                }
                            } else {
                                hunger.tick_counter = MAX_HUNGER_TICK_COUNTER - Roll::dice(3, 10);
                                hunger.current_status = HungerStatus::Normal;
                                zone.decals_tiles.insert(
                                    Zone::get_index_from_xy(&position.x, &position.y),
                                    DecalType::Vomit,
                                );
                                if hungry_entity.id() == player_id {
                                    game_state
                                        .game_log
                                        .entries
                                        .push("You ate too much and vomit!".to_string());
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
                                game_state
                                    .game_log
                                    .entries
                                    .push("You are no longer starved".to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}
