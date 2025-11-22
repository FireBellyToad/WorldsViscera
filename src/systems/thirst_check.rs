use std::cmp::{max, min};

use hecs::World;

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, MyTurn, Position},
        health::Thirst,
        player::Player,
    },
    constants::MAX_THIRST_TICK_COUNTER,
    maps::zone::{DecalType, Zone},
    utils::roll::Roll,
};

/// Thirst status enum
#[derive(Debug, PartialEq)]
pub enum ThirstStatus {
    Quenched,
    Normal,
    Thirsty,
    Dehydrated,
}

impl ThirstStatus {
    pub fn to_string(&self) -> &'static str {
        match *self {
            ThirstStatus::Quenched => "3/3",
            ThirstStatus::Normal => "2/3",
            ThirstStatus::Thirsty => "1/3",
            ThirstStatus::Dehydrated => "0/3",
        }
    }
}

pub struct ThirstCheck {}

/// Checking thirst status
impl ThirstCheck {
    pub fn run(ecs_world: &mut World) {
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut thirsty_entities = ecs_world
                .query::<(&mut Thirst, &CombatStats, &Position)>()
                .with::<&MyTurn>();

            let player_id = Player::get_entity_id(ecs_world);

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            //Log all the thirst checks
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (thirsty_entity, (thirst, stats, position)) in &mut thirsty_entities {
                // When clock is depleted, decrease thirst status
                // TODO Calculate penalties
                thirst.tick_counter = max(0, thirst.tick_counter - 1);

                if thirst.tick_counter <= MAX_THIRST_TICK_COUNTER && thirst.tick_counter == 0 {
                    match thirst.current_status {
                        ThirstStatus::Quenched => {
                            thirst.tick_counter = MAX_THIRST_TICK_COUNTER;
                            thirst.current_status = ThirstStatus::Normal;
                        }
                        ThirstStatus::Normal => {
                            thirst.tick_counter = MAX_THIRST_TICK_COUNTER;
                            thirst.current_status = ThirstStatus::Thirsty;
                        }
                        ThirstStatus::Thirsty => {
                            thirst.tick_counter = MAX_THIRST_TICK_COUNTER;
                            thirst.current_status = ThirstStatus::Dehydrated;

                            if thirsty_entity.id() == player_id {
                                game_log.entries.push("You are dehydrated!".to_string());
                            }
                        }
                        ThirstStatus::Dehydrated => {
                            // 33% of chance to be damaged by thirst
                            if Roll::d6() <= 2 {
                                // if can starve and be damaged, damage the entity
                                if let Ok(mut damage_starving_entity) =
                                    ecs_world.get::<&mut SufferingDamage>(thirsty_entity)
                                {
                                    damage_starving_entity.damage_received += 1;

                                    if thirsty_entity.id() == player_id {
                                        game_log
                                            .entries
                                            .push("Dehydration wastes you away!".to_string());
                                    }
                                }
                            }
                        }
                    }
                } else if thirst.tick_counter > MAX_THIRST_TICK_COUNTER {
                    // If drinking something, keep delta and increase status
                    // Do not do a double step (FIXME think about it)
                    thirst.tick_counter = min(
                        MAX_THIRST_TICK_COUNTER,
                        thirst.tick_counter - MAX_THIRST_TICK_COUNTER,
                    );
                    match thirst.current_status {
                        ThirstStatus::Quenched => {
                            if Roll::d20() <= stats.current_toughness {
                                thirst.tick_counter = MAX_THIRST_TICK_COUNTER;
                                if thirsty_entity.id() == player_id {
                                    game_log.entries.push(
                                        "You drank too much and feel slightly nauseous".to_string(),
                                    );
                                }
                            } else {
                                //Less severe than being oversatiated...
                                thirst.tick_counter = MAX_THIRST_TICK_COUNTER - Roll::dice(2, 10);
                                thirst.current_status = ThirstStatus::Normal;
                                zone.decals_tiles.insert(
                                    Zone::get_index_from_xy(&position.x, &position.y),
                                    DecalType::Vomit,
                                );
                                if thirsty_entity.id() == player_id {
                                    game_log
                                        .entries
                                        .push("You drank too much and vomit!".to_string());
                                }
                            }
                        }
                        ThirstStatus::Normal => {
                            thirst.current_status = ThirstStatus::Quenched;
                        }
                        ThirstStatus::Thirsty => {
                            thirst.current_status = ThirstStatus::Normal;
                        }
                        ThirstStatus::Dehydrated => {
                            thirst.current_status = ThirstStatus::Thirsty;
                            if thirsty_entity.id() == player_id {
                                game_log
                                    .entries
                                    .push("You are no longer dehydrated".to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}
