use hecs::Entity;

use crate::{
    components::{
        combat::{CanHide, CombatStats, IsHidden},
        common::{GameLog, MyTurn, Named, Position},
    },
    constants::MAX_HIDDEN_TURNS,
    engine::state::GameState,
    maps::zone::Zone,
    utils::roll::Roll,
};

pub struct HiddenManager {}

impl HiddenManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;

        let mut hidden_entities: Vec<(Entity, i32)> = Vec::new();
        let mut exposed_entities: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of perishable entities
            let mut stealthers = ecs_world
                .query::<(
                    &mut CanHide,
                    &CombatStats,
                    &Position,
                    &Named,
                    Option<&mut IsHidden>,
                )>()
                .with::<&MyTurn>();

            //Log

            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");

            for (entity, (can_hide, stats, position, named, hidden)) in &mut stealthers {
                let have_made_dex_saving_throw = Roll::d20() <= stats.current_dexterity;

                if can_hide.cooldown > 0 {
                    can_hide.cooldown -= 1;
                    println!(
                        "entity {} can_hide.cooldown {}",
                        entity.id(),
                        can_hide.cooldown
                    );
                }

                match hidden {
                    Some(hidden_component) => {
                        hidden_component.hidden_counter -= 1;

                        // If cannot be hidden anymore
                        if hidden_component.hidden_counter <= 0 {
                            // Do a dex saving throw
                            if have_made_dex_saving_throw {
                                //Reset timer
                                hidden_component.hidden_counter =
                                    (stats.current_dexterity / 3) * stats.speed;
                            } else {
                                // Log if within players view
                                if zone.visible_tiles
                                    [Zone::get_index_from_xy(&position.x, &position.y)]
                                {
                                    game_state
                                        .game_log
                                        .entries
                                        .push(format!("A {} suddenly appears!", named.name));
                                }

                                // Cannot hide again for 9 -  (stats.current_dexterity / 3) turns
                                can_hide.cooldown = (MAX_HIDDEN_TURNS
                                    - (stats.current_dexterity / 3))
                                    * stats.speed;
                                exposed_entities.push(entity);
                            }
                        }
                    }
                    None => {
                        // Just hide if a saving throw has been made
                        if have_made_dex_saving_throw && can_hide.cooldown <= 0 {
                            hidden_entities
                                .push((entity, (stats.current_dexterity / 3) * stats.speed));

                            if zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)]
                            {
                                game_state
                                    .game_log
                                    .entries
                                    .push(format!("The {} suddenly disappears!", named.name));
                            }
                        }
                    }
                }
            }
        }

        // Register that now edible is rottend
        for (entity, counter) in hidden_entities {
            let _ = ecs_world.insert_one(
                entity,
                IsHidden {
                    hidden_counter: counter,
                },
            );
        }

        // Despawn completely rotted edibles
        for entity in exposed_entities {
            let _ = ecs_world.remove_one::<IsHidden>(entity);
        }
    }
}
