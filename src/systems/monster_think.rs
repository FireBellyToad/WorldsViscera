use hecs::{Entity, World};

use crate::{
    components::{
        common::*,
        monster::{Aquatic, Monster, WantsToApproach},
        player::Player,
    },
    maps::zone::Zone,
    utils::pathfinding::Pathfinding,
};

/// Monster Think struct
pub struct MonsterThink {}

impl MonsterThink {
    /// Monster acting function
    pub fn run(ecs_world: &mut World) {
        let mut approacher_list: Vec<(Entity, Entity, (i32, i32))> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut named_monsters = ecs_world
                .query::<(&mut Viewshed, &mut Position, Option<&Aquatic>)>()
                .with::<(&Monster, &MyTurn)>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            let mut player_query = ecs_world.query::<&Position>().with::<&Player>();
            let (player_entity, player_position) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");

            // For each viewshed position monster component join
            for (monster, (viewshed, position, acquatic)) in &mut named_monsters {
                /*
                1. Quando X vede una creatura Y

                    1.1 Se Y non è della sua specie e X è STARVED e X ha un livello maggiore o uguale a Y+1, X lo attaccherà
                    1.2 Se Y è di una specie al quale X è ostile e X è in buona salute e X ha un livello maggiore o uguale a Y+1, X lo attaccherà
                    1.3 Se Y è di una specie al quale X è ostile e X è in pericolo o X ha un livello minore di Y+1, X fuggirà
                    1.4 Altrimenti lo ignorerà se non per reagire ad eventuali attacchi.

                1. Quando X vede un oggetto Y

                    2.1 Se Y è edibile e X non è sazio, X prova a mangiarlo.
                    2.2 Se Y è bevibile e X non è quenched, X prova a berlo
                    2.3 Se Y è qualcos'altro, X è astuto e ha spazio nell'inventario, X lo raccoglierà

                3. Si muove casualmente nella zona

                */

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
                        acquatic.is_some(),
                    );

                    //If can actually reach the player
                    if let Some((path, _)) = pathfinding_result {
                        // Approach player
                        // TODO be more generalized! Player is not the only one to follow
                        // TODO What about wandering monsters? Target must be optional
                        approacher_list.push((monster, player_entity, (path[1].0, path[1].1)));
                    }
                }
            }
        }

        // Approach if needed
        for (approacher, target, (move_to_x, move_to_y)) in approacher_list {
            let _ = ecs_world.insert_one(
                approacher,
                WantsToApproach {
                    target,
                    move_to_x,
                    move_to_y,
                },
            );
        }
    }
}
