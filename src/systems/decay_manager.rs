use hecs::Entity;

use crate::{
    components::{
        common::{Named, SmellIntensity, Smellable},
        items::{InBackback, Perishable, Rotten},
    },
    constants::STARTING_ROT_COUNTER,
    engine::state::GameState,
    utils::roll::Roll,
};

pub struct DecayManager {}

/// Handles the decay of perishable items in the game world.
impl DecayManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let mut expired_edibles: Vec<(Entity, String)> = Vec::new();
        let mut rotten_edibles_to_despawn: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            //Log all the drop downs

            // List of perishable entities
            let mut perishables =
                ecs_world.query::<(&mut Perishable, &Named, Option<&InBackback>)>();

            for (entity, (perishable, named, in_backpack_option)) in &mut perishables {
                perishable.rot_counter -= 1;

                if perishable.rot_counter <= 0 {
                    // Check if something is already rotten
                    match ecs_world.get::<&Rotten>(entity) {
                        Ok(_) => {
                            // despawn if rot while already rotten
                            rotten_edibles_to_despawn.push(entity);

                            if let Some(in_backpack) = in_backpack_option
                                && player_id == in_backpack.owner.id()
                            {
                                game_state
                                    .game_log
                                    .entries
                                    .push(format!("Your {} rots away", named.name));
                            }
                        }
                        Err(_) => {
                            // Rot
                            perishable.rot_counter = STARTING_ROT_COUNTER + Roll::d100();
                            expired_edibles.push((entity, named.name.clone()));
                        }
                    }
                }
            }
        }

        // Register that now edible is rottend
        for (entity, name) in expired_edibles {
            let _ = ecs_world.insert(
                entity,
                (
                    Rotten {},
                    Smellable {
                        intensity: SmellIntensity::Faint,
                        smell_log: Some(format!("rotten {}", name)),
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
