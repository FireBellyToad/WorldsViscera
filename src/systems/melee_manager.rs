use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CanHide, CombatStats, IsHidden, SufferingDamage, WantsToMelee},
        common::{GameLog, MyTurn, Named},
        items::{Armor, Equipped, Eroded, Weapon},
        monster::Venomous,
        player::Player,
    },
    constants::MAX_HIDDEN_TURNS,
    utils::roll::Roll,
};

pub struct MeleeManager {}

impl MeleeManager {
    pub fn run(ecs_world: &mut World) {
        let mut wants_to_melee_list: Vec<Entity> = Vec::new();
        let mut hidden_list: Vec<Entity> = Vec::new();
        let player_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut attackers = ecs_world
                .query::<(
                    &WantsToMelee,
                    &Named,
                    &CombatStats,
                    Option<&IsHidden>,
                    Option<&Venomous>,
                )>()
                .with::<&MyTurn>();

            let mut equipped_weapons = ecs_world.query::<(&Weapon, &Equipped, Option<&Eroded>)>();
            let mut equipped_armors = ecs_world.query::<(&Armor, &Equipped, Option<&Eroded>)>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (attacker, (wants_melee, named_attacker, attacker_stats, hidden, venomous)) in
                &mut attackers
            {
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
                    let attacker_dice = MeleeManager::get_damage_dice(
                        attacker_stats.unarmed_attack_dice,
                        attacker.id(),
                        &mut equipped_weapons,
                    );

                    let damage_roll: i32;
                    let target_armor = MeleeManager::get_armor_value(
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
                                damage_roll = max(0, Roll::dice(1, attacker_dice));
                                target_damage.toughness_damage_received += damage_roll;

                                game_log.entries.push(format!(
                                    "{} hits the {} for {} venomous damage",
                                    named_attacker.name, named_target.name, damage_roll
                                ));
                            } else if wants_melee.target.id() == player_id {
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
                                    damage_roll =
                                        max(0, Roll::dice(2, attacker_dice) - target_armor);
                                    game_log.entries.push(format!(
                                        "{} sneak attacks the {} for {} damage!",
                                        named_attacker.name, named_target.name, damage_roll
                                    ));
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
                                        Roll::dice(1, attacker_dice) - target_stats.base_armor,
                                    );
                                    game_log.entries.push(format!(
                                        "{} hits the {} for {} damage",
                                        named_attacker.name, named_target.name, damage_roll
                                    ));
                                }
                            }
                            target_damage.damage_received += damage_roll;
                        }
                    }

                    wants_to_melee_list.push(attacker);
                }
            }
        }

        // Remove owner's will to attack
        for attacker in wants_to_melee_list {
            let _ = ecs_world.remove_one::<WantsToMelee>(attacker);
        }

        //No longer hidden
        for attacker in hidden_list {
            let _ = ecs_world.remove_one::<IsHidden>(attacker);
        }
    }

    // Gets attack dice
    fn get_damage_dice(
        unarmed_attack_dice: i32,
        attacker_id: u32,
        equipped_weapons: &mut hecs::QueryBorrow<'_, (&Weapon, &Equipped, Option<&Eroded>)>,
    ) -> i32 {
        // Use weapon dice when equipped (reduced by erosion)
        for (_, (attacker_weapon, equipped_to, eroded)) in equipped_weapons.iter() {
            if equipped_to.owner.id() == attacker_id {
                if let Some(erosion) = eroded {
                    return max(1, attacker_weapon.attack_dice - erosion.value as i32);
                } else {
                    return attacker_weapon.attack_dice;
                }
            }
        }
        unarmed_attack_dice
    }

    // Gets armor value
    fn get_armor_value(
        base_armor: i32,
        target_id: u32,
        equipped_armors: &mut hecs::QueryBorrow<'_, (&Armor, &Equipped, Option<&Eroded>)>,
    ) -> i32 {
        // Use weapon dice when equipped
        for (_, (attacker_armor, equipped_to, eroded)) in equipped_armors.iter() {
            if equipped_to.owner.id() == target_id {
                if let Some(erosion) = eroded {
                    return max(0, attacker_armor.value - erosion.value as i32);
                } else {
                    return attacker_armor.value;
                }
            }
        }
        base_armor
    }
}
