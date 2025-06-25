use hecs::{Entity, World};

use crate::{
    components::{
        common::{GameLog, Named, Position},
        health::Thirst,
        items::{Quaffable, WantsToDrink},
    },
    utils::roll::Roll,
};

pub struct DrinkingQuaffables {}

impl DrinkingQuaffables {
    pub fn run(ecs_world: &mut World) {
        let mut drinker_drunk_list: Vec<(Entity, Entity)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut drinkers = ecs_world.query::<(&WantsToDrink, &mut Thirst, &Position)>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (drinker, (wants_to_drink, thirst, _p)) in &mut drinkers {
                // Pick up and keep track of the owner
                drinker_drunk_list.push((drinker, wants_to_drink.item));

                let quaffable_thirst = ecs_world.get::<&Quaffable>(wants_to_drink.item).unwrap();

                // Show appropriate log messages
                let named_eater = ecs_world.get::<&Named>(drinker).unwrap();
                let named_edible = ecs_world.get::<&Named>(wants_to_drink.item).unwrap();

                game_log.entries.push(format!(
                    "{} drank the {}",
                    named_eater.name, named_edible.name
                ));

                //----------- 
                // TODO place here something to handle tainted drinks
                //----------- 

                thirst.tick_counter += Roll::dice(
                    quaffable_thirst.thirst_dice_number,
                    quaffable_thirst.thirst_dice_size,
                );
            }
        }

        for (eater, eaten) in drinker_drunk_list {
            // Despawn item from World
            let _ = ecs_world.despawn(eaten);

            // Remove owner's will to drink
            let _ = ecs_world.remove_one::<WantsToDrink>(eater);
        }
    }
}
