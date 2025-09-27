use hecs::{Entity, World};

use crate::{
    components::{
        common::*,
        health::Hunger,
        items::{Edible, Item},
        monster::{Aquatic, IsSmart, Monster, WantsToApproach},
        player::Player,
    },
    maps::zone::Zone,
    systems::hunger_check::HungerStatus,
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
                .query::<(
                    &mut Viewshed,
                    &mut Position,
                    &Hunger,
                    Option<&IsSmart>,
                    Option<&Aquatic>,
                )>()
                .with::<(&Monster, &MyTurn)>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            let player_id = Player::get_entity_id(ecs_world);

            // For each viewshed position monster component join
            for (monster, (viewshed, position, hunger, smart, acquatic)) in &mut named_monsters {
                let target_picked = MonsterThink::pick_target(
                    ecs_world,
                    zone,
                    viewshed,
                    hunger,
                    &player_id,
                    smart.is_some(),
                );

                //If enemy can see player, follow him and try to attack when close enough
                if let Some((target, target_x, target_y)) = target_picked {
                    println!("position x {} y {}", position.x, position.y);
                    println!("target x {} y {}", target_x, target_y);
                    let pathfinding_result = Pathfinding::dijkstra_wrapper(
                        position.x,
                        position.y,
                        target_x,
                        target_y,
                        zone,
                        true,
                        acquatic.is_some(),
                    );

                    //If can actually reach the player
                    if let Some((path, _)) = pathfinding_result {
                        println!("path len {}", path.len());
                        if path.len() > 1 {
                            // Approach something of its interest
                            // TODO What about wandering monsters? Target must be optional
                            approacher_list.push((monster, target, (path[1].0, path[1].1)));
                        }
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

    /// pick a target from visible tiles
    fn pick_target(
        ecs_world: &World,
        zone: &Zone,
        viewshed: &Viewshed,
        hunger: &Hunger,
        player_id: &u32,
        _is_smart: bool,
    ) -> Option<(Entity, i32, i32)> {
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

        for (x, y) in viewshed.visible_tiles.iter() {
            // println!("checking x {} y {}", x, y);
            let index = Zone::get_index_from_xy(*x, *y);
            for &entity in &zone.tile_content[index] {
                // If less than Satiated try to eat something edible
                if hunger.current_status != HungerStatus::Satiated
                    && ecs_world.satisfies::<(&Item,&Edible)>(entity).unwrap_or(false)
                {
                    println!(
                        "Entity with id {} at x {} y {} is edible",
                        entity.id(),
                        *x,
                        *y
                    );
                    return Some((entity, *x, *y));
                } else if *player_id == entity.id() {
                    // TODO not only player, also things that the monster HATE
                    println!("Entity with id {} is player", entity.id());
                    return Some((entity, *x, *y));
                }
            }
        }

        // TODO Order by priority

        None
    }
}
