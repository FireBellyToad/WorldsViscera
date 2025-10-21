use hecs::{Entity, World};

use crate::{
    components::{
        combat::CombatStats,
        common::*,
        monster::{Aquatic, Monster, WantsToApproach},
    },
    maps::zone::Zone,
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
                    &CombatStats,
                    Option<&Aquatic>,
                    &mut WantsToApproach,
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
                let (move_to_x, move_to_y) =
                    (wants_to_approach.target_x, wants_to_approach.target_y);

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
                    zone.blocked_tiles[Zone::get_index_from_xy(position.x, position.y)] = false;
                    position.x = path[1].0;
                    position.y = path[1].1;
                    zone.blocked_tiles[Zone::get_index_from_xy(position.x, position.y)] = true;

                    //Monster must wait too after an action!
                    waiter_speed_list.push((monster_entity, stats.speed));
                }

                if wants_to_approach.counter == 0 {
                    // Does this entity still exist and has a position?
                    approacher_list.push(monster_entity);
                } else {
                    wants_to_approach.counter -= 1;
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
