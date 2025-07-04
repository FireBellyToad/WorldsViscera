use hecs::{Entity, World};

use crate::{
    components::items::{Perishable, Rotten},
    constants::STARTING_ROT_COUNTER,
    utils::roll::Roll,
};

pub struct DecayManager {}

impl DecayManager {
    pub fn run(ecs_world: &mut World) {
        let mut expired_edibles: Vec<Entity> = Vec::new();
        let mut rotten_edibles: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to drop items
            let mut perishables = ecs_world.query::<&mut Perishable>();

            for (entity, perishable) in &mut perishables {
                perishable.rot_counter -= 1;
                
                let is_already_rotten = ecs_world.get::<&Rotten>(entity).is_ok();

                // Check if something is already rotten
                if perishable.rot_counter <= 0 {
                    if is_already_rotten {
                        rotten_edibles.push(entity);
                    } else {
                        perishable.rot_counter = STARTING_ROT_COUNTER + Roll::d100();
                        expired_edibles.push(entity);
                    }
                }
            }
        }

        // Register that now edible is rottend
        for entity in expired_edibles {
            let _ = ecs_world.insert_one(entity, Rotten {});
        }

        // Despawn completely rotted edibles
        for entity in rotten_edibles {
            let _ = ecs_world.despawn(entity);
        }
    }
}
