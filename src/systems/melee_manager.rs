use crate::{
    components::{
        combat::{Grappled, InflictsDamage},
        common::{Immunity, ImmunityTypeEnum},
        health::{Blind, DiseaseType, Diseased},
        monster::{DiseaseBearer, Grappler},
    },
    constants::MAX_DISEASE_TICK_COUNTER,
    engine::state::GameState,
    utils::common::Utils,
};
use std::{
    cmp::max,
    collections::{HashMap, hash_map::Entry},
};

use hecs::Entity;

use crate::{
    components::{
        combat::{CanHide, CombatStats, IsHidden, SufferingDamage, WantsToMelee},
        common::{MyTurn, Named, Position},
        items::{Armor, Equipped, Eroded, MeleeWeapon},
        monster::Venomous,
    },
    constants::MAX_HIDDEN_TURNS,
    maps::zone::Zone,
    utils::roll::Roll,
};

pub struct MeleeManager {}

impl MeleeManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let mut wants_to_melee_list: Vec<(Entity, i32)> = Vec::new();
        let mut hidden_list: Vec<Entity> = Vec::new();
        let mut infected_list: Vec<(Entity, DiseaseType)> = Vec::new();
        let mut grappled_entities: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut attackers = ecs_world
                .query::<(
                    &WantsToMelee,
                    &Named,
                    &Position,
                    &CombatStats,
                    Option<&IsHidden>,
                    Option<&Venomous>,
                    Option<&DiseaseBearer>,
                    Option<&Grappler>,
                )>()
                .with::<&MyTurn>();

            let mut equipped_weapons =
                ecs_world.query::<(&MeleeWeapon, &InflictsDamage, &Equipped, Option<&Eroded>)>();
            let mut equipped_armors = ecs_world.query::<(&Armor, &Equipped, Option<&Eroded>)>();

            //Log all the fights
            // TODO what about unseen fights? Something should be heard by the player

            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");

            for (
                attacker,
                (
                    wants_melee,
                    named_attacker,
                    attacker_position,
                    attacker_stats,
                    hidden_opt,
                    venomous_opt,
                    disease_bearer_opt,
                    grappler_opt,
                ),
            ) in &mut attackers
            {
                let attacker_is_player = attacker.id() == player_id;
                let target_is_player = wants_melee.target.id() == player_id;

                //Sum damage, keeping in mind that could not have SufferingDamage component
                if let Ok(mut target_damage) =
                    ecs_world.get::<&mut SufferingDamage>(wants_melee.target)
                {
                    println!(
                        "---- wants_melee.target Checking entity {:?} components ----",
                        wants_melee.target
                    );
                    let mut target_query = ecs_world
                        .query_one::<(
                            &CombatStats,
                            &Named,
                            Option<&Blind>,
                            Option<&Grappled>,
                            Option<&Immunity>,
                        )>(wants_melee.target)
                        .expect("Must have one");

                    // Show appropriate log messages
                    if let Some((
                        target_stats,
                        named_target,
                        target_blind_opt,
                        target_grappled_opt,
                        target_immunity_opt,
                    )) = target_query.get()
                    {
                        let (attacker_dice_number, attacker_dice, erosion) =
                            MeleeManager::get_damage_dices(
                                attacker_stats.unarmed_attack_dice,
                                attacker.id(),
                                &mut equipped_weapons,
                            );
                        println!(
                            "attacker_dice_number {}, attacker_dice: {:?}, erosion: {:?}",
                            attacker_dice_number, attacker_dice, erosion
                        );

                        let damage_roll: i32;
                        let target_armor = Utils::get_armor_value(
                            target_stats.base_armor,
                            wants_melee.target.id(),
                            &mut equipped_armors,
                        );
                        // Sneak attack doubles damage
                        // Can sneak attack if hidden or target is blind or grappled
                        let is_grappled_by_attacker = target_grappled_opt.is_some()
                            && target_grappled_opt.unwrap().by.id() == attacker.id();

                        //Venomous damage targets toughness ignoring armor
                        match venomous_opt {
                            Some(_) => {
                                // TODO what about venom immunity?
                                let saving_throw_roll = Roll::d20();
                                if saving_throw_roll > attacker_stats.current_toughness {
                                    damage_roll = max(
                                        0,
                                        Roll::dice(attacker_dice_number, attacker_dice) - erosion,
                                    );
                                    target_damage.toughness_damage_received += damage_roll;

                                    if attacker_is_player {
                                        game_state.game_log.entries.push(format!(
                                            "You {} the {} for {} venomous damage",
                                            named_attacker
                                                .attack_verb
                                                .clone()
                                                .expect("attack_verb must not be None "),
                                            named_target.name,
                                            damage_roll
                                        ));
                                    } else if target_is_player {
                                        game_state.game_log.entries.push(format!(
                                            "The {} {} you for {} venomous damage",
                                            named_attacker
                                                .attack_verb
                                                .clone()
                                                .expect("attack_verb must not be None "),
                                            named_attacker.name,
                                            damage_roll
                                        ));
                                    } else {
                                        // Log NPC infighting only if visible
                                        if zone.visible_tiles[Zone::get_index_from_xy(
                                            &attacker_position.x,
                                            &attacker_position.y,
                                        )] {
                                            game_state.game_log.entries.push(format!(
                                                "The {} {} the {} for {} venomous damage",
                                                named_attacker.name,
                                                named_attacker
                                                    .attack_verb
                                                    .clone()
                                                    .expect("attack_verb must not be None "),
                                                named_target.name,
                                                damage_roll
                                            ));
                                        }
                                    }
                                } else if target_is_player {
                                    game_state.game_log.entries.push(
                                        "The hit makes you feel dizzy for a moment, then it passes"
                                            .to_string(),
                                    );
                                }
                            }
                            None => {
                                if is_grappled_by_attacker
                                    || hidden_opt.is_some()
                                    || target_blind_opt.is_some()
                                {
                                    damage_roll = max(
                                        0,
                                        Roll::dice(attacker_dice_number * 2, attacker_dice)
                                            - target_armor
                                            - erosion,
                                    );

                                    if attacker_is_player {
                                        game_state.game_log.entries.push(format!(
                                            "You sneak attack the {} for {} damage!",
                                            named_target.name, damage_roll
                                        ));
                                    } else if target_is_player {
                                        game_state.game_log.entries.push(format!(
                                            "The {} sneak attacks you for {} damage!",
                                            named_attacker.name, damage_roll
                                        ));
                                    } else {
                                        // Log NPC infighting only if visible
                                        if zone.visible_tiles[Zone::get_index_from_xy(
                                            &attacker_position.x,
                                            &attacker_position.y,
                                        )] {
                                            game_state.game_log.entries.push(format!(
                                                "The {} sneak attacks the {} for {} damage!",
                                                named_attacker.name, named_target.name, damage_roll
                                            ));
                                        }
                                    }
                                    if hidden_opt.is_some() {
                                        hidden_list.push(attacker);
                                        // Cannot hide again for 9 - (stats.current_dexterity / 3) turns
                                        let mut can_hide = ecs_world
                                            .get::<&mut CanHide>(attacker)
                                            .expect("Entity does not have CanHide");
                                        can_hide.cooldown = (MAX_HIDDEN_TURNS
                                            - (attacker_stats.current_dexterity / 3))
                                            * attacker_stats.speed;
                                    }
                                } else {
                                    // Standard attack
                                    damage_roll = max(
                                        0,
                                        Roll::dice(1, attacker_dice) - target_armor - erosion,
                                    );
                                    if attacker_is_player {
                                        game_state.game_log.entries.push(format!(
                                            "You {} the {} for {} damage",
                                            named_attacker
                                                .attack_verb
                                                .clone()
                                                .expect("attack_verb must not be None "),
                                            named_target.name,
                                            damage_roll
                                        ));
                                    } else if target_is_player {
                                        game_state.game_log.entries.push(format!(
                                            "The {} {} you for {} damage",
                                            named_attacker.name,
                                            named_attacker
                                                .attack_verb
                                                .clone()
                                                .expect("attack_verb must not be None "),
                                            damage_roll
                                        ));
                                    } else {
                                        // Log NPC infighting only if visible
                                        if zone.visible_tiles[Zone::get_index_from_xy(
                                            &attacker_position.x,
                                            &attacker_position.y,
                                        )] {
                                            game_state.game_log.entries.push(format!(
                                                "{} {} the {} for {} damage",
                                                named_attacker.name,
                                                named_attacker
                                                    .attack_verb
                                                    .clone()
                                                    .expect("attack_verb must not be None "),
                                                named_target.name,
                                                damage_roll
                                            ));
                                        }
                                    }
                                }

                                target_damage.damage_received += damage_roll;
                                target_damage.damager = Some(attacker);
                            }
                        }

                        // If the attacker is a disease bearers
                        if let Some(dis_bear_some) = disease_bearer_opt {
                            let disease_type = dis_bear_some.disease_type.clone();

                            // If not immune and saving throw fails, inflict disease
                            if let Some(target_immunity) = target_immunity_opt
                                && !target_immunity.to.iter().any(|i| match i {
                                    ImmunityTypeEnum::Disease(d) => d == &disease_type,
                                    _ => false,
                                })
                                && Roll::d20() > target_stats.current_toughness
                            {
                                // If the target is already infected...
                                if let Ok(mut dis) =
                                    ecs_world.get::<&mut Diseased>(wants_melee.target)
                                {
                                    match dis.tick_counters.entry(disease_type) {
                                        Entry::Occupied(mut entry) => {
                                            //worsen its status
                                            entry.insert((0, false));
                                        }
                                        Entry::Vacant(entry) => {
                                            // Infect the healthy target otherwise
                                            entry.insert((
                                                MAX_DISEASE_TICK_COUNTER + Roll::d20(),
                                                false,
                                            ));
                                        }
                                    }
                                } else {
                                    // Infect the healthy target otherwise
                                    infected_list.push((wants_melee.target, disease_type));
                                    if player_id == wants_melee.target.id() {
                                        game_state
                                            .game_log
                                            .entries
                                            .push("You start to feel ill.".to_string());
                                    }
                                }
                            } else {
                                // Immune or unaffected
                                if player_id == wants_melee.target.id() {
                                    game_state.game_log.entries.push(
                                        "You felt a little sick, but it passed quickly."
                                            .to_string(),
                                    );
                                }
                            }
                        }

                        if !is_grappled_by_attacker && grappler_opt.is_some() {
                            grappled_entities.push((attacker, wants_melee.target));
                            if Roll::d20() > target_stats.current_dexterity {
                                if target_is_player {
                                    game_state
                                        .game_log
                                        .entries
                                        .push(format!("The {} grabs on you!", named_attacker.name));
                                } else {
                                    game_state.game_log.entries.push(format!(
                                        "The {} grabs on the {}!",
                                        named_attacker.name, named_target.name
                                    ));
                                }
                            }
                        }

                        wants_to_melee_list.push((attacker, attacker_stats.speed));
                    } else {
                        if ecs_world.contains(wants_melee.target) {
                            println!(
                                "---- wants_melee.target {:?} has no CombatStats or Named, wat? ----",
                                wants_melee.target
                            );
                        } else {
                            println!(
                                "---- wants_melee.target {:?} is not in the world, skipping ----",
                                wants_melee.target
                            );
                        }
                    }
                }
            }
        }

        // Remove owner's will to attack
        for (attacker, speed) in wants_to_melee_list {
            let _ = ecs_world.remove_one::<WantsToMelee>(attacker);
            Utils::wait_after_action(ecs_world, attacker, speed);
        }

        // No longer hidden after attack
        for attacker in hidden_list {
            let _ = ecs_world.remove_one::<IsHidden>(attacker);
        }
        // Grappled entities
        for (by, grappled_ent) in grappled_entities {
            let _ = ecs_world.insert_one(grappled_ent, Grappled { by });
        }

        // If the attacker inflicts disease and target fails the saving throws
        // inflict disease
        for (infected, disease_type) in infected_list {
            // Infect the healthy target otherwise
            let mut tick_counters = HashMap::new();
            tick_counters.insert(
                disease_type,
                (MAX_DISEASE_TICK_COUNTER + Roll::d20(), false),
            );
            let _ = ecs_world.insert_one(infected, Diseased { tick_counters });
        }
    }

    // Gets attack dice
    fn get_damage_dices(
        unarmed_attack_dice: i32,
        attacker_id: u32,
        equipped_weapons: &mut hecs::QueryBorrow<
            '_,
            (&MeleeWeapon, &InflictsDamage, &Equipped, Option<&Eroded>),
        >,
    ) -> (i32, i32, i32) {
        // Use weapon dice when equipped (reduced by erosion)
        for (_, (_, inflicts_damage, equipped_to, eroded_opt)) in equipped_weapons.iter() {
            if equipped_to.owner.id() == attacker_id {
                if let Some(erosion) = eroded_opt {
                    return (
                        inflicts_damage.number_of_dices,
                        inflicts_damage.dice_size,
                        erosion.value as i32,
                    );
                } else {
                    return (
                        inflicts_damage.number_of_dices,
                        inflicts_damage.dice_size,
                        0,
                    );
                }
            }
        }
        (1, unarmed_attack_dice, 0)
    }
}
