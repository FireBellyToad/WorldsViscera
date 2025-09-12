use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CanHide, CombatStats, IsHidden, SufferingDamage, WantsToMelee},
        common::{GameLog, MyTurn, Named},
        items::{Equipped, Weapon},
    },
    constants::MAX_HIDDEN_TURNS,
    utils::roll::Roll,
};

pub struct MeleeManager {}

impl MeleeManager {
    pub fn run(ecs_world: &mut World) {
        let mut wants_to_melee_list: Vec<Entity> = Vec::new();
        let mut hidden_list: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut attackers = ecs_world
                .query::<(&WantsToMelee, Option<&IsHidden>)>()
                .with::<&MyTurn>();
            let mut equipped_weapons = ecs_world.query::<(&Weapon, &Equipped)>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (attacker, (wants_melee, hidden)) in &mut attackers {
                //Sum damage, keeping in mind that could not have SufferingDamage component
                if let Ok(mut target_damage) = ecs_world.get::<&mut SufferingDamage>(wants_melee.target)
                {
                    let attacker_stats = ecs_world
                        .get::<&CombatStats>(attacker)
                        .expect("Entity does not have CombatStats");
                    let target_stats = ecs_world
                        .get::<&CombatStats>(wants_melee.target)
                        .expect("Entity does not have CombatStats");

                    // Show appropriate log messages
                    let named_attacker = ecs_world
                        .get::<&Named>(attacker)
                        .expect("Entity is not Named");
                    let named_target = ecs_world
                        .get::<&Named>(wants_melee.target)
                        .expect("Entity is not Named");
                    let attacker_dice = MeleeManager::get_damage_dice(
                        attacker_stats.unarmed_attack_dice,
                        attacker.id(),
                        &mut equipped_weapons,
                    );

                    let damage_roll: i32;
                    // Sneak attack doubles damage
                    match hidden {
                        Some(_) => {
                            damage_roll =
                                max(0, Roll::dice(2, attacker_dice) - target_stats.base_armor);
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
                            damage_roll =
                                max(0, Roll::dice(1, attacker_dice) - target_stats.base_armor);
                            game_log.entries.push(format!(
                                "{} hits the {} for {} damage",
                                named_attacker.name, named_target.name, damage_roll
                            ));
                        }
                    }

                    target_damage.damage_received += damage_roll;

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
        equipped_weapons: &mut hecs::QueryBorrow<'_, (&Weapon, &Equipped)>,
    ) -> i32 {
        // Use weapon dice when equipped
        for (_, (attacker_weapon, equipped_to)) in equipped_weapons.iter() {
            if equipped_to.owner.id() == attacker_id {
                println!(
                    "Weapon equipped by {:?} is {}",
                    equipped_to.owner, attacker_weapon.attack_dice
                );
                return attacker_weapon.attack_dice;
            }
        }
        unarmed_attack_dice
    }
}
