use hecs::Entity;

use crate::{
    components::{
        actions::WantsToApply,
        combat::CombatStats,
        common::{Key, Lock, Named, Position, Wet},
        health::{Cured, DiseaseType},
        items::{Appliable, Applied, Cure, InBackback, MustBeFueled, TurnedOff, TurnedOn},
        player::Player,
    },
    constants::NEXT_TO_DISTANCE,
    engine::state::GameState,
    maps::zone::{TileType, Zone},
    utils::common::Utils,
};

pub struct ApplySystem {}

impl ApplySystem {
    /// Check entities that wants to apply Something
    pub fn check(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;

        let mut applicators_items_applied: Vec<(Entity, Entity, i32)> = Vec::new();

        {
            let mut appliers = ecs_world.query::<(&WantsToApply, &CombatStats)>();

            for (applier, (wants_to_apply, stats)) in &mut appliers {
                let time = ecs_world
                    .get::<&Appliable>(wants_to_apply.item)
                    .expect("Must have Appliable component")
                    .application_time;
                applicators_items_applied.push((applier, wants_to_apply.item, stats.speed * time));
            }
        }

        for (applier, item, time) in applicators_items_applied {
            let _ = ecs_world.remove_one::<WantsToApply>(applier);
            let _ = ecs_world.insert_one(item, Applied {});

            Utils::wait_after_action(ecs_world, applier, time);
        }
    }

    /// Implement behavious for applied item
    pub fn do_applications(game_state: &mut GameState) {
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let zone = game_state
            .current_zone
            .as_mut()
            .expect("Must have current zone");

        let mut entities_applied: Vec<Entity> = Vec::new();
        let mut entities_to_turn_on: Vec<Entity> = Vec::new();
        let mut entities_to_turn_off: Vec<Entity> = Vec::new();
        let mut entities_cured: Vec<(Entity, Vec<DiseaseType>)> = Vec::new();
        let mut entities_to_despawn: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let ecs_world = &mut game_state.ecs_world;

            // If there are no applied items, avoid querying other components
            if ecs_world.query::<&Applied>().iter().len() == 0 {
                return;
            }

            // List of entities that want to act
            let mut applyables_turned_on = ecs_world
                .query::<(&TurnedOn, &Named, &InBackback)>()
                .with::<&Applied>();
            let mut applyables_turned_off = ecs_world
                .query::<(
                    &TurnedOff,
                    &Named,
                    &InBackback,
                    Option<&Wet>,
                    Option<&MustBeFueled>,
                )>()
                .with::<&Applied>();
            let mut applyables_cured = ecs_world
                .query::<(&Cure, &Named, &InBackback)>()
                .with::<&Applied>();
            let mut appliables_keys = ecs_world.query::<(&Key, &InBackback)>().with::<&Applied>();

            // Turn off item
            for (turnable, (_, named, in_backback)) in &mut applyables_turned_on {
                entities_to_turn_off.push(turnable);
                entities_applied.push(turnable);
                println!(
                    "ApplySystem::do_applications turnable {} is turned off",
                    turnable.id()
                );

                if player_id == in_backback.owner.id() {
                    game_state
                        .game_log
                        .add_entry(&format!("You turn off your {}", named.name));
                }
            }

            // Turn on item
            for (turnable, (_, named, in_backback, wet, must_be_fueled)) in
                &mut applyables_turned_off
            {
                entities_applied.push(turnable);

                if let Some(fuel) = must_be_fueled {
                    if wet.is_some() {
                        if player_id == in_backback.owner.id() {
                            game_state.game_log.add_entry(&format!(
                                "Your {} is too wet to be turned on",
                                named.name
                            ));
                        }
                        continue;
                    } else if fuel.fuel_counter < 1 {
                        if player_id == in_backback.owner.id() {
                            game_state
                                .game_log
                                .add_entry(&format!("Your {} has no fuel", named.name));
                        }
                        continue;
                    }
                }

                entities_to_turn_on.push(turnable);
                if player_id == in_backback.owner.id() {
                    game_state
                        .game_log
                        .add_entry(&format!("You turn on your {}", named.name));
                }
            }

            // Curing diseases with an applied item
            for (entity, (cure, named, in_backback)) in &mut applyables_cured {
                entities_applied.push(entity);
                entities_cured.push((in_backback.owner, cure.diseases.clone()));

                if player_id == in_backback.owner.id() {
                    game_state
                        .game_log
                        .add_entry(&format!("You apply the {} on yourself", named.name));
                } else {
                    //TODO would be nice to log npc
                }
            }

            // Applying keys
            for (applied_key_entity, (key, in_backback)) in &mut appliables_keys {
                let key_user_position = ecs_world
                    .get::<&Position>(in_backback.owner)
                    .expect("key user must have position");
                let mut q = ecs_world
                    .query_one::<(&Position, &mut Lock)>(key.lock)
                    .unwrap_or_else(|_| panic!("Key lock {} not found", key.lock.id()));
                let (lock_position, lock) = q.get().expect("q for lock failed");

                if Utils::distance(
                    &key_user_position.x,
                    &lock_position.x,
                    &key_user_position.y,
                    &lock_position.y,
                ) <= NEXT_TO_DISTANCE
                {
                    entities_applied.push(applied_key_entity);
                    entities_to_despawn.push(applied_key_entity);
                    lock.keys_to_unlock -= 1;
                    //Remove lock when opened
                    if lock.keys_to_unlock == 0 {
                        entities_to_despawn.push(key.lock);
                        game_state.game_log.add_entry("The lock opens!");
                        zone.tiles[Zone::get_index_from_xy(&lock_position.x, &lock_position.y)] =
                            TileType::DownPassage;
                    } else {
                        game_state
                            .game_log
                            .add_entry("The lock moves, but is not open yet.");

                        zone.tiles[Zone::get_index_from_xy(&lock_position.x, &lock_position.y)] =
                            match zone.tiles
                                [Zone::get_index_from_xy(&lock_position.x, &lock_position.y)]
                            {
                                TileType::GoldLock(locks_to_open) => {
                                    TileType::GoldLock(locks_to_open - 1.0)
                                }
                                _ => panic!("Unexpected tile type for lock tile"),
                            }
                    }
                } else {
                    game_state
                        .game_log
                        .add_entry("You are too far away from the lock to apply the key.");
                }
            }
        }

        for entity in entities_applied {
            let _ = game_state.ecs_world.remove_one::<Applied>(entity);
        }

        for entity in entities_to_turn_on {
            let _ = game_state
                .ecs_world
                .exchange_one::<TurnedOff, TurnedOn>(entity, TurnedOn {});
            Player::force_view_recalculation(game_state);
        }

        for entity in entities_to_turn_off {
            let _ = game_state
                .ecs_world
                .exchange_one::<TurnedOn, TurnedOff>(entity, TurnedOff {});
            Player::force_view_recalculation(game_state);
        }

        for (entity, diseases) in entities_cured {
            let _ = game_state.ecs_world.insert_one(entity, Cured { diseases });
        }

        for entity in entities_to_despawn {
            let _ = game_state.ecs_world.despawn(entity);
        }
    }
}
