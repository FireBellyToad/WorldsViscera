use crate::components::combat::IsHidden;
use crate::components::combat::WantsToShoot;
use crate::components::items::Equippable;
use crate::components::items::Equipped;
use crate::components::monster::IsPrey;
use crate::constants::MAP_HEIGHT;
use crate::constants::MAP_WIDTH;
use crate::constants::MAX_PRIORITIES_NUMBER;
use crate::constants::TILE_SIZE;
use crate::constants::UI_BORDER;
use crate::utils::roll::Roll;
use std::collections::HashSet;

use hecs::{Entity, World};
use macroquad::color::YELLOW;
use macroquad::shapes::draw_circle;

use crate::{
    components::{
        actions::{WantsItem, WantsToEat, WantsToEquip, WantsToInvoke},
        combat::{WantsToMelee, WantsToZap},
        common::*,
        health::Hunger,
        items::{Bulky, Deadly, Edible, Item},
        monster::{Aquatic, Monster, Small, Smart, WantsToApproach},
        player::Player,
    },
    constants::{
        MAX_ITEMS_IN_BACKPACK, MAX_ITEMS_IN_BACKPACK_FOR_SMALL, NEXT_TO_DISTANCE, ON_TOP_DISTANCE,
    },
    maps::zone::Zone,
    systems::hunger_check::HungerStatus,
    utils::{
        common::{ItemsInBackpack, Utils},
        pathfinding::Pathfinding,
    },
};

#[derive(PartialEq, Clone)]
pub enum MonsterAction {
    Move,
    Eat,
    Attack,
    Shoot,
    PickUp,
    Zap,
}

struct MonsterThinkData<'a> {
    pub position: &'a Position,
    pub zone: &'a Zone,
    pub viewshed: &'a Viewshed,
    pub hunger: &'a Hunger,
    pub self_id: &'a u32,
    pub _player_id: &'a u32,
    pub is_smart: bool,
    pub is_small: bool,
    pub can_invoke: bool,
    pub can_shoot: bool,
    pub species: &'a SpeciesEnum,
    pub hates: &'a HashSet<u32>,
    pub backpack_is_not_full: bool,
    pub is_prey: bool,
}

type MonsterTargetPick = (MonsterAction, Option<Entity>, i32, i32);

/// Monster Think struct
pub struct MonsterThink {}

