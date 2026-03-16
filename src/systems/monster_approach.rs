use hecs::Entity;

use crate::{
    components::{
        combat::{CombatStats, Grappled, SufferingDamage},
        common::*,
        monster::{Aquatic, LeaveTrail, Monster, SnakeBody, SnakeHead, WantsToApproach},
    },
    engine::state::GameState,
    maps::zone::{DecalType, Zone},
    utils::{common::Utils, pathfinding::Pathfinding, roll::Roll},
};

/// Monster AI struct
pub struct MonsterApproach {}

impl MonsterApproach {
    /// Monster acting function
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;

        let mut waiter_speed_list: Vec<(Entity, i32)> = Vec::new();
        let mut approacher_list: Vec<Entity> = Vec::new();
        // Scope for keeping borrow checker quiet
        {
            let mut named_monsters = ecs_world
                .query::<(
                    &mut Viewshed,
                    &mut Position,
                    &Named,
                    &CombatStats,
                    &mut SufferingDamage,
                    Option<&Aquatic>,
                    &mut WantsToApproach,
                    Option<&LeaveTrail>,
                    Option<&Immobile>,
                    Option<&SnakeHead>,
                    Option<&Grappled>,
                )>()
                .with::<(&Monster, &MyTurn)>()
                .without::<&SnakeBody>();

            let zone = game_state
                .current_zone
                .as_mut()
                .expect("must have Some Zone");

            // For each viewshed position monster component join
            for (
                monster_entity,
                (
                    viewshed,
                    position,
                    named,
                    stats,
                    suffering_damage,
                    aquatic_opt,
                    wants_to_approach,
                    leave_trail_opt,
                    immobile_opt,
                    snake_head_opt,
                    grappled_opt,
                ),
            ) in &mut named_monsters
            {
                let current_pos_index = Zone::get_index_from_xy(&position.x, &position.y);

                // Checking if could slip on slime before moving away
                if leave_trail_opt.is_none() && snake_head_opt.is_none() // Snake monsters cannot slip (TODO really?)
                    && let Some(special_tile) = zone.decals_tiles.get(&current_pos_index)
                    && let DecalType::Slime = special_tile
                {
                    // Do DEX saving or slip on slime!
                    if stats.current_dexterity < Roll::d20() {
                        waiter_speed_list.push((monster_entity, stats.speed));
                        if zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)] {
                            // Log NPC infighting only if visible
                            game_state
                                .game_log
                                .entries
                                .push(format!("The {} slips on the slime!", named.name));
                        }
                        continue;
                    }
                }

                //Monster must wait too after an action, even if this turn will not move!
                waiter_speed_list.push((monster_entity, stats.speed));

                // Do not do anything if the monster is immobile
                if immobile_opt.is_some() {
                    approacher_list.push(monster_entity);
                    continue;
                } else if let Some(grappled) = grappled_opt {
                    if Roll::d20() <= stats.current_dexterity
                        && let Ok(mut g_query) =
                            ecs_world.query_one::<(&Named, &CombatStats)>(grappled.by)
                    {
                        let (grappler_name, grappler_stats) =
                            g_query.get().expect("g_query must have result");

                        // Grappler lose turn
                        if zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)] {
                            game_state.game_log.entries.push(format!(
                                "The {} escapes the {}'s grasp!",
                                named.name, grappler_name.name
                            ));
                        }
                        waiter_speed_list.push((grappled.by, grappler_stats.speed));
                    } else {
                        // Stop moving
                        waiter_speed_list.push((monster_entity, stats.speed));
                        continue;
                    }
                }

                let pathfinding_result = Pathfinding::dijkstra_wrapper(
                    position.x,
                    position.y,
                    wants_to_approach.target_x,
                    wants_to_approach.target_y,
                    zone,
                    true,
                    aquatic_opt.is_some(),
                );

                //If can actually reach the new position, do it or else stay still
                if let Some((path, _)) = pathfinding_result
                    && path.len() > 1
                {
                    // Update view
                    viewshed.must_recalculate = true;

                    // Avoid overlap with other monsters and player
                    zone.blocked_tiles[Zone::get_index_from_xy(&position.x, &position.y)] = false;

                    // Shift snake body parts to previous part position
                    // .......    .......
                    // ..<000. => .<000..
                    // .......    .......
                    if let Some(snake) = snake_head_opt {
                        let mut new_x = position.x;
                        let mut new_y = position.y;
                        let mut previous_x;
                        let mut previous_y;
                        for &body_part in snake.body.iter() {
                            if let Ok(mut part_pos) = ecs_world.get::<&mut Position>(body_part) {
                                // Shift each part of snake body
                                previous_x = part_pos.x;
                                previous_y = part_pos.y;
                                part_pos.x = new_x;
                                part_pos.y = new_y;
                                new_x = previous_x;
                                new_y = previous_y;
                            } else {
                                panic!("Why snake body part do not have Position?")
                            }
                        }
                    }

                    //Move monster (or head)
                    position.x = path[1].0;
                    position.y = path[1].1;
                    zone.blocked_tiles[Zone::get_index_from_xy(&position.x, &position.y)] = true;

                    // Checking if could step on acid after moving
                    if leave_trail_opt.is_none()
                        && let Some(special_tile) = zone.decals_tiles.get(&current_pos_index)
                        && let DecalType::Acid = special_tile
                    {
                        // Do DEX saving or slip on slime!
                        if stats.current_dexterity < Roll::d20() {
                            suffering_damage.damage_received += Roll::dice(1, 3);
                            if zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)]
                            {
                                // Log only if visible
                                game_state
                                    .game_log
                                    .entries
                                    .push(format!("The {} burn itself on the acid!", named.name));
                            }
                        }
                    }

                    if wants_to_approach.counter == 0 {
                        // Approached point, stop moving
                        approacher_list.push(monster_entity);
                    } else {
                        wants_to_approach.counter -= 1;
                    }
                } else {
                    // Approached point, stop moving
                    approacher_list.push(monster_entity);
                }
            }
        }

        // TODO account speed penalties
        for (waiter, speed) in waiter_speed_list {
            Utils::wait_after_action(ecs_world, waiter, speed);
        }

        for approacher in approacher_list {
            let _ = ecs_world.remove_one::<WantsToApproach>(approacher);
        }
    }
}
