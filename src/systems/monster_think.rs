use hecs::{Entity, World};

use crate::{
    components::{
        actions::{WantsItem, WantsToEat},
        combat::{CombatStats, WantsToMelee},
        common::*,
        health::Hunger,
        items::{Deadly, Edible, Item, Unsavoury},
        monster::{Aquatic, IsSmart, Monster, WantsToApproach},
        player::Player,
    },
    maps::zone::Zone,
    systems::hunger_check::HungerStatus,
    utils::{
        common::{EdibleItem, Utils},
        pathfinding::Pathfinding,
    },
};

/// Monster Think struct
pub struct MonsterThink {}

impl MonsterThink {
    /// Monster acting function
    pub fn run(ecs_world: &mut World) {
        let mut approacher_list: Vec<(Entity, Option<Entity>, i32, i32)> = Vec::new();
        let mut pickup_list: Vec<(Entity, Entity)> = Vec::new();
        let mut attacker_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut eat_target_list: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut named_monsters = ecs_world
                .query::<(
                    &mut Viewshed,
                    &mut Position,
                    &Hunger,
                    &Named,
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
            for (monster, (viewshed, position, hunger, named, smart, aquatic)) in
                &mut named_monsters
            {
                let target_picked = MonsterThink::choose_target(
                    ecs_world,
                    zone,
                    viewshed,
                    hunger,
                    named,
                    &monster.id(),
                    &player_id,
                    smart.is_some(),
                );

                //If enemy can see target, do action relative to it
                if let (Some(target), target_x, target_y) = target_picked {
                    // get target position and distance from monster
                    let mut target_query = ecs_world
                        .query_one::<(Option<&CombatStats>, Option<&Item>, Option<&Edible>)>(target)
                        .expect("target_query failed");
                    let (target_has_stats, target_is_item, target_is_edible) = target_query
                        .get()
                        .expect("cannot extract result from target_query");

                    let distance = Utils::distance(position.x, target_x, position.y, target_y);

                    if distance < 1.5 && target_has_stats.is_some() {
                        //Adiacent target, Attack
                        //TODO there should be something else if not hostile
                        attacker_target_list.push((monster, target));
                    } else if distance == 0.0 && target_is_item.is_some() {
                        //Target below
                        if target_is_edible.is_some() {
                            // Is edible, so eat it
                            eat_target_list.push((monster, target));
                        } else if smart.is_some() {
                            //pick up staff if monster is smart enough
                            pickup_list.push((monster, target));
                        }
                    } else {
                        //Target is far away, try to approach it
                        //TODO if hostile and monster has ranged weapon, should attack
                        let pathfinding_result = Pathfinding::dijkstra_wrapper(
                            position.x,
                            position.y,
                            target_x,
                            target_y,
                            zone,
                            true,
                            aquatic.is_some(),
                        );

                        //If can actually reach the position
                        if let Some((path, _)) = pathfinding_result {
                            if path.len() > 1 {
                                // Approach something of its interest. x,y are passed to avoid unique borrow issues later on
                                approacher_list.push((monster, Some(target), target_x, target_y));
                            }
                        } else {
                            //No target in sight, wander around
                            //TODO what about immovable monsters?
                            approacher_list.push((monster, None, 0, 0));
                        }
                    }
                } else {
                    //No target in sight, wander around
                    //TODO what about immovable monsters?
                    approacher_list.push((monster, None, 0, 0));
                }
            }
        }

        // Approach if needed
        for (approacher, target, target_x, target_y) in approacher_list {
            let _ = ecs_world.insert_one(
                approacher,
                WantsToApproach {
                    target,
                    target_x,
                    target_y,
                },
            );
        }

        // Attack if needed
        for (attacker, target) in attacker_target_list {
            let _ = ecs_world.insert_one(attacker, WantsToMelee { target });
        }

        // eat if needed
        for (eater, item) in eat_target_list {
            let _ = ecs_world.insert_one(eater, WantsToEat { item });
        }

        // pick up item
        for (pickupper, item) in pickup_list {
            let _ = ecs_world.insert_one(pickupper, WantsItem { item });
        }
    }

    /// pick a target from visible tiles
    fn choose_target(
        ecs_world: &World,
        zone: &Zone,
        viewshed: &Viewshed,
        hunger: &Hunger,
        named: &Named,
        self_id: &u32,
        player_id: &u32,
        is_smart: bool,
    ) -> (Option<Entity>, i32, i32) {
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

        // Search in range of view possible targets
        for (x, y) in viewshed.visible_tiles.iter() {
            let index = Zone::get_index_from_xy(*x, *y);

            for &entity in &zone.tile_content[index] {
                // If looking at someone else
                if *self_id != entity.id() {
                    let is_creature = ecs_world.satisfies::<&Player>(entity).unwrap_or(false)
                        || ecs_world.satisfies::<&Monster>(entity).unwrap_or(false);

                    if is_creature {
                        //TODO the player should not be the only enemy
                        let is_enemy = *player_id == entity.id();

                        // Starvation makes the monster behave more aggressively
                        // TODO do not make it suicidial, do level check on target
                        // TODO Should be a cannibal in this state?
                        if hunger.current_status == HungerStatus::Starved {
                            println!(
                                "{} Entity {} - {} is prey",
                                named.name,
                                self_id,
                                entity.id()
                            );
                            return (Some(entity), *x, *y);
                        } else if is_enemy {
                            println!(
                                "{} Entity {} - {} is enemy",
                                named.name,
                                self_id,
                                entity.id()
                            );
                            return (Some(entity), *x, *y);
                        }
                    } else if ecs_world.satisfies::<&Item>(entity).unwrap_or(false) {
                        // Is item
                        let is_edible = ecs_world.satisfies::<&Edible>(entity).unwrap_or(false);

                        if is_edible {
                            let is_deadly = ecs_world.satisfies::<&Deadly>(entity).unwrap_or(false);

                            match hunger.current_status {
                                HungerStatus::Starved => {
                                    // If starved and not smart, do stupid stuff like eating deadly food
                                    if (!is_smart && is_deadly) || !is_deadly {
                                        println!(
                                            "{} Entity {} - {} is food",
                                            named.name,
                                            self_id,
                                            entity.id()
                                        );
                                        return (Some(entity), *x, *y);
                                    }
                                }
                                HungerStatus::Satiated => {
                                    //Do nothing with it
                                    //TODO maybe pick it up for later?
                                }
                                _ => return (Some(entity), *x, *y),
                            }
                        } else if is_smart {
                            // TODO should pick it up
                            return (Some(entity), *x, *y);
                        }
                    }
                }
            }
        }

        // TODO Order by priority

        // No valid target found
        println!("{} Entity {} - no target", named.name, self_id);
        (None, 0, 0)
    }
}