impl MonsterThink {
    /// Monster acting function
    pub fn run(ecs_world: &mut World) {
        let mut approacher_list: Vec<(Entity, i32, i32, u32)> = Vec::new();
        let mut pickup_list: Vec<(Entity, Entity)> = Vec::new();
        let mut attacker_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut zapper_list: Vec<(Entity, Entity, i32, i32)> = Vec::new();
        let mut shooter_list: Vec<(Entity, Entity, i32, i32)> = Vec::new();
        let mut eat_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut equipper_item_list: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut named_monsters = ecs_world
                .query::<(
                    &Viewshed,
                    &Position,
                    &Species,
                    &Hates,
                    &Hunger,
                    Option<&Small>,
                    Option<&Smart>,
                    Option<&Aquatic>,
                    Option<&WantsToApproach>,
                    Option<&IsPrey>,
                )>()
                .with::<(&Monster, &MyTurn)>();

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            let player_id = Player::get_entity_id(ecs_world);

            // For each viewshed position monster component join
            for (
                monster,
                (
                    viewshed,
                    position,
                    species,
                    hates,
                    hunger,
                    small,
                    smart,
                    aquatic,
                    wants_to_approach,
                    is_prey,
                ),
            ) in &mut named_monsters
            {
                let mut items_in_backpacks_query = ecs_world.query::<ItemsInBackpack>();
                let items_in_backpacks: Vec<(Entity, ItemsInBackpack)> =
                    items_in_backpacks_query.iter().collect();

                // if smart, try to equip potential items
                let mut item_to_equip_found = false;
                if smart.is_some() {
                    item_to_equip_found = match MonsterThink::has_nothing_to_equip(
                        &mut equipper_item_list,
                        monster,
                        &items_in_backpacks,
                    ) {
                        Some(result) => result,
                        None => continue,
                    };
                }

                if !item_to_equip_found {
                    let total_items = items_in_backpacks.iter().len();

                    let invokables: Vec<&(Entity, ItemsInBackpack)> = items_in_backpacks
                        .iter()
                        .filter(|(_, (_, in_backpack, invokable, _, _, _, _, _, _, _))| {
                            in_backpack.owner.id() == monster.id() && invokable.is_some()
                        })
                        .collect();

                    // Get all monster's equipped ranged weapons
                    let (can_shoot, equipped_ranged_weapon) =
                        MonsterThink::check_if_can_shoot(monster, &items_in_backpacks);

                    // Look around for a target and decide what to do
                    let target_picked = MonsterThink::choose_target_and_action(
                        ecs_world,
                        MonsterThinkData {
                            position,
                            zone,
                            viewshed,
                            hunger,
                            self_id: &monster.id(),
                            _player_id: &player_id,
                            is_smart: smart.is_some(),
                            is_small: small.is_some(),
                            can_invoke: !invokables.is_empty(),
                            can_shoot,
                            species: &species.value,
                            hates: &hates.list,
                            backpack_is_not_full: (small.is_none()
                                && total_items < MAX_ITEMS_IN_BACKPACK)
                                || (small.is_some()
                                    && total_items < MAX_ITEMS_IN_BACKPACK_FOR_SMALL),
                            is_prey: is_prey.is_some(),
                        },
                    );

                    //If enemy can see target, do action relative to it
                    let (action, target, target_x, target_y) = target_picked;
                    match action {
                        MonsterAction::Move => {
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
                            if let Some((path, _)) = pathfinding_result
                                && path.len() > 1
                                && target.is_some()
                            {
                                // Approach something of its interest. x,y are passed to avoid unique borrow issues later on
                                approacher_list.push((monster, target_x, target_y, 0));
                            } else if wants_to_approach.is_none() {
                                //  No target in sight, wander around for a while (if not already doing so)
                                // clamped inside map
                                //TODO what about immovable monsters?
                                let random_dest_x =
                                    (Roll::d6() - Roll::d6() + position.x).clamp(1, MAP_WIDTH - 1);
                                let random_dest_y =
                                    (Roll::d6() - Roll::d6() + position.y).clamp(1, MAP_HEIGHT - 1);
                                approacher_list.push((monster, random_dest_x, random_dest_y, 3));
                            }
                        }
                        MonsterAction::Eat => {
                            if let Some(t) = target {
                                eat_target_list.push((monster, t));
                            }
                        }
                        MonsterAction::Attack => {
                            if let Some(t) = target {
                                attacker_target_list.push((monster, t));
                            }
                        }
                        MonsterAction::Zap => {
                            if target.is_some() {
                                zapper_list.push((monster, invokables[0].0, target_x, target_y));
                            }
                        }
                        MonsterAction::Shoot => {
                            if target.is_some() {
                                shooter_list.push((
                                    monster,
                                    equipped_ranged_weapon
                                        .expect("Must have some Equipped Ranged Weapon!"),
                                    target_x,
                                    target_y,
                                ));
                            }
                        }
                        MonsterAction::PickUp => {
                            if let Some(t) = target {
                                pickup_list.push((monster, t));
                            }
                        }
                    }
                }
            }
        }

