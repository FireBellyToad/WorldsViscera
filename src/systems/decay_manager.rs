use hecs::Entity;

use crate::{
    components::{
        common::{Named, Position, SmellIntensity, Smellable},
        items::{Corpse, InBackback, Perishable, Rotten},
    },
    constants::STARTING_ROT_COUNTER,
    engine::state::GameState,
    spawning::spawner::Spawn,
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
        let mut rotten_edibles_to_despawn: Vec<(Entity, Option<(i32, i32)>)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            //Log all the drop downs

            // List of perishable entities
            let mut perishables = ecs_world.query::<(
                &mut Perishable,
                &Named,
                Option<&Position>,
                Option<&InBackback>,
                Option<&Corpse>,
            )>();

            for (entity, (perishable, named, position_opt, in_backpack_opt, corpse_opt)) in
                &mut perishables
            {
                perishable.rot_counter -= 1;

                if perishable.rot_counter <= 0 {
                    // Check if something is already rotten
                    match ecs_world.get::<&Rotten>(entity) {
                        Ok(_) => {
                            // Will not have a position if in backpack (None),
                            // or else it will have the position of the corpse.
                            // If not a corpse, just set it to None.
                            let must_spawn_mushroom_at_position = if corpse_opt.is_some() {
                                position_opt.map(|p| (p.x, p.y))
                            } else {
                                None
                            };

                            // despawn if rot while already rotten
                            rotten_edibles_to_despawn
                                .push((entity, must_spawn_mushroom_at_position));

                            if let Some(in_backpack) = in_backpack_opt
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
        for (entity, position_opt) in rotten_edibles_to_despawn {
            let _ = ecs_world.despawn(entity);

            // Randomly spawn a mushroom at the rotted corpse's location
            // is Some only if the decayed entiy was a corpse and wa not in backpack
            if Roll::d6() == 6
                && let Some((x, y)) = position_opt
            {
                Spawn::random_mushroom(ecs_world, x, y);
            }
        }
    }
}
