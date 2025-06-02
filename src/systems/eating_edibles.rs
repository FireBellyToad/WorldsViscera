
use std::cmp::min;

use hecs::{Entity, World};

use crate::components::{combat::{CombatStats}, 
    common::{GameLog, Named}, items::{WantsToEat,Edible}}
;

pub struct EatingEdibles {}

impl EatingEdibles {
    pub fn run(ecs_world: &mut World) {
        let mut eater_eaten_list: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut eaters = ecs_world.query::<(&WantsToEat, &mut CombatStats)>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (eater, (wants_to_eat, combat_stats)) in &mut eaters {
                // Pick up and keep track of the owner
                eater_eaten_list.push((eater, wants_to_eat.item));

                //TODO must not heal!
                //TODO also check if is really edible
                let edible_nutrition = ecs_world.get::<&Edible>(wants_to_eat.item).unwrap();
                combat_stats.current_stamina = min(combat_stats.max_stamina, combat_stats.current_stamina + edible_nutrition.nutrition_amount);
                
                // Show appropriate log messages
                let named_eater = ecs_world.get::<&Named>(eater).unwrap();
                let named_edible = ecs_world.get::<&Named>(wants_to_eat.item).unwrap();

                game_log.entries.push(format!(
                    "{} eat the {}",
                    named_eater.name, named_edible.name
                ));
            }
        }
        
        for (eater, eaten) in eater_eaten_list {
            // Despawn item from World
            let _ = ecs_world.despawn(eaten);

            // Remove owner's will to pick up
            let _ = ecs_world.remove_one::<WantsToEat>(eater);
        }
    }
}
