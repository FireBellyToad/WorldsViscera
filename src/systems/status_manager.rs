use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CanHide, CombatStats, IsHidden},
        common::{GameLog, MyTurn, Named, Position}
    },
    maps::zone::Zone,
    utils::roll::Roll,
};

pub struct StatusManager {}

impl StatusManager {
    pub fn run(ecs_world: &mut World) {
        let mut hidden_entities: Vec<(Entity, i32)> = Vec::new();
        let mut exposed_entities: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of perishable entities
            let mut stealthers = ecs_world.query::<(
                &CanHide,
                &CombatStats,
                &Position,
                &Named,
                Option<&mut IsHidden>,
            )>().with::<&MyTurn>();

            //Log
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_e, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            for (entity, (_c, stats, position, named, hidden)) in &mut stealthers {
                let have_made_dex_saving_throw = Roll::d20() <= stats.current_dexterity;

                if hidden.is_some() {
                    let hidden_component = hidden.unwrap();
                    hidden_component.hidden_counter -= 1;

                    // If cannot be hidden anymore
                    if hidden_component.hidden_counter <= 0 {
                        // Do a dex saving throw
                        if have_made_dex_saving_throw {
                            //Reset timer
                            hidden_component.hidden_counter = stats.current_dexterity / 3;
                        } else {
                            println!("entity {} un-hides", entity.id());

                            // TODO Cannot hide again for 9 -  (stats.current_dexterity / 3) turns

                            // Log if within players view
                            if zone.visible_tiles[Zone::get_index_from_xy(position.x, position.y)] {
                                game_log
                                    .entries
                                    .push(format!("A {} suddenly appears!", named.name));
                            }
                            exposed_entities.push(entity);
                        }
                    }
                } else if have_made_dex_saving_throw {
                    // Just hide if a saving throw has been made
                    hidden_entities.push((entity, stats.current_dexterity / 3));

                    if zone.visible_tiles[Zone::get_index_from_xy(position.x, position.y)] {
                        game_log
                            .entries
                            .push(format!("The {} suddenly disappears!", named.name));
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
