use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CanHide, CombatStats, IsHidden, SufferingDamage, WantsToMelee},
        common::{GameLog, MyTurn, Named},
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

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (attacker, (wants_melee, hidden)) in &mut attackers {
                let target_damage = ecs_world.get::<&mut SufferingDamage>(wants_melee.target);

                //Sum damage, keeping in mind that could not have SufferingDamage component
                if target_damage.is_ok() {
                    let attacker_stats = ecs_world.get::<&CombatStats>(attacker).unwrap();
                    let target_stats = ecs_world.get::<&CombatStats>(wants_melee.target).unwrap();

                    // Show appropriate log messages
                    let named_attacker = ecs_world.get::<&Named>(attacker).unwrap();
                    let named_target = ecs_world.get::<&Named>(wants_melee.target).unwrap();

                    let damage_roll: i32;
                    // Sneak attack doubles damage
                    if hidden.is_some() {
                        damage_roll = max(
                            0,
                            Roll::dice(2, attacker_stats.unarmed_attack_dice)
                                - target_stats.base_armor,
                        );
                        game_log.entries.push(format!(
                            "{} sneak attacks the {} for {} damage!",
                            named_attacker.name, named_target.name, damage_roll
                        ));
                        hidden_list.push(attacker);

                        // Cannot hide again for 9 - (stats.current_dexterity / 3) turns
                        let mut can_hide = ecs_world.get::<&mut CanHide>(attacker).unwrap();
                        can_hide.cooldown = (MAX_HIDDEN_TURNS
                            - (attacker_stats.current_dexterity / 3))
                            * attacker_stats.speed;
                    } else {
                        damage_roll = max(
                            0,
                            Roll::dice(1, attacker_stats.unarmed_attack_dice)
                                - target_stats.base_armor,
                        );
                        game_log.entries.push(format!(
                            "{} hits the {} for {} damage",
                            named_attacker.name, named_target.name, damage_roll
                        ));
                    }

                    target_damage.unwrap().damage_received += damage_roll;

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
}
