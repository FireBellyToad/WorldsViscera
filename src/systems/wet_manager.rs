use hecs::{Entity, World};

use crate::{
    components::{
        common::{GameLog, Named, Position, Wet},
        items::{InBackback, MustBeFueled, TurnedOff, TurnedOn},
        player::Player,
    },
    constants::STARTING_WET_COUNTER,
    maps::zone::Zone,
};

pub struct WetManager {}

impl WetManager {
    pub fn run(ecs_world: &mut World) {
        let mut entities_that_got_wet: Vec<Entity> = Vec::new();
        let mut entities_in_backpack_to_turn_off: Vec<Entity> = Vec::new();
        let mut entities_that_dryed: Vec<Entity> = Vec::new();

        let player_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            // List of entities that want to drop items
            let mut wettable_entities = ecs_world.query::<(Option<&Position>, Option<&mut Wet>)>();

            for (entity, (position_opt, is_wet)) in &mut wettable_entities {
                // Wet everyone walking in water
                if let Some(position) = position_opt
                    && zone.water_tiles[Zone::get_index_from_xy(position.x, position.y)]
                {
                    if let Some(is_wet_component) = is_wet {
                        is_wet_component.tick_countdown = STARTING_WET_COUNTER;
                    } else {
                        // Log only the first time the player gets wet
                        // Avoid multiple logs while walking in water
                        if player_id == entity.id() {
                            game_log.entries.push("You get wet".to_string());
                        }

                        entities_that_got_wet.push(entity);

                        let mut items_of_wet_entity = ecs_world.query::<(
                            &Named,
                            &InBackback,
                            Option<&TurnedOn>,
                            Option<&MustBeFueled>,
                        )>();

                        let items_to_wet: Vec<(
                            Entity,
                            (
                                &Named,
                                &InBackback,
                                Option<&TurnedOn>,
                                Option<&MustBeFueled>,
                            ),
                        )> = items_of_wet_entity
                            .iter()
                            .filter(|(_, (_, in_backpack, _, _))| {
                                in_backpack.owner.id() == player_id
                            })
                            .collect();

                        for (item, (named, _, turned_on, fueled)) in items_to_wet {
                            entities_that_got_wet.push(item);
                            if turned_on.is_some() && fueled.is_some(){
                                entities_in_backpack_to_turn_off.push(item);
                                if player_id == entity.id() {
                                    game_log.entries.push(format!(
                                        "Your {} gets wet and turns itself off!",
                                        named.name
                                    ));
                                }
                            }
                        }
                    }
                } else if let Some(is_wet_component) = is_wet {
                    // Water dries out in time
                    is_wet_component.tick_countdown -= 1;
                    if is_wet_component.tick_countdown <= 0 {
                        entities_that_dryed.push(entity);

                        if player_id == entity.id() {
                            game_log.entries.push("You are no longer wet".to_string());
                        }
                    }
                }
            }
        }

        // Register that now edible is rottend
        for entity in entities_that_got_wet {
            let _ = ecs_world.insert_one(
                entity,
                Wet {
                    tick_countdown: STARTING_WET_COUNTER,
                },
            );
        }

        // Dry entity
        for entity in entities_that_dryed {
            let _ = ecs_world.remove_one::<Wet>(entity);
        }

        for entity in entities_in_backpack_to_turn_off {
            let _ = ecs_world.exchange_one::<TurnedOn, TurnedOff>(entity, TurnedOff {});
            Player::force_view_recalculation(ecs_world);
        }
    }
}
