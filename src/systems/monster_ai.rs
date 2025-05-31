use std::cmp::max;

use hecs::World;

use crate::{
    components::{
        combat::{CombatStats, Damageable},
        common::*,
        monster::Monster,
        player::Player,
    },
    map::{Map, get_index_from_xy},
    utils::{pathfinding_utils::PathfindingUtils, point::Point, random_util::RandomUtils},
};

/// Monster AI struct
pub struct MonsterAI {}

impl MonsterAI {
    /// Monster acting function
    pub fn act(ecs_world: &World) {
        let mut named_monsters =
            ecs_world.query::<(&mut Viewshed, &Monster, &mut Position, &Named, &CombatStats)>();

        let mut map_query = ecs_world.query::<&mut Map>();
        let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

        let mut player_query =
            ecs_world.query::<(&Player, &mut Damageable, &Position, &CombatStats)>();
        let (_e, (_p, player, player_position, player_stats)) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        let mut game_log_query = ecs_world.query::<&mut GameLog>();
        let (_e, game_log) = game_log_query
            .iter()
            .last()
            .expect("Game log is not in hecs::World");

        // For each viewshed position monster component join
        for (_e, (viewshed, _monster, position, named, monster_stats)) in &mut named_monsters {
            //If enemy can see player, follow him and try to attack when close enough
            if viewshed.visible_tiles.contains(&Point {
                x: player_position.x,
                y: player_position.y,
            }) {
                let pathfinding_result = PathfindingUtils::a_star_wrapper(
                    position.x,
                    position.y,
                    player_position.x,
                    player_position.y,
                    map,
                );

                //If can actually reach the player
                if pathfinding_result.is_some() {
                    let distance = ((position.x.abs_diff(player_position.x).pow(2)
                        + position.y.abs_diff(player_position.y).pow(2))
                        as f32)
                        .sqrt();

                    //Attack or move
                    if distance < 1.5 {
                        let damage = max(
                            0,
                            RandomUtils::dice(1, monster_stats.unarmed_attack_dice) - player_stats.base_armor,
                        );

                        // Add game log
                        game_log.entries.push(format!(
                            "{} hits you for {} damage",
                            named.name, damage
                        ));

                        player.damage_received += damage;
                    } else {
                        viewshed.must_recalculate = true;
                        let (path, _c) = pathfinding_result.unwrap();

                        // Avoid overlap with other monsters and player
                        if path.len() > 1 {
                            map.blocked_tiles[get_index_from_xy(position.x, position.y)] = false;
                            position.x = path[1].0;
                            position.y = path[1].1;
                            map.blocked_tiles[get_index_from_xy(position.x, position.y)] = true;
                        }
                    }
                }
            }
        }
    }
}
