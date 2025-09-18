use hecs::{Entity, World};

use crate::{
    components::{
        common::{GameLog, Named, SmellIntensity, Smellable},
        items::{InBackback, Perishable, Unsavoury},
        player::Player,
    },
    constants::STARTING_ROT_COUNTER,
    utils::roll::Roll,
};

pub struct DecayManager {}

impl DecayManager {
    pub fn run(ecs_world: &mut World) {
        let mut expired_edibles: Vec<(Entity,String)> = Vec::new();
        let mut rotten_edibles_to_despawn: Vec<Entity> = Vec::new();
        let player_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            //Log all the drop downs
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            // List of perishable entities
            let mut perishables =
                ecs_world.query::<(&mut Perishable, &Named, Option<&InBackback>)>();

            for (entity, (perishable, named, in_backpack_option)) in &mut perishables {
                perishable.rot_counter -= 1;

                if perishable.rot_counter <= 0 {
                    // Check if something is already rotten (is Unsavoury)
                    match ecs_world.get::<&Unsavoury>(entity) {
                        Ok(_) => {
                            // despawn if rot while already rotten
                            rotten_edibles_to_despawn.push(entity);

                            if let Some(in_backpack) = in_backpack_option
                                && player_id == in_backpack.owner.id()
                            {
                                game_log
                                    .entries
                                    .push(format!("Your {} rots away", named.name));
                            }
                        }
                        Err(_) => {
                            // Rot
                            perishable.rot_counter = STARTING_ROT_COUNTER + Roll::d100();
                            expired_edibles.push((entity,named.name.clone()));
                        }
                    }
                }
            }
        }

        // Register that now edible is rottend
        for (entity,name) in expired_edibles {
            let _ = ecs_world.insert(
                entity,
                (
                    Unsavoury {
                        game_log: String::from("rotten"),
                    },
                    Smellable {
                        intensity: SmellIntensity::Faint,
                        smell_log: format!("rotten {}",name),
                    },
                ),
            );
        }

        // Despawn completely rotted edibles
        for entity in rotten_edibles_to_despawn {
            let _ = ecs_world.despawn(entity);
        }
    }
}
