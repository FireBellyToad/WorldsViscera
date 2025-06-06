use std::cmp::min;

use hecs::World;

use crate::{components::
    combat::{CombatStats, StaminaHeal}, constants::MAX_STAMINA_HEAL_COUNTER}
;

pub struct StaminaHealing {}

impl StaminaHealing {
    pub fn run(ecs_world: &mut World) {
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut statted_entities = ecs_world.query::<(&mut CombatStats, &mut StaminaHeal)>();

            for (_e, (stats, stamina_heal)) in &mut statted_entities {

                // Each 9 ticks, heal 1 STA
                if stamina_heal.counter == 0 && stats.max_stamina > stats.current_stamina {
                    stamina_heal.counter = MAX_STAMINA_HEAL_COUNTER;
                    println!("Starting to heal");
                } else if stamina_heal.counter > 0 {
                    stamina_heal.counter -= 1;
                    println!(" stamina_heal.counter {}", stamina_heal.counter);
                    if stamina_heal.counter == 0 {
                        stats.current_stamina = min(stats.max_stamina, stats.current_stamina + 1);
                    }
                }
            }
        }
    }
}
