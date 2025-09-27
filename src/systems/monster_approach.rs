use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToEat, combat::{CombatStats, WantsToMelee}, common::*, items::{Edible, Item}, monster::{Monster, WantsToApproach}
    },
    constants::MAX_ACTION_SPEED,
    maps::zone::Zone,
    utils::common::Utils,
};

/// Monster AI struct
pub struct MonsterApproach {}

impl MonsterApproach {
    /// Monster acting function
    pub fn run(ecs_world: &mut World) {
        let mut attacker_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut eat_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut waiter_speed_list: Vec<(Entity, i32)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut named_monsters = ecs_world
                .query::<(&mut Viewshed, &mut Position, &CombatStats, &WantsToApproach)>()
                .with::<(&Monster, &MyTurn)>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            // For each viewshed position monster component join
            for (monster_entity, (viewshed, position, stats, wants_to_approach)) in
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
                let target_real_position = ecs_world
                    .get::<&Position>(wants_to_approach.target)
                    .expect("Entity has no Position");

                let distance = Utils::distance(
                    position.x,
                    target_real_position.x,
                    position.y,
                    target_real_position.y,
                );

                //Attack or move
                // TODO just reaching, maybe we don't need to attack
                let target_has_stats = ecs_world.satisfies::<&CombatStats>(wants_to_approach.target).unwrap_or(false);
                let target_is_edible_item = ecs_world.satisfies::<(&Item,&Edible)>(wants_to_approach.target).unwrap_or(false);
                if distance < 1.5 && target_has_stats {
                    // TODO this is nice, but we must handle it in during the thinking phasse
                    attacker_target_list.push((monster_entity, wants_to_approach.target));
                    //Monster must wait too after an action!
                    waiter_speed_list.push((monster_entity, stats.speed));
                } else if distance == 0.0 && target_is_edible_item{
                    // TODO this is nice, but we must handle it in during the thinking phasse
                    eat_target_list.push((monster_entity, wants_to_approach.target));
                    //Monster must wait too after an action!
                    waiter_speed_list.push((monster_entity, stats.speed));
                }else{
                    // Update view
                    viewshed.must_recalculate = true;

                    // Avoid overlap with other monsters and player
                    zone.blocked_tiles[Zone::get_index_from_xy(position.x, position.y)] = false;
                    position.x = wants_to_approach.move_to_x;
                    position.y = wants_to_approach.move_to_y;
                    zone.blocked_tiles[Zone::get_index_from_xy(position.x, position.y)] = true;

                    //Monster must wait too after an action!
                    waiter_speed_list.push((monster_entity, stats.speed));
                }
            }
        }

        // Attack if needed
        for (attacker, target) in attacker_target_list {
            let _ = ecs_world.insert_one(attacker, WantsToMelee { target });
        }

        // eat if needed
        for (eater, item) in eat_target_list {
            let _ = ecs_world.insert_one(eater, WantsToEat { item});
        }

        // TODO account speed penalties
        for (must_wait, speed) in waiter_speed_list {
            let _ = ecs_world.exchange_one::<MyTurn, WaitingToAct>(
                must_wait,
                WaitingToAct {
                    tick_countdown: max(1, MAX_ACTION_SPEED - speed),
                },
            );
        }
    }
}
