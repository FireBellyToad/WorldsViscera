use std::cmp::min;

use hecs::Entity;

use crate::{
    components::{
        combat::CombatStats,
        common::{GameLog, MyTurn, Named},
        health::{CanAutomaticallyHeal, Hunger, Paralyzed, Thirst},
    },
    constants::{MAX_STAMINA_HEAL_TICK_COUNTER, MAX_STATS_HEAL_TICK_COUNTER},
    engine::state::GameState,
    systems::{hunger_check::HungerStatus, thirst_check::ThirstStatus},
};

/// Handles automatic healing in entities
pub struct AutomaticHealing {}

impl AutomaticHealing {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let mut entities_free: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            // List of entities that has stats
            let mut statted_entities = ecs_world
                .query::<(
                    &mut CombatStats,
                    &mut CanAutomaticallyHeal,
                    &Hunger,
                    &Thirst,
                    &Named,
                    Option<&Paralyzed>,
                )>()
                .with::<&MyTurn>();

            for (entity, (stats, auto_heal, hunger, thirst, named, paralyzed_opt)) in
                &mut statted_entities
            {
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

                                // Remove Paralyzed component if entity's dexterity is healed
                                if paralyzed_opt.is_some() {
                                    entities_free.push(entity);

                                    if entity.id() == player_id {
                                        game_log.entries.push("You can move now".to_string());
                                    } else {
                                        game_log
                                            .entries
                                            .push(format!("{} can move now", named.name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        //Remove Paralyzed component from entities that have been cured
        for entity in entities_free {
            let _ = ecs_world.remove_one::<Paralyzed>(entity);
        }
    }
}
