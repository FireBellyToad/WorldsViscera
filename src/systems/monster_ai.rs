use hecs::World;

use crate::{
    components::{common::*, monster::Monster, player::Player},
    map::{Map, get_index_from_xy},
    utils::{pathfinding_utils::PathfindingUtils, point::Point},
};

/// Monster AI struct
pub struct MonsterAI {}

impl MonsterAI {
    /// Monster acting function
    pub fn act(ecs_world: &World) {
        let mut named_monsters =
            ecs_world.query::<(&mut Viewshed, &Monster, &mut Position, &Named)>();

        let mut map_query = ecs_world.query::<&mut Map>();
        let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

        let mut player_query = ecs_world.query::<(&Player, &Target)>();
        let (_e, (_p, player_position)) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        // For each viewshed position monster component join
        for (_e, (viewshed, _monster, position, named)) in &mut named_monsters {
            //This is the same as println but for WebAssembly
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
                let distance = ((position.x.abs_diff(player_position.x).pow(2)
                    + position.y.abs_diff(player_position.y).pow(2))
                    as f32)
                    .sqrt();

                if distance < 1.5 {
                    println!("{} licks your skin", named.name)
                } else if pathfinding_result.is_some() {
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
