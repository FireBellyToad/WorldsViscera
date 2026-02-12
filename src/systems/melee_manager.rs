use crate::{
    components::{
        combat::InflictsDamage,
        health::{DiseaseType, Diseased},
        monster::DiseaseBearer,
    },
    constants::MAX_DISEASE_TICK_COUNTER,
    engine::state::GameState,
    utils::common::Utils,
};
use std::cmp::max;

use hecs::Entity;

use crate::{
    components::{
        combat::{CanHide, CombatStats, IsHidden, SufferingDamage, WantsToMelee},
        common::{GameLog, MyTurn, Named, Position},
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
                )>()
                .with::<&MyTurn>();

            let mut equipped_weapons =
                ecs_world.query::<(&MeleeWeapon, &InflictsDamage, &Equipped, Option<&Eroded>)>();
            let mut equipped_armors = ecs_world.query::<(&Armor, &Equipped, Option<&Eroded>)>();

            //Log all the fights
            // TODO what about unseen fights? Something should be heard by the player
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            for (
                attacker,
                (
                    wants_melee,
                    named_attacker,
                    attacker_position,
                    attacker_stats,
                    hidden,
                    venomous,
                    disease_bearer,
                ),
            ) in &mut attackers
            {
                let attacker_is_player = attacker.id() == player_id;
                let target_is_player = wants_melee.target.id() == player_id;

                //Sum damage, keeping in mind that could not have SufferingDamage component
                if let Ok(mut target_damage) =
                    ecs_world.get::<&mut SufferingDamage>(wants_melee.target)
                {
                    let target_stats = ecs_world
                        .get::<&CombatStats>(wants_melee.target)
                        .expect("Entity does not have CombatStats");

                    // Show appropriate log messages
                    let named_target = ecs_world
                        .get::<&Named>(wants_melee.target)
                        .expect("Entity is not Named");
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

                    //Venomous damage targets toughness ignoring armor
                    match venomous {
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
                                    game_log.entries.push(format!(
                                        "You hit the {} for {} venomous damage",
                                        named_target.name, damage_roll
                                    ));
                                } else if target_is_player {
                                    game_log.entries.push(format!(
                                        "The {} hits you for {} venomous damage",
                                        named_attacker.name, damage_roll
                                    ));
                                } else {
                                    // Log NPC infighting only if visible
                                    if zone.visible_tiles[Zone::get_index_from_xy(
                                        &attacker_position.x,
                                        &attacker_position.y,
                                    )] {
                                        game_log.entries.push(format!(
                                            "The {} hits the {} for {} venomous damage",
                                            named_attacker.name, named_target.name, damage_roll
                                        ));
                                    }
                                }
                            } else if target_is_player {
                                game_log.entries.push(
                                    "The hit makes you feel dizzy for a moment, then it passes"
                                        .to_string(),
                                );
                            }
                        }
                        None => {
                            // Sneak attack doubles damage
                            match hidden {
                                Some(_) => {
                                    damage_roll = max(
                                        0,
                                        Roll::dice(attacker_dice_number * 2, attacker_dice)
                                            - target_armor
                                            - erosion,
                                    );

                                    if attacker_is_player {
                                        game_log.entries.push(format!(
                                            "You sneak attack the {} for {} damage!",
                                            named_target.name, damage_roll
                                        ));
                                    } else if target_is_player {
                                        game_log.entries.push(format!(
                                            "The{} sneak attacks you for {} damage!",
                                            named_attacker.name, damage_roll
                                        ));
                                    } else {
                                        // Log NPC infighting only if visible
                                        if zone.visible_tiles[Zone::get_index_from_xy(
                                            &attacker_position.x,
                                            &attacker_position.y,
                                        )] {
                                            game_log.entries.push(format!(
                                                "The {} sneak attacks the {} for {} damage!",
                                                named_attacker.name, named_target.name, damage_roll
                                            ));
                                        }
                                    }
                                    hidden_list.push(attacker);

                                    // Cannot hide again for 9 - (stats.current_dexterity / 3) turns
                                    let mut can_hide = ecs_world
                                        .get::<&mut CanHide>(attacker)
                                        .expect("Entity does not have CanHide");
                                    can_hide.cooldown = (MAX_HIDDEN_TURNS
                                        - (attacker_stats.current_dexterity / 3))
                                        * attacker_stats.speed;
                                }
                                None => {
                                    // Standard attack
                                    damage_roll = max(
                                        0,
                                        Roll::dice(1, attacker_dice) - target_armor - erosion,
                                    );
                                    if attacker_is_player {
                                        game_log.entries.push(format!(
                                            "You hit the {} for {} damage",
                                            named_target.name, damage_roll
                                        ));
                                    } else if target_is_player {
                                        game_log.entries.push(format!(
                                            "The {} hits you for {} damage",
                                            named_attacker.name, damage_roll
                                        ));
                                    } else {
                                        // Log NPC infighting only if visible
                                        if zone.visible_tiles[Zone::get_index_from_xy(
                                            &attacker_position.x,
                                            &attacker_position.y,
                                        )] {
                                            game_log.entries.push(format!(
                                                "{} hits the {} for {} damage",
                                                named_attacker.name, named_target.name, damage_roll
                                            ));
                                        }
                                    }
                                }
                            }
                            target_damage.damage_received += damage_roll;
                            target_damage.damager = Some(attacker);
                        }
                    }

                    // If the attacker inflicts disease and target fails the saving throws
                    // inflict disease
                    if let Some(dis_bear_some) = disease_bearer
                        && Roll::d20() > target_stats.current_toughness
                    {
                        // If the target is already infected, worsen its status
                        if let Ok(mut disease) = ecs_world.get::<&mut Diseased>(wants_melee.target)
                        {
                            disease.is_improving = false;
                            disease.tick_counter = 0;
                        } else {
                            // Infect the healthy target otherwise
                            infected_list
                                .push((wants_melee.target, dis_bear_some.disease_type.clone()));
                            if player_id == wants_melee.target.id() {
                                game_log
                                    .entries
                                    .push("You start to feel ill...".to_string());
                            }
                        }
                    }

                    wants_to_melee_list.push((attacker, attacker_stats.speed));
                }
            }
        }

        // Remove owner's will to attack
        for (attacker, speed) in wants_to_melee_list {
            let _ = ecs_world.remove_one::<WantsToMelee>(attacker);
            Utils::wait_after_action(ecs_world, attacker, speed);
        }

        //No longer hidden
        for attacker in hidden_list {
            let _ = ecs_world.remove_one::<IsHidden>(attacker);
        }

        // Infect the infected
        for (infected, disease_type) in infected_list {
            let _ = ecs_world.insert_one(
                infected,
                Diseased {
                    tick_counter: MAX_DISEASE_TICK_COUNTER + Roll::d20(),
                    is_improving: false,
                    disease_type,
                },
            );
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
