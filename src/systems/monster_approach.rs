use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::*,
        monster::{Aquatic, LeaveTrail, Monster, WantsToApproach},
    },
    maps::zone::{DecalType, Zone},
    utils::{common::Utils, pathfinding::Pathfinding, roll::Roll},
};

/// Monster AI struct
pub struct MonsterApproach {}

impl MonsterApproach {
    /// Monster acting function
    pub fn run(ecs_world: &mut World) {
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
                )>()
                .with::<(&Monster, &MyTurn)>();

            //Log all actions
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

            // For each viewshed position monster component join
            for (
                monster_entity,
                (
                    viewshed,
                    position,
                    named,
                    stats,
                    suffering_damage,
                    aquatic,
                    wants_to_approach,
                    leave_trail,
                ),
            ) in &mut named_monsters
            {
                let current_pos_index = Zone::get_index_from_xy(&position.x, &position.y);
                if leave_trail.is_none()
                    && let Some(special_tile) = zone.decals_tiles.get(&current_pos_index)
                {
                    match special_tile {
                        DecalType::Slime => {
                            // Do DEX saving or slip on slime!
                            if stats.current_dexterity < Roll::d20() {
                                waiter_speed_list.push((monster_entity, stats.speed));
                                if zone.visible_tiles
                                    [Zone::get_index_from_xy(&position.x, &position.y)]
                                {
                                    // Log NPC infighting only if visible
                                    game_log
                                        .entries
                                        .push(format!("The {} slips on the slime!", named.name));
                                }
                                continue;
                            }
                        }
                        DecalType::Acid => {
                            // Do DEX saving or slip on slime!
                            if stats.current_dexterity < Roll::d20() {
                                suffering_damage.damage_received += Roll::dice(1, 3);
                                if zone.visible_tiles
                                    [Zone::get_index_from_xy(&position.x, &position.y)]
                                {
                                    // Log only if visible
                                    game_log.entries.push(format!(
                                        "The {} burn itself on the acid!",
                                        named.name
                                    ));
                                }
                                continue;
                            }
                        }
                        _ => {}
                    }
                }

                //Monster must wait too after an action, even if this turn will not move!
                waiter_speed_list.push((monster_entity, stats.speed));

                let (move_to_x, move_to_y) =
                    (wants_to_approach.target_x, wants_to_approach.target_y);

                if zone.blocked_tiles[Zone::get_index_from_xy(&move_to_x, &move_to_y)] {
                    // If destination is somehow now blocked, monster move to first empty space from top left.
                    let mut can_move = false;
                    for y in position.y - 1..position.y + 1 {
                        for x in position.x - 1..position.x + 1 {
                            if !zone.blocked_tiles[Zone::get_index_from_xy(&x, &y)] {
                                wants_to_approach.target_x = x;
                                wants_to_approach.target_y = y;
                                can_move = true;
                                break;
                            }
                        }
                    }
                    //If none found, just stops
                    if !can_move {
                        continue;
                    }
                }

                let pathfinding_result = Pathfinding::dijkstra_wrapper(
                    position.x,
                    position.y,
                    move_to_x,
                    move_to_y,
                    zone,
                    true,
                    aquatic.is_some(),
                );

                //If can actually reach the new position, do it or else stay still
                if let Some((path, _)) = pathfinding_result
                    && path.len() > 1
                {
                    // Update view
                    viewshed.must_recalculate = true;

                    // Avoid overlap with other monsters and player
                    zone.blocked_tiles[Zone::get_index_from_xy(&position.x, &position.y)] = false;
                    position.x = path[1].0;
                    position.y = path[1].1;
                    zone.blocked_tiles[Zone::get_index_from_xy(&position.x, &position.y)] = true;

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
