use hecs::{Entity, World};

use crate::{
    components::{combat::WantsToMelee, common::*, monster::Monster, player::Player},
    maps::zone::Zone,
    utils::pathfinding::Pathfinding,
};

/// Monster AI struct
pub struct MonsterAI {}

impl MonsterAI {
    /// Monster acting function
    pub fn act(ecs_world: &mut World) {
        let mut attacker_target_list: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut named_monsters = ecs_world.query::<(&mut Viewshed, &Monster, &mut Position)>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_e, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            let mut player_query = ecs_world.query::<(&Player, &Position)>();
            let (player_entity, (_p, player_position)) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");

            // For each viewshed position monster component join
            for (monster_entity, (viewshed, _monster, position)) in &mut named_monsters {
                //If enemy can see player, follow him and try to attack when close enough
                if viewshed
                    .visible_tiles
                    .contains(&(player_position.x, player_position.y))
                {
                    let pathfinding_result = Pathfinding::dijkstra_wrapper(
                        position.x,
                        position.y,
                        player_position.x,
                        player_position.y,
                        zone,
                        true,
                    );

                    //If can actually reach the player
                    if pathfinding_result.is_some() {
                        let distance = ((position.x.abs_diff(player_position.x).pow(2)
                            + position.y.abs_diff(player_position.y).pow(2))
                            as f32)
                            .sqrt();

                        //Attack or move
                        if distance < 1.5 {
                            attacker_target_list.push((monster_entity, player_entity));
                        } else {
                            viewshed.must_recalculate = true;
                            let (path, _c) = pathfinding_result.unwrap();

                            // Avoid overlap with other monsters and player
                            if path.len() > 1 {
                                zone.blocked_tiles
                                    [Zone::get_index_from_xy(position.x, position.y)] = false;
                                position.x = path[1].0;
                                position.y = path[1].1;
                                zone.blocked_tiles
                                    [Zone::get_index_from_xy(position.x, position.y)] = true;
                            }
                        }
                    }
                }
            }
        }

        // Attack if needed
        for (attacker, target) in attacker_target_list {
            let _ = ecs_world.insert_one(attacker, WantsToMelee { target });
        }
    }
}
