use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToSmell,
        common::{GameLog, Smellable},
    },
    maps::zone::Zone,
};

pub struct SmellManager {}

impl SmellManager {
    pub fn run(ecs_world: &mut World) {
        let mut wants_to_smell_list: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to smell things
            let mut smellers = ecs_world.query::<&WantsToSmell>();

            //Log all the smells
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut zone_query = ecs_world.query::<&Zone>();
            let (_e, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            for (smeller, wants_to_smell) in &mut smellers {
                let index =
                    Zone::get_index_from_xy(wants_to_smell.target.0, wants_to_smell.target.1);
                let target_list = &zone.tile_content[index];

                for &target in target_list {
                    let target_smell = ecs_world.get::<&Smellable>(target);

                    //Sum damage, keeping in mind that could not have SufferingDamage component
                    if target_smell.is_ok() {
                        // Show appropriate log messages
                        let smells = target_smell.unwrap();
                        game_log
                            .entries
                            .push(format!("You smell {}", smells.smell_log));
                    } else {
                        game_log.entries.push(format!("You smell nothing strange"));
                    }
                }

                // prepare lists for removal
                wants_to_smell_list.push(smeller);
            }
        }

        // Remove owner's will to invoke and zap
        for smeller in wants_to_smell_list {
            let _ = ecs_world.remove_one::<WantsToSmell>(smeller);
        }
    }
}
