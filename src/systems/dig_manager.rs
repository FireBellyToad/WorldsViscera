use std::borrow::Cow;

use hecs::Entity;

use crate::{
    components::{
        actions::{WantsToDig, WantsToEat},
        combat::{CombatStats, InflictsDamage},
        common::{DigProductEnum, Diggable, MyTurn, Named, Position},
    },
    engine::state::GameState,
    maps::zone::{TileType, Zone},
    spawning::spawner::Spawn,
    utils::{common::Utils, roll::Roll},
};

pub struct DigManager {}

impl DigManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let mut diggers_list: Vec<(Entity, i32)> = Vec::new();
        let mut digged_list: Vec<Entity> = Vec::new();
        let mut produced_list: Vec<(i32, i32, DigProductEnum)> = Vec::new();
        let mut wants_to_eat_list: Vec<(Entity, i32)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to dig
            let mut collectors = ecs_world
                .query::<(&WantsToDig, &CombatStats, &Named)>()
                .with::<&MyTurn>();

            let zone = game_state
                .current_zone
                .as_mut()
                .expect("must have Some Zone");

            //Log all the pick ups

            for (digger, (wants_to_dig, stats, named)) in &mut collectors {
                let mut diggable_query = ecs_world
                    .query_one::<(&mut Diggable, &Position)>(wants_to_dig.target)
                    .expect("target must be diggable in a position!");
                let (diggable, pos) = diggable_query
                    .get()
                    .expect("must have Some diggable in a position!");

                // Extract dig tool stats. Could be a real item (InflictsDamage) or just the CombatStats of a StoneEater monster
                let mut dig_tool_query = ecs_world
                    .query_one::<(Option<&InflictsDamage>, Option<&CombatStats>)>(wants_to_dig.tool)
                    .unwrap_or_else(|_| {
                        panic!(
                            "Must have InflictsDamage or CombatStats on dig tool {:?}!",
                            wants_to_dig.tool
                        )
                    });
                let (diggable_inf_dam, dig_tool_combat_stats) = dig_tool_query
                    .get()
                    .expect("must have Some diggable in a position!");

                let dig_tool_dice: (i32, i32) = if let Some(stats) = dig_tool_combat_stats {
                    (1, stats.unarmed_attack_dice)
                } else if let Some(inf_dam) = diggable_inf_dam {
                    // Double the number of dices to account for the cracked wall
                    (inf_dam.number_of_dices * 2, inf_dam.dice_size)
                } else {
                    panic!("Cannot determine dig tool dice!");
                };

                // Subtract dig points and log
                let dig_roll = Roll::dice(dig_tool_dice.0, dig_tool_dice.1);
                diggable.dig_points -= dig_roll;
                diggers_list.push((digger, stats.speed));
                //Create mock edibile for stone eaters only (the entity must have CombatStats component)
                if dig_tool_combat_stats.is_some() {
                    wants_to_eat_list.push((digger, dig_roll));
                }

                if digger.id() == player_id {
                    game_state
                        .game_log
                        .entries
                        .push(Cow::Borrowed("You dig the cracked stone wall"));
                } else if zone.visible_tiles[Zone::get_index_from_xy(&pos.x, &pos.y)] {
                    game_state.game_log.add_entry(Cow::Owned(format!(
                        "The {} digs the cracked stone wall",
                        named.name
                    )));
                }

                // Clear path if digged enough
                if diggable.dig_points <= 0 {
                    if zone.visible_tiles[Zone::get_index_from_xy(&pos.x, &pos.y)] {
                        game_state
                            .game_log
                            .entries
                            .push(Cow::Borrowed("The cracked wall opens!"));
                    } else {
                        game_state
                            .game_log
                            .entries
                            .push(Cow::Borrowed("You hear falling rocks"));
                    }

                    zone.tiles[Zone::get_index_from_xy(&pos.x, &pos.y)] = TileType::Floor;

                    digged_list.push(wants_to_dig.target);

                    match diggable.produces {
                        DigProductEnum::Gold => {
                            produced_list.push((pos.x, pos.y, diggable.produces.clone()));
                        }
                        _ => {
                            // Could randomly produce the diggable's product (usually stone)
                            if Roll::d6() == 1 {
                                produced_list.push((pos.x, pos.y, diggable.produces.clone()));
                            }
                        }
                    }
                }
            }
        }

        for (digger, speed) in diggers_list {
            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsToDig>(digger);
            Utils::wait_after_action(ecs_world, digger, speed);
        }

        for target in digged_list {
            let _ = ecs_world.despawn(target);
        }

        // Spawn edible stones for diggers that eat stones
        for (digger, dig_roll) in wants_to_eat_list {
            let item = Spawn::edible_stone(ecs_world, dig_roll);
            let _ = ecs_world.insert_one(digger, WantsToEat { item });
        }

        for (x, y, product) in produced_list {
            match product {
                DigProductEnum::Gold => {
                    let _ = Spawn::raw_gold(ecs_world, x, y);
                }
                DigProductEnum::Stone => {
                    let _ = Spawn::slingshot_ammo(ecs_world, x, y);
                }
            }
        }
    }
}
