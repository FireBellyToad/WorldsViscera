use hecs::{Entity, World};

use crate::{
    components::{
        common::{GameLog, Named, Position},
        health::Hunger,
        items::{Edible, Rotten, WantsToEat},
        player::Player,
    },
    maps::zone::{ParticleType, Zone},
    systems::hunger_check::HungerStatus,
    utils::roll::Roll,
};

pub struct EatingEdibles {}

impl EatingEdibles {
    pub fn run(ecs_world: &mut World) {
        let mut eater_list: Vec<Entity> = Vec::new();
        let mut eaten_list: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut eaters = ecs_world.query::<(&WantsToEat, &mut Hunger, &Position)>();

            let player_id = Player::get_entity_id(ecs_world);

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_e, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (eater, (wants_to_eat, hunger, position)) in &mut eaters {
                let possible_edible = ecs_world.get::<&Edible>(wants_to_eat.item);

                // Keep track of the eater
                eater_list.push(eater);
                if possible_edible.is_err() {
                    if eater.id() == player_id {
                        game_log.entries.push(format!("You can't eat that!"));
                    }
                    continue;
                }

                // Eat!
                eaten_list.push(wants_to_eat.item);

                let edible_nutrition = possible_edible.unwrap();

                // Show appropriate log messages
                let named_edible = ecs_world.get::<&Named>(wants_to_eat.item).unwrap();
                if eater.id() == player_id {
                    game_log
                        .entries
                        .push(format!("You ate the {}", named_edible.name));
                } else {
                    let named_eater = ecs_world.get::<&Named>(eater).unwrap();

                    game_log.entries.push(format!(
                        "{} ate the {}",
                        named_eater.name, named_edible.name
                    ));
                }

                // Is it rotten? Then vomit badly
                // TODO vomit MORE BADLY
                let is_rotten = ecs_world.get::<&Rotten>(wants_to_eat.item).is_ok();
                if is_rotten {
                    hunger.tick_counter -= Roll::dice(3, 10);
                    match hunger.current_status {
                        HungerStatus::Satiated => {
                            hunger.current_status = HungerStatus::Normal;
                        }
                        HungerStatus::Normal => {
                            hunger.current_status = HungerStatus::Hungry;
                        }
                        HungerStatus::Hungry => {
                            hunger.current_status = HungerStatus::Starved;
                        }
                        HungerStatus::Starved => {}
                    }

                    if eater.id() == player_id {
                        game_log
                            .entries
                            .push(format!("You ate rotten food! You vomit!"));
                    }

                    zone.particle_tiles.insert(
                        Zone::get_index_from_xy(position.x, position.y),
                        ParticleType::Vomit,
                    );
                } else {
                    hunger.tick_counter += Roll::dice(
                        edible_nutrition.nutrition_dice_number,
                        edible_nutrition.nutrition_dice_size,
                    );
                }
            }
        }

        for eaten in eaten_list {
            // Despawn item from World
            let _ = ecs_world.despawn(eaten);
        }

        for eater in eater_list {
            // Remove owner's will to eat
            let _ = ecs_world.remove_one::<WantsToEat>(eater);
        }
    }
}
