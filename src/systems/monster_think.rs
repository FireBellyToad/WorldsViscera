use crate::components::actions::WantsToDig;
use crate::components::combat::GazeAttack;
use crate::components::combat::IsHidden;
use crate::components::combat::WantsToCast;
use crate::components::combat::WantsToGaze;
use crate::components::combat::WantsToShoot;
use crate::components::items::Equippable;
use crate::components::items::Equipped;
use crate::components::monster::Prey;
use crate::components::monster::StoneEater;
use crate::constants::MAP_HEIGHT;
use crate::constants::MAP_WIDTH;
use crate::constants::MAX_PRIORITIES_NUMBER;
use crate::engine::state::GameState;
use crate::utils::common::EdibleInBackpack;
use crate::utils::roll::Roll;
use std::collections::HashSet;

use hecs::{Entity, World};

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
    Invoke,
    Cast,
    Dig,
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
    can_cast: bool,
    can_eat_stone: bool,
    has_dig_tool: bool,
}

type MonsterTargetPick = (MonsterAction, Option<Entity>, i32, i32);

/// Monster Think struct
pub struct MonsterThink {}

impl MonsterThink {
    /// Monster acting function
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let mut approacher_list: Vec<(Entity, i32, i32, u32)> = Vec::new();
        let mut pickup_list: Vec<(Entity, Entity)> = Vec::new();
        let mut attacker_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut caster_list: Vec<(Entity, Entity, i32, i32)> = Vec::new();
        let mut zapper_list: Vec<(Entity, Entity, i32, i32)> = Vec::new();
        let mut shooter_list: Vec<(Entity, Entity, i32, i32)> = Vec::new();
        let mut eat_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut equipper_item_list: Vec<(Entity, Entity)> = Vec::new();
        let mut gaze_at_target_list: Vec<(Entity, Entity)> = Vec::new();
        let mut dig_target_list: Vec<(Entity, Entity, Option<Entity>)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut all_monsters = ecs_world
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
                    Option<&Prey>,
                    Option<&GazeAttack>,
                    Option<&StoneEater>,
                )>()
                .with::<(&Monster, &MyTurn)>();

            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");

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
                    gaze_attack_opt,
                    stone_eater_opt,
                ),
            ) in &mut all_monsters
            {
                let mut items_in_backpacks_query = ecs_world.query::<ItemsInBackpack>();
                let items_in_backpacks: Vec<(Entity, ItemsInBackpack)> =
                    items_in_backpacks_query.iter().collect();

                let mut edible_in_backpacks_query = ecs_world.query::<EdibleInBackpack>();
                let edible_in_backpacks: Vec<(Entity, EdibleInBackpack)> =
                    edible_in_backpacks_query.iter().collect();

                let mut has_equipped_item_in_backpack = false;
                let mut has_eaten_edible_in_backpack = false;
                if smart.is_some() {
                    // if smart, try to equip potential items
                    has_equipped_item_in_backpack = MonsterThink::handle_npc_equipment(
                        &mut equipper_item_list,
                        monster,
                        &items_in_backpacks,
                    );

                    // If smart, can eat something from backpack when not satiated
                    has_eaten_edible_in_backpack = MonsterThink::handle_edibles_in_backpack(
                        &mut eat_target_list,
                        monster,
                        hunger,
                        edible_in_backpacks,
                    );
                }

                // if has not equipped an item or eaten, try to cast a spell or invoke an item
                if !has_equipped_item_in_backpack && !has_eaten_edible_in_backpack {
                    let total_items = items_in_backpacks.iter().len();

                    // Get castable spells
                    let castable_spells_list =
                        MonsterThink::get_castable_spells(ecs_world, monster);

                    // Get invokables
                    let invokables: Vec<&(Entity, ItemsInBackpack)> = items_in_backpacks
                        .iter()
                        .filter(|(_, (_, in_backpack, invokable, ..))| {
                            in_backpack.owner.id() == monster.id() && invokable.is_some()
                        })
                        .collect();

                    let dig_tool: Vec<&(Entity, ItemsInBackpack)> = items_in_backpacks
                        .iter()
                        .filter(|(_, (_, in_backpack, .., dig_tool))| {
                            in_backpack.owner.id() == monster.id() && dig_tool.is_some()
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
                            can_cast: !castable_spells_list.is_empty(),
                            can_eat_stone: stone_eater_opt.is_some(),
                            has_dig_tool: !dig_tool.is_empty(),
                        },
                    );

                    // If enemy can see target, do action relative to it
                    let (action, target, target_x, target_y) = target_picked;

                    // Gaze attacks are directed to the target the monster is looking at
                    // These attacks are nasty because they are additional attacks in that turn
                    if gaze_attack_opt.is_some()
                        && let Some(t) = target
                    {
                        gaze_at_target_list.push((monster, t));
                    }

                    // Active action
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
                                // No target in sight, wander around for a while (if not already doing so)
                                // clamped inside map
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
                        MonsterAction::Dig => {
                            if let Some(t) = target {
                                // if has dig tool, use it; otherwise use his own attack (so insert None)
                                if dig_tool.is_empty() {
                                    dig_target_list.push((monster, t, None));
                                } else {
                                    dig_target_list.push((monster, t, Some(dig_tool[0].0)));
                                }
                            }
                        }
                        MonsterAction::Attack => {
                            if let Some(t) = target {
                                attacker_target_list.push((monster, t));
                            }
                        }
                        MonsterAction::Invoke => {
                            if target.is_some() {
                                zapper_list.push((monster, invokables[0].0, target_x, target_y));
                            }
                        }
                        MonsterAction::Cast => {
                            if target.is_some()
                                && let Some(&spell_to_cast) = castable_spells_list.first()
                            {
                                caster_list.push((monster, spell_to_cast, target_x, target_y));
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

        // Gaze at target
        for (gazer, target) in gaze_at_target_list {
            let _ = ecs_world.insert_one(gazer, WantsToGaze { target });
        }

        // Attack if needed
        for (attacker, target) in attacker_target_list {
            let _ = ecs_world.insert_one(attacker, WantsToMelee { target });
        }

        // eat if needed
        for (eater, item) in eat_target_list {
            let _ = ecs_world.insert_one(eater, WantsToEat { item });
        }

        // dig if needed
        for (digger, target, dig_tool_opt) in dig_target_list {
            if let Some(dig_tool) = dig_tool_opt {
                let _ = ecs_world.insert_one(
                    digger,
                    WantsToDig {
                        target,
                        tool: dig_tool,
                    },
                );
            } else {
                let _ = ecs_world.insert_one(
                    digger,
                    WantsToDig {
                        target,
                        tool: digger,
                    },
                );
            }
        }

        // pick up item
        for (pickupper, item) in pickup_list {
            let _ = ecs_world.insert_one(
                pickupper,
                WantsItem {
                    items: vec![item],
                    was_bought: false,
                },
            );
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

        // Cast spell
        for (caster, spell, x, y) in caster_list {
            let _ = ecs_world.insert(
                caster,
                (WantsToCast { spell }, WantsToZap { target: (x, y) }),
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

    /// Check if the monster has nothing that can equip and do it if is the case.
    /// Will also check if the monster has equippables that overlaps on a body location which is already equipped with something else.
    fn handle_npc_equipment(
        equipper_item_list: &mut Vec<(Entity, Entity)>,
        monster: Entity,
        items_of_monster: &Vec<(Entity, ItemsInBackpack)>,
    ) -> bool {
        // What the monster is already equipping
        let equipped: Vec<(&Entity, &Equipped)> = items_of_monster
            .iter()
            .filter_map(
                |(entity, (_, in_backpack, _, _, _, _, _, _, equipped, ..))| {
                    if in_backpack.owner.id() == monster.id() && equipped.is_some() {
                        Some((entity, equipped.expect("Equippable item is missing")))
                    } else {
                        None
                    }
                },
            )
            .collect();

        items_of_monster
            .iter()
            // Get all equippables that are not currently equipped by the monster
            .filter_map(
                |(entity, (_, in_backpack, .., equippable, equipped, _, _))| {
                    if in_backpack.owner.id() == monster.id()
                        && equippable.is_some()
                        && equipped.is_none()
                    {
                        Some((entity, equippable.expect("Equippable item is missing")))
                    } else {
                        None
                    }
                },
            )
            // If has nothing equipped or has at least one item which body location do not overlap
            // with currently equipped items
            .any(|(item_a, equippable_a)| {
                if equipped.is_empty()
                    || equipped.iter().any(|(_, equipped_b)| {
                        !Utils::occupies_same_location(
                            &equippable_a.body_location,
                            &equipped_b.body_location,
                        )
                    })
                {
                    // Equip the first item that does not overlap and leave the loop
                    equipper_item_list.push((monster, *item_a));
                    return true;
                }
                false
            })
    }

    /// pick a target from visible tiles
    fn choose_target_and_action(
        ecs_world: &World,
        monster_dto: MonsterThinkData,
    ) -> MonsterTargetPick {
        // Array to put the targets found in order of priority (0 = top priority, 4 = least priority)
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
                        } else if monster_dto.can_cast {
                            // Cast if far away and has spells. Prey could also shoot predators
                            action = MonsterAction::Cast;
                        } else if monster_dto.is_smart && monster_dto.can_invoke {
                            // Zap if far away and is smart. Prey could also zap predators
                            action = MonsterAction::Invoke;
                        } else if monster_dto.is_smart && monster_dto.can_shoot {
                            // Shoot if far away and is smart. Prey could also shoot predators
                            action = MonsterAction::Shoot;
                        } else {
                            action = MonsterAction::Move;
                        }

                        // Starvation makes the monster behave more aggressively
                        // Should be a cannibal in this state
                        // TODO do not make it suicidial, do level check on target
                        if monster_dto.hunger.current_status == HungerStatus::Starved
                            && targets_vec[0].is_none()
                        {
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
                                if monster_dto.is_prey && targets_vec[0].is_none() {
                                    // If prey but can somehow do a ranged attack, just attack
                                    if action == MonsterAction::Invoke
                                        || action == MonsterAction::Cast
                                        || action == MonsterAction::Shoot
                                    {
                                        targets_vec[1] = Some((action, Some(entity), x, y));
                                    } else {
                                        // Escape the enemy!
                                        let (target_x, target_y) =
                                            Utils::calculate_farthest_visible_point(
                                                &x,
                                                &y,
                                                monster_dto.viewshed,
                                            );
                                        targets_vec[0] =
                                            Some((action, Some(entity), target_x, target_y));
                                    }
                                } else if targets_vec[1].is_none() {
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
                                    if (!monster_dto.is_smart || !is_deadly)
                                        && targets_vec[0].is_none()
                                    {
                                        targets_vec[0] = Some((action, Some(entity), x, y));
                                    }
                                }
                                HungerStatus::Satiated => {
                                    //Do nothing with it
                                    //TODO maybe pick it up for later?
                                }
                                _ => {
                                    if targets_vec[2].is_none() {
                                        targets_vec[2] = Some((action, Some(entity), x, y));
                                    }
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

                                if targets_vec[3].is_none() {
                                    targets_vec[3] = Some((action, Some(entity), x, y));
                                }
                            }
                        }
                    } else if (monster_dto.can_eat_stone || monster_dto.has_dig_tool)
                        && ecs_world.satisfies::<&Diggable>(entity).unwrap_or(false)
                    {
                        // If is StoneEater or has DiggingTool equipped, dig or move towards a Crack wall
                        if distance < NEXT_TO_DISTANCE {
                            action = MonsterAction::Dig;
                        } else {
                            action = MonsterAction::Move
                        }

                        // Since StoneEaters will get to reduce their hunger with digging activities,
                        // let them handle the action with a greater priority (index 3)
                        // Monster with DiggingTool equipped will handle with a lower priority (index 4)
                        if monster_dto.can_eat_stone && targets_vec[3].is_none() {
                            targets_vec[3] = Some((action, Some(entity), x, y));
                        } else if monster_dto.has_dig_tool && targets_vec[4].is_none() {
                            targets_vec[4] = Some((action, Some(entity), x, y));
                        }
                    }
                }
            }
        }

        // return the first valid target by priority
        // .flatten() gets all the Some(_) values, .next() gets the first element
        if let Some(monster_action) = targets_vec.into_iter().flatten().next() {
            return monster_action;
        }

        // No valid target found
        (MonsterAction::Move, None, -1, -1)
    }

    /// Check if the monster can shoot. This is done by checking if the monster has a ranged weapon equipped and has ammo for at least one of them
    fn check_if_can_shoot(
        monster: Entity,
        items_in_backpacks: &Vec<(Entity, ItemsInBackpack)>,
    ) -> (bool, Option<Entity>) {
        // Check if monster has at least one ammo for the equipped ranged weapon
        return items_in_backpacks
            .iter()
            // Find the first equipped ranged weapon with available ammo
            .find_map(
                |(
                    weapon_entity,
                    (_, in_backpack, _, _, _, _, _, _, equipped_opt, ranged_weapon_opt, ..),
                )| {
                    // If the monsters has the ammo for at least one equipped ranged weapon, it can shoot!
                    // Most of the time all ranged weapons occupy BothHands BodyLocation
                    if in_backpack.owner.id() == monster.id()
                        && equipped_opt.is_some()
                        && let Some(ranged_weapon) = ranged_weapon_opt
                        && ranged_weapon.ammo_count_total > 0
                    {
                        return Some((true, Some(*weapon_entity)));
                    } else {
                        return None;
                    }
                },
            )
            .unwrap_or((false, None));
    }

    /// Get monster's castable spells, confronting its spell list and all the spells with cooldown < 1
    fn get_castable_spells(ecs_world: &World, monster: Entity) -> Vec<Entity> {
        let mut ready_spells_query = ecs_world.query::<&Spell>();
        let ready_spells: Vec<Entity> = ready_spells_query
            .iter()
            .filter_map(|(e, s)| if s.spell_cooldown < 1 { Some(e) } else { None })
            .collect();
        //Get monster spell list and filter castable spells
        let mut spells_list_query = ecs_world.query::<&SpellList>();
        let mut castable_spells_list: Vec<Entity> = Vec::new();
        for (e, list) in &mut spells_list_query {
            if e.id() == monster.id() {
                //Get monster spell list and get castable spells in it
                for s in &list.spells {
                    if ready_spells.contains(s) {
                        castable_spells_list.push(*s);
                    }
                }
            }
        }
        castable_spells_list
    }

    /// Handles edible items in the monster's backpack.
    /// If there is at least one edible item in the backpack of a monster, he will eat it when not satiated.
    fn handle_edibles_in_backpack(
        eat_target_list: &mut Vec<(Entity, Entity)>,
        monster: Entity,
        hunger: &Hunger,
        edible_in_backpacks: Vec<(Entity, EdibleInBackpack)>,
    ) -> bool {
        if hunger.current_status != HungerStatus::Satiated {
            edible_in_backpacks.iter().any(|(e, (in_back, _))| {
                if in_back.owner.id() == monster.id() {
                    eat_target_list.push((monster, *e));
                    true
                } else {
                    false
                }
            })
        } else {
            false
        }
    }
}
