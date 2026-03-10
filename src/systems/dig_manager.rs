use hecs::Entity;

use crate::{
    components::{
        actions::WantsToDig,
        combat::{CombatStats, InflictsDamage},
        common::{DigProductEnum, Diggable, MyTurn, Position},
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

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to dig
            let mut collectors = ecs_world
                .query::<(&WantsToDig, &CombatStats)>()
                .with::<&MyTurn>();

            let zone = game_state
                .current_zone
                .as_mut()
                .expect("must have Some Zone");

            //Log all the pick ups

            for (digger, (wants_to_dig, stats)) in &mut collectors {
                let mut diggable_query = ecs_world
                    .query_one::<(&mut Diggable, &Position)>(wants_to_dig.target)
                    .expect("target must be diggable in a position!");
                let dig_tool_dice = ecs_world
                    .get::<&InflictsDamage>(wants_to_dig.tool)
                    .expect("must be diggable!");

                let (diggable, pos) = diggable_query
                    .get()
                    .expect("must have Some diggable in a position!");
                // Subtract dig points and log
                diggable.dig_points -=
                    Roll::dice(dig_tool_dice.number_of_dices, dig_tool_dice.dice_size);
                diggers_list.push((digger, stats.speed));

                if digger.id() == player_id {
                    game_state
                        .game_log
                        .entries
                        .push("You dig the cracked stone wall".to_string());
                }

                // Clear path if digged enough
                if diggable.dig_points <= 0 {
                    game_state
                        .game_log
                        .entries
                        .push("The cracked wall opens!".to_string());

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
