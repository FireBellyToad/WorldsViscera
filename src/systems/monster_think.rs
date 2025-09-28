use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToEat,
        combat::{CombatStats, WantsToMelee},
        common::*,
        health::Hunger,
        items::{Edible, Item},
        monster::{Aquatic, IsSmart, Monster, WantsToApproach},
        player::Player,
    },
    maps::zone::Zone,
    systems::hunger_check::HungerStatus,
    utils::{common::Utils, pathfinding::Pathfinding},
};

/// Monster Think struct
pub struct MonsterThink {}

impl MonsterThink {
    /// Monster acting function
    pub fn run(ecs_world: &mut World) {
        let mut approacher_list: Vec<(Entity, Option<Entity>)> = Vec::new();
        let mut attacker_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut eat_target_list: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut named_monsters = ecs_world
                .query::<(
                    &mut Viewshed,
                    &mut Position,
                    &CombatStats,
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
            for (monster, (viewshed, position, stats, hunger, smart, aquatic)) in
                &mut named_monsters
            {
                let target_picked = MonsterThink::pick_target(
                    ecs_world,
                    zone,
                    viewshed,
                    hunger,
                    &monster.id(),
                    &player_id,
                    smart.is_some(),
                );

                //If enemy can see player, follow him and try to attack when close enough
                if let Some(target) = target_picked {
                    let mut target_query = ecs_world
                        .query_one::<(&Position, Option<&CombatStats>, Option<(&Item, &Edible)>)>(
                            target,
                        )
                        .expect("target_query failed");
                    let (target_position, target_has_stats, target_is_edible_item) = target_query
                        .get()
                        .expect("cannot extract result from target_query");

                    let distance = Utils::distance(
                        position.x,
                        target_position.x,
                        position.y,
                        target_position.y,
                    );

                    //Attack or move
                    if distance < 1.5 && target_has_stats.is_some() {
                        // TODO this is nice, but we must handle it in during the thinking phasse
                        attacker_target_list.push((monster, target));
                    } else if distance == 0.0 && target_is_edible_item.is_some() {
                        // TODO this is nice, but we must handle it in during the thinking phasse
                        eat_target_list.push((monster, target));
                    } else {
                        let pathfinding_result = Pathfinding::dijkstra_wrapper(
                            position.x,
                            position.y,
                            target_position.x,
                            target_position.y,
                            zone,
                            true,
                            aquatic.is_some(),
                        );

                        //If can actually reach the position
                        if let Some((path, _)) = pathfinding_result {
                            if path.len() > 1 {
                                // Approach something of its interest
                                // TODO What about wandering monsters? Target must be optional
                                approacher_list.push((monster, Some(target)));
                            }
                        } else {
                            approacher_list.push((monster, None));
                        }
                    }
                } else {
                    approacher_list.push((monster, None));
                }
            }
        }

        // Approach if needed
        for (approacher, target) in approacher_list {
            let _ = ecs_world.insert_one(approacher, WantsToApproach { target });
        }

        // Attack if needed
        for (attacker, target) in attacker_target_list {
            let _ = ecs_world.insert_one(attacker, WantsToMelee { target });
        }

        // eat if needed
        for (eater, item) in eat_target_list {
            let _ = ecs_world.insert_one(eater, WantsToEat { item });
        }
    }

    /// pick a target from visible tiles
    fn pick_target(
        ecs_world: &World,
        zone: &Zone,
        viewshed: &Viewshed,
        hunger: &Hunger,
        _self_id: &u32,
        player_id: &u32,
        _is_smart: bool,
    ) -> Option<Entity> {
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
            let index = Zone::get_index_from_xy(*x, *y);
            for &entity in &zone.tile_content[index] {
                // If less than Satiated try to eat something edible
                if hunger.current_status != HungerStatus::Satiated
                    && ecs_world
                        .satisfies::<(&Item, &Edible)>(entity)
                        .unwrap_or(false)
                {
                    println!(
                        "Entity with id {} at x {} y {} is edible",
                        entity.id(),
                        *x,
                        *y
                    );
                    return Some(entity);
                } else if *player_id == entity.id() {
                    // TODO not only player, also things that the monster HATE
                    println!("Entity with id {} is player", entity.id());
                    return Some(entity);
                }
            }
        }

        // TODO Order by priority

        None
    }
}
