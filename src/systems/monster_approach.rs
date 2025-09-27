use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::CombatStats,
        common::*,
        monster::{Aquatic, Monster, WantsToApproach},
    },
    maps::zone::Zone,
    utils::{common::Utils, pathfinding::Pathfinding},
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
                    &CombatStats,
                    Option<&Aquatic>,
                    &WantsToApproach,
                )>()
                .with::<(&Monster, &MyTurn)>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            // For each viewshed position monster component join
            for (monster_entity, (viewshed, position, stats, aquatic, wants_to_approach)) in
                &mut named_monsters
            {
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

                // Does this entity still exist?
                approacher_list.push(monster_entity);
                if ecs_world.contains(wants_to_approach.target) {
                    let target_position = ecs_world
                        .get::<&Position>(wants_to_approach.target)
                        .expect("target must have Position");

                    let pathfinding_result = Pathfinding::dijkstra_wrapper(
                        position.x,
                        position.y,
                        target_position.x,
                        target_position.y,
                        zone,
                        true,
                        aquatic.is_some(),
                    );

                    //If can actually reach the player
                    if let Some((path, _)) = pathfinding_result
                        && path.len() > 1
                    {
                        // Update view
                        viewshed.must_recalculate = true;

                        // Avoid overlap with other monsters and player
                        zone.blocked_tiles[Zone::get_index_from_xy(position.x, position.y)] = false;
                        position.x = path[1].0;
                        position.y = path[1].1;
                        zone.blocked_tiles[Zone::get_index_from_xy(position.x, position.y)] = true;

                        //Monster must wait too after an action!
                        waiter_speed_list.push((monster_entity, stats.speed));
                    }
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
