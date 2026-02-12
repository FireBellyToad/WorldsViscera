use hecs::Entity;

use crate::{
    components::{
        actions::WantsToSmell,
        common::{CanSmell, GameLog, Position, SmellIntensity, Smellable},
    },
    engine::state::GameState,
    maps::zone::Zone,
    utils::common::Utils,
};

pub struct SmellManager {}

impl SmellManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;

        let mut wants_to_smell_list: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to smell things
            let mut smellers = ecs_world.query::<(&WantsToSmell, &CanSmell, &Position)>();

            //Log all the smells
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut zone_query = ecs_world.query::<&Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            for (smeller, (wants_to_smell, smell_ability, smeller_position)) in &mut smellers {
                let index =
                    Zone::get_index_from_xy(&wants_to_smell.target.0, &wants_to_smell.target.1);

                let mut have_smelled_something = false;

                // Targets in water cannot be smelled
                if !zone.water_tiles[index] {
                    let target_list = &zone.tile_content[index];

                    for &target in target_list {
                        let target_smell = ecs_world.get::<&Smellable>(target);

                        if let Ok(smells) = target_smell {
                            // Show appropriate log messages
                            let distance = Utils::distance(
                                &wants_to_smell.target.0,
                                &smeller_position.x,
                                &wants_to_smell.target.1,
                                &smeller_position.y,
                            );

                            let can_smell = smell_ability.intensity != SmellIntensity::None // the player cannot smell anything (common cold or other penalities)
                                                && ((distance < smell_ability.radius / 2.0 && smells.intensity == SmellIntensity::Faint) // Faint odors can be smell from half normal distance
                                                    || (distance < smell_ability.radius
                                                        && (smells.intensity == SmellIntensity::Strong // Strong odors can be smelled at double distance.
                                                            || smell_ability.intensity == SmellIntensity::Strong))); // Player have improved smell (can smell faint odors from far away)

                            if can_smell {
                                have_smelled_something = true;
                                game_log.entries.push(format!(
                                    "You smell {}",
                                    smells
                                        .smell_log
                                        .as_ref()
                                        .expect("must have valid smell log")
                                ));
                            }
                        }
                    }
                }

                if !have_smelled_something {
                    game_log
                        .entries
                        .push("You smell nothing strange".to_string());
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
