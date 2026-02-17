use hecs::Entity;

use crate::{
    components::{
        actions::WantsToDig,
        combat::{CombatStats, InflictsDamage},
        common::{Diggable, MyTurn, Position},
    },
    engine::state::GameState,
    maps::zone::{TileType, Zone},
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
                let mut diggable = ecs_world
                    .get::<&mut Diggable>(wants_to_dig.target)
                    .expect("must be diggable!");
                let dig_tool_dice = ecs_world
                    .get::<&InflictsDamage>(wants_to_dig.tool)
                    .expect("must be diggable!");

                // Subtract dig points and log
                diggable.dig_points -=
                    Roll::dice(dig_tool_dice.number_of_dices, dig_tool_dice.dice_size);
                diggers_list.push((digger, stats.speed));

                if digger.id() == player_id {
                    game_state
                        .game_log
                        .entries
                        .push("You dig the crack in the stone wall".to_string());
                }

                // Clear path if digged enough
                if diggable.dig_points <= 0 {
                    game_state
                        .game_log
                        .entries
                        .push("The cracked wall opens!".to_string());

                    let pos = ecs_world
                        .get::<&Position>(wants_to_dig.target)
                        .expect("must have Position!");

                    // TODO drop stones!
                    zone.tiles[Zone::get_index_from_xy(&pos.x, &pos.y)] = TileType::Floor;

                    digged_list.push(wants_to_dig.target);
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
    }
}
