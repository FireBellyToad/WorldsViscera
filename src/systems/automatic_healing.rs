use std::cmp::min;

use hecs::World;

use crate::{
    components::{
        combat::CombatStats,
        health::{CanAutomaticallyHeal, Hunger},
    },
    constants::MAX_STAMINA_HEAL_TICK_COUNTER,
    systems::hunger_check::HungerStatus,
};

pub struct AutomaticHealing {}

impl AutomaticHealing {
    pub fn run(ecs_world: &mut World) {
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut statted_entities =
                ecs_world.query::<(&mut CombatStats, &mut CanAutomaticallyHeal, &Hunger)>();

            for (_e, (stats, stamina_heal, hunger)) in &mut statted_entities {
                // Each 4 ticks, heal 1 STA
                if stamina_heal.tick_counter == 0 && stats.max_stamina > stats.current_stamina {
                    stamina_heal.tick_counter = MAX_STAMINA_HEAL_TICK_COUNTER;
                }

                // Cannot heal if starving!
                if stamina_heal.tick_counter > 0 && hunger.current_status != HungerStatus::Starved {
                    stamina_heal.tick_counter -= 1;
                    if stamina_heal.tick_counter == 0 {
                        stats.current_stamina = min(stats.max_stamina, stats.current_stamina + 1);
                    }
                }
            }
        }
    }
}
