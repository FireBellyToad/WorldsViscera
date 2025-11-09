use hecs::World;

use crate::{
    components::{
        combat::CombatStats,
        common::{Experience, GameLog, Level},
        player::Player,
    },
    constants::AUTO_ADVANCE_EXP_COUNTER_START,
    utils::roll::Roll,
};

pub struct AdvancementSystem {}

/// Level and Experience Advancement System
impl AdvancementSystem {
    pub fn run(ecs_world: &mut World) {
        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut experienced_entities =
                ecs_world.query::<(&mut Level, &mut Experience, &mut CombatStats)>();

            let player_id = Player::get_entity_id(ecs_world);

            //Log all
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (exp_entity, (level, experience, stats)) in &mut experienced_entities {
                // Level advancement
                if experience.value >= ((level.value + 2).pow(3)) {
                    level.value += 1;
                    experience.value = 0;
                    experience.auto_advance_counter = AUTO_ADVANCE_EXP_COUNTER_START;
                    if exp_entity.id() == player_id {
                        game_log
                            .entries
                            .push(format!("You have reached level {}", level.value));
                    }

                    // Increase stats and Stamina
                    // TODO if soldier, increase is 2d3
                    let stamina_increase = Roll::dice(1, 3);
                    stats.max_stamina += stamina_increase;
                    stats.current_stamina += stamina_increase;

                    let new_toughness = Roll::stat() + 1;
                    if new_toughness > stats.max_toughness {
                        stats.max_toughness += 1;
                        stats.current_toughness += 1;
                    }

                    let new_dexterity = Roll::stat() + 1;
                    if new_dexterity > stats.max_dexterity {
                        stats.max_dexterity += 1;
                        stats.current_dexterity += 1;
                    }
                } else if experience.auto_advance_counter == 0 {
                    // Get experience for surviving enough ticks
                    experience.value += 1;
                    experience.auto_advance_counter = AUTO_ADVANCE_EXP_COUNTER_START;
                } else {
                    experience.auto_advance_counter -= 1;
                }
            }
        }
    }
}
