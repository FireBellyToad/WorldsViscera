use hecs::World;

use crate::{
    components::{
        common::{MyTurn, Position},
        monster::{LeaveTrail, TrailPlaceholder},
    },
    maps::zone::Zone,
};

pub struct LeaveTrailSystem {}

impl LeaveTrailSystem {
    pub fn run(ecs_world: &mut World) {
        let mut trail_to_spawn: Vec<(usize, u32)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            // List of entities that has stats
            let mut trailers = ecs_world
                .query::<(&LeaveTrail, &Position)>()
                .with::<&MyTurn>();

            for (_, (leave, position)) in &mut trailers {
                let trail_pos_idx = Zone::get_index_from_xy(&position.x, &position.y);
                if !zone.water_tiles[trail_pos_idx] {
                    // Insert the trail tile at the entity's position
                    let _ = zone.decals_tiles.insert(trail_pos_idx, leave.of.clone());
                    //get ready to spawn trail counter entity
                    trail_to_spawn.push((trail_pos_idx, leave.trail_lifetime));
                }
            }
        }

        // Spawn trail entities at positions in trail_to_spawn.
        // This is done to ensure that the trail will vanish after a certain time.
        for (trail_pos_idx, trail_counter) in trail_to_spawn {
            ecs_world.spawn((
                true,
                TrailPlaceholder {
                    trail_counter,
                    trail_pos_idx,
                },
            ));
        }
    }

    /// Handle spawned trail entities and despawn them after a certain time, cleaning up the zone's trail decals
    pub fn handle_spawned_trail(ecs_world: &mut World) {
        let mut to_despawn = Vec::new();
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut trails_spawned = ecs_world.query::<&mut TrailPlaceholder>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            for (entity, trail) in &mut trails_spawned {
                if trail.trail_counter == 0 {
                    zone.decals_tiles.remove(&trail.trail_pos_idx);
                    to_despawn.push(entity);
                } else {
                    trail.trail_counter -= 1;
                }
            }
        }

        for entity in to_despawn {
            let _ = ecs_world.despawn(entity);
        }
    }
}
