use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToDrink,
        common::{GameLog, Named, Position},
        health::Thirst,
        items::Quaffable,
        player::Player,
    },
    utils::roll::Roll,
};

pub struct DrinkingQuaffables {}

impl DrinkingQuaffables {
    pub fn run(ecs_world: &mut World) {
        let mut drinker_list: Vec<Entity> = Vec::new();
        let mut drunk_list: Vec<Entity> = Vec::new();
        let player_id = Player::get_entity_id(ecs_world);

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
                // Keep track of the drinker
                drinker_list.push(drinker);
                //Drink!
                drunk_list.push(wants_to_drink.item);

                let quaffable_thirst = ecs_world.get::<&Quaffable>(wants_to_drink.item).expect("Entity is not Quaffable");

                // Show appropriate log messages
                let named_quaffable = ecs_world.get::<&Named>(wants_to_drink.item).expect("Entity is not Named");

                if drinker.id() == player_id {
                    game_log
                        .entries
                        .push(format!("You drank the {}", named_quaffable.name));
                } else {
                    let named_drinker = ecs_world.get::<&Named>(drinker).expect("Entity is not Named");
                    game_log.entries.push(format!(
                        "{} drank the {}",
                        named_drinker.name, named_quaffable.name
                    ));
                }

                //-----------
                // TODO place here something to handle tainted drinks
                //-----------

                thirst.tick_counter += Roll::dice(
                    quaffable_thirst.thirst_dice_number,
                    quaffable_thirst.thirst_dice_size,
                );
            }
        }

        for drunk in drunk_list {
            // Despawn item from World
            let _ = ecs_world.despawn(drunk);
        }

        for drinker in drinker_list {
            // Remove owner's will to drink
            let _ = ecs_world.remove_one::<WantsToDrink>(drinker);
            
            if player_id == drinker.id() {
                Player::wait_after_action(ecs_world);
            }
        }
    }
}
