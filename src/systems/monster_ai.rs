use hecs::World;

use crate::{
    components::{common::*, monster::Monster, player::Player},
    utils::point::Point,
};

/// Monster AI struct
pub struct MonsterAI {}

impl MonsterAI {
    /// Monster acting function
    pub fn act(ecs_world: &World) {
        let mut named_monsters = ecs_world.query::<(&Viewshed, &Monster, &Name)>();
        let mut player_query = ecs_world.query::<(&Player, &Position)>();
        let (_e, (_p, player_position)) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        // For each viewshed position monster component join
        for (_e, (viewshed, _monster, name)) in &mut named_monsters {
            //This is the same as println but for WebAssembly
            if viewshed.visible_tiles.contains(&Point {
                x: player_position.x,
                y: player_position.y,
            }) {
                println!("{} do horrible stuff", name.name)
            }
        }
    }
}
