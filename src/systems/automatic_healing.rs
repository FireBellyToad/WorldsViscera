use std::cmp::min;

use hecs::World;

use crate::{
    components::{
        combat::CombatStats, common::MyTurn, health::{CanAutomaticallyHeal, Hunger, Thirst}
    },
    constants::MAX_STAMINA_HEAL_TICK_COUNTER,
    systems::{hunger_check::HungerStatus, thirst_check::ThirstStatus},
};

pub struct AutomaticHealing {}

impl AutomaticHealing {
    pub fn run(ecs_world: &mut World) {
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut statted_entities =
                ecs_world.query::<(&mut CombatStats, &mut CanAutomaticallyHeal, &Hunger, &Thirst)>().with::<&MyTurn>();

            for (_e, (stats, stamina_heal, hunger, thirst)) in &mut statted_entities {
                // Each 4 ticks, heal 1 STA
                if stamina_heal.tick_counter == 0 && stats.max_stamina > stats.current_stamina {
                    stamina_heal.tick_counter = MAX_STAMINA_HEAL_TICK_COUNTER;
                }

                // Cannot heal if starving or dehydrated!
                //TODO improve for hungerless or thirstless monsters
                if stamina_heal.tick_counter > 0  && hunger.current_status != HungerStatus::Starved && thirst.current_status != ThirstStatus::Dehydrated {
                    stamina_heal.tick_counter -= 1;
                    if stamina_heal.tick_counter == 0 {
                        stats.current_stamina = min(stats.max_stamina, stats.current_stamina + 1);
                    }
                }
            }
        }
    }
}