        // Approach if needed
        for (approacher, target_x, target_y, counter) in approacher_list {
            let _ = ecs_world.insert_one(
                approacher,
                WantsToApproach {
                    target_x,
                    target_y,
                    counter,
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

        // Zap place
        for (zapper, item, x, y) in zapper_list {
            let _ = ecs_world.insert(
                zapper,
                (WantsToInvoke { item }, WantsToZap { target: (x, y) }),
            );
        }

        // Shoot place
        for (shooter, weapon, x, y) in shooter_list {
            let _ = ecs_world.insert(
                shooter,
                (WantsToShoot { weapon }, WantsToZap { target: (x, y) }),
            );
        }

        // Equip item
        for (equipper, item) in equipper_item_list {
            let body_location;
            // Scope to keep the borrow check quiet
            {
                let equippable = ecs_world
                    .get::<&Equippable>(item)
                    .expect("Should be Equippable!");
                body_location = equippable.body_location.clone();
            }
            let _ = ecs_world.insert_one(
                equipper,
                WantsToEquip {
                    item,
                    body_location,
                },
            );
        }
    }

    /// Check if the monster has nothing that can equip.
    /// Will also check if the monster has equippables that overlaps on a body location which is already equipped with something else
    fn has_nothing_to_equip(
        equipper_item_list: &mut Vec<(Entity, Entity)>,
        monster: Entity,
        items_of_monster: &Vec<(Entity, ItemsInBackpack)>,
    ) -> Option<bool> {
        // All equippables that are not equipped by the monster
        let equippables: Vec<(&Entity, &Equippable)> = items_of_monster
            .iter()
            .filter(
                |(_, (_, in_backpack, _, _, _, _, _, equippable, equipped, _))| {
                    in_backpack.owner.id() == monster.id()
                        && equippable.is_some()
                        && equipped.is_none()
                },
            )
            .map(|(entity, (_, _, _, _, _, _, _, equippable, _, _))| {
                (entity, equippable.expect("Equippable item is missing"))
            })
            .collect();

        // All equippables that are equipped by the monster
        let equipped: Vec<(&Entity, &Equipped)> = items_of_monster
            .iter()
            .filter(|(_, (_, in_backpack, _, _, _, _, _, _, equipped, _))| {
                in_backpack.owner.id() == monster.id() && equipped.is_some()
            })
            .map(|(entity, (_, _, _, _, _, _, _, _, equipped, _))| {
                (entity, equipped.expect("Equippable item is missing"))
            })
            .collect();
        let mut item_to_equip_found = !equippables.is_empty();
        if !equippables.is_empty() {
            for (item_a, equippable_a) in equippables {
                // If has nothing equipped or has at least one item which body location do not overlap
                // with currently equipped items
                item_to_equip_found = equipped.is_empty()
                    || equipped.iter().any(|(_, equipped_b)| {
                        !Utils::occupies_same_location(
                            &equippable_a.body_location,
                            &equipped_b.body_location,
                        )
                    });

                if item_to_equip_found {
                    // Equip the first item that does not overlap
                    equipper_item_list.push((monster, *item_a));
                    break;
                }
            }

            if item_to_equip_found {
                // If something is equipped, continue to the next monster
                return None;
            }
        }
        Some(item_to_equip_found)
    }

    /// pick a target from visible tiles
    fn choose_target_and_action(
        ecs_world: &World,
        monster_dto: MonsterThinkData,
    ) -> MonsterTargetPick {
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
        // Array to put the targets found in order of priority (0 = top priority, 9 = least priority)
        let mut targets_vec: Vec<Option<MonsterTargetPick>> = vec![None; MAX_PRIORITIES_NUMBER];

        // Search in range of view possible targets
        for &index in monster_dto.viewshed.visible_tiles.iter() {
            let (x, y) = Zone::get_xy_from_index(index);

            let distance: f32 =
                Utils::distance(&monster_dto.position.x, &x, &monster_dto.position.y, &y);

            let mut action: MonsterAction;
            for &entity in &monster_dto.zone.tile_content[index] {
                // If looking at someone else
                if *monster_dto.self_id != entity.id() {
                    let is_creature = ecs_world.satisfies::<&Player>(entity).unwrap_or(false)
                        || ecs_world.satisfies::<&Monster>(entity).unwrap_or(false);

                    // If looking at a creature that is not hidden
                    if is_creature && !ecs_world.satisfies::<&IsHidden>(entity).unwrap_or(false) {
                        // Attack if next to it and is not prey
                        if distance < NEXT_TO_DISTANCE && !monster_dto.is_prey {
                            action = MonsterAction::Attack;
                        } else if monster_dto.is_smart && monster_dto.can_invoke {
                            // Zap if far away and is smart. Prey could also zap predators
                            action = MonsterAction::Zap;
                        } else if monster_dto.is_smart && monster_dto.can_shoot {
                            // Shoot if far away and is smart. Prey could also shoot predators
                            action = MonsterAction::Shoot;
                        } else {
                            action = MonsterAction::Move;
                        }

                        // Starvation makes the monster behave more aggressively
                        // Should be a cannibal in this state
                        // TODO do not make it suicidial, do level check on target
                        if monster_dto.hunger.current_status == HungerStatus::Starved {
                            targets_vec[0] = Some((action, Some(entity), x, y));
                        } else {
                            let target_species = ecs_world
                                .get::<&Species>(entity)
                                .expect("must have Species");

                            let is_enemy = Utils::what_hates(monster_dto.species)
                                .contains(&target_species.value)
                                || monster_dto.hates.contains(&entity.id());

                            if is_enemy {
                                //Enemy target is far away, try to approach it. Unless it's prey, than it should escape
                                if monster_dto.is_prey {
                                    let (target_x, target_y) =
                                        Utils::calculate_farthest_visible_point(
                                            &x,
                                            &y,
                                            monster_dto.viewshed,
                                        );
                                    targets_vec[0] =
                                        Some((action, Some(entity), target_x, target_y));
                                } else {
                                    targets_vec[1] = Some((action, Some(entity), x, y));
                                }
                            }
                        }
                    } else if ecs_world.satisfies::<&Item>(entity).unwrap_or(false) {
                        // Is item
                        let is_edible = ecs_world.satisfies::<&Edible>(entity).unwrap_or(false);
                        let is_bulky = ecs_world.satisfies::<&Bulky>(entity).unwrap_or(false);

                        if is_edible {
                            let is_deadly = ecs_world.satisfies::<&Deadly>(entity).unwrap_or(false);

                            if distance == ON_TOP_DISTANCE {
                                action = MonsterAction::Eat;
                            } else {
                                action = MonsterAction::Move
                            }

                            match monster_dto.hunger.current_status {
                                HungerStatus::Starved => {
                                    // If starved and not smart, do stupid stuff like eating deadly food
                                    if !monster_dto.is_smart || !is_deadly {
                                        targets_vec[0] = Some((action, Some(entity), x, y));
                                    }
                                }
                                HungerStatus::Satiated => {
                                    //Do nothing with it
                                    //TODO maybe pick it up for later?
                                }
                                _ => {
                                    targets_vec[3] = Some((action, Some(entity), x, y));
                                }
                            }
                        } else if monster_dto.is_smart && monster_dto.backpack_is_not_full {
                            // If item is bulky, small monsters will not pick it up
                            if !is_bulky || !monster_dto.is_small {
                                // Should pick it up if smart enough
                                if distance == ON_TOP_DISTANCE {
                                    action = MonsterAction::PickUp;
                                } else {
                                    action = MonsterAction::Move;
                                }
                                targets_vec[4] = Some((action, Some(entity), x, y));
                            }
                        }
                    }
                }
            }
        }

        // return the first valid target by priority
        for monster_action_opt in targets_vec {
            if let Some(action) = monster_action_opt {
                return action;
            }
        }

        // No valid target found
        (MonsterAction::Move, None, -1, -1)
    }

    /// Check if the monster can shoot. This is done by checking if the monster has a ranged weapon equipped and has ammo for at least one of them
    fn check_if_can_shoot(
        monster: Entity,
        items_in_backpacks: &Vec<(Entity, ItemsInBackpack)>,
    ) -> (bool, Option<Entity>) {
        let equipped_ranged_weapons: Vec<&(Entity, ItemsInBackpack)> = items_in_backpacks
            .iter()
            .filter(
                |(_, (_, in_backpack, _, _, _, _, _, _, equipped, ranged))| {
                    in_backpack.owner.id() == monster.id() && equipped.is_some() && ranged.is_some()
                },
            )
            .collect();

        // Has no equipped ranged weapons, cannot shoot
        if equipped_ranged_weapons.is_empty() {
            return (false, None);
        }

        // Check if monster has at least one ammo for the equipped ranged weapon
        for (weapon_entity, (_, _, _, _, _, _, _, _, _, ranged_weapon_opt)) in
            &equipped_ranged_weapons
        {
            // If the monsters has the ammo for at least one equipped ranged weapon, it can shoot!
            // Most of the time all ranged weapons occupy BothHands BodyLocation
            if let Some(ranged_weapon) = ranged_weapon_opt
                && ranged_weapon.ammo_count_total > 0
            {
                return (true, Some(*weapon_entity));
            }
        }

        (false, None)
    }
}
