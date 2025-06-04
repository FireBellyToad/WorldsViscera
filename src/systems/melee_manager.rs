use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage, WantsToMelee},
        common::{GameLog, Named},
    },
    utils::roll::Roll,
};

pub struct MeleeManager {}

impl MeleeManager {
    pub fn run(ecs_world: &mut World) {
        let mut wants_to_melee_list: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut attackers = ecs_world.query::<&WantsToMelee>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (attacker, wants_melee) in &mut attackers {
                let attacker_stats = ecs_world.get::<&CombatStats>(attacker).unwrap();
                let target_stats = ecs_world.get::<&CombatStats>(wants_melee.target).unwrap();
                let target_damage = ecs_world.get::<&mut SufferingDamage>(wants_melee.target);
                
                //Sum damage, keeping in mind that could not have SufferingDamage component
                if target_damage.is_ok() {
                    let damage_roll = max(
                        0,
                        Roll::dice(1, attacker_stats.unarmed_attack_dice) - target_stats.base_armor,
                    );

                    target_damage.unwrap().damage_received += damage_roll;

                    // Show appropriate log messages
                    let named_attacker = ecs_world.get::<&Named>(attacker).unwrap();
                    let named_target = ecs_world.get::<&Named>(wants_melee.target).unwrap();

                    game_log.entries.push(format!(
                        "{} hits the {} for {} damage",
                        named_attacker.name, named_target.name, damage_roll
                    ));

                    wants_to_melee_list.push(attacker);
                }
            }
        }

        // Remove owner's will to attack
        for attacker in wants_to_melee_list {
            let _ = ecs_world.remove_one::<WantsToMelee>(attacker);
        }
    }
}
