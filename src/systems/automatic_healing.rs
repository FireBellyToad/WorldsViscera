use std::cmp::min;

use hecs::World;

use crate::{
    components::{
        combat::CombatStats,
        common::MyTurn,
        health::{CanAutomaticallyHeal, Hunger, Thirst},
    },
    constants::{MAX_STAMINA_HEAL_TICK_COUNTER, MAX_STATS_HEAL_TICK_COUNTER},
    systems::{hunger_check::HungerStatus, thirst_check::ThirstStatus},
};

pub struct AutomaticHealing {}

impl AutomaticHealing {
    pub fn run(ecs_world: &mut World) {
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut statted_entities = ecs_world
                .query::<(
                    &mut CombatStats,
                    &mut CanAutomaticallyHeal,
                    &Hunger,
                    &Thirst,
                )>()
                .with::<&MyTurn>();

            for (_, (stats, auto_heal, hunger, thirst)) in &mut statted_entities {
                // Each 4 ticks, heal 1 STA. if STA is full, each 16 ticks, heal 1 stats point
                if auto_heal.tick_counter == 0 && stats.max_stamina > stats.current_stamina {
                    auto_heal.tick_counter = MAX_STAMINA_HEAL_TICK_COUNTER;
                } else if auto_heal.tick_counter == 0 {
                    auto_heal.tick_counter = MAX_STATS_HEAL_TICK_COUNTER;
                }

                // Cannot heal if starving or dehydrated!
                if auto_heal.tick_counter > 0
                    && hunger.current_status != HungerStatus::Starved
                    && thirst.current_status != ThirstStatus::Dehydrated
                {
                    auto_heal.tick_counter -= 1;

                    if auto_heal.tick_counter == 0 {
                        // Heal 1 STA. If all STA is healed, heals stats
                        if stats.max_stamina > stats.current_stamina {
                            stats.current_stamina =
                                min(stats.max_stamina, stats.current_stamina + 1);
                        } else {
                            if stats.max_toughness > stats.current_toughness {
                                stats.current_toughness =
                                    min(stats.max_toughness, stats.current_toughness + 1);
                            }
                            if stats.max_dexterity > stats.current_dexterity {
                                stats.current_dexterity =
                                    min(stats.max_dexterity, stats.current_dexterity + 1);
                            }
                        }
                    }
                }
            }
        }
    }
}
