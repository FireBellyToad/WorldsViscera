use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToApply,
        common::{GameLog, Named, Wet},
        health::{Cured, DiseaseType},
        items::{Appliable, Applied, Cure, InBackback, MustBeFueled, TurnedOff, TurnedOn},
        player::Player,
    },
    engine::state::EngineState,
};

pub struct ApplySystem {}

impl ApplySystem {
    /// Check entities that wants to apply Something
    pub fn check(ecs_world: &mut World) {
        let mut applicators_items_applied: Vec<(Entity, Entity, i32)> = Vec::new();
        let player_id = Player::get_entity_id();

        {
            let mut appliers = ecs_world.query::<&WantsToApply>();

            for (applier, wants_to_apply) in &mut appliers {
                let time = ecs_world
                    .get::<&Appliable>(wants_to_apply.item)
                    .expect("Must have Appliable component")
                    .application_time;
                applicators_items_applied.push((applier, wants_to_apply.item, time));
            }
        }

        for (applier, item, multiplier) in applicators_items_applied {
            let _ = ecs_world.remove_one::<WantsToApply>(applier);
            let _ = ecs_world.insert_one(item, Applied {});

            if player_id == applier.id() {
                Player::wait_after_action(ecs_world, multiplier);
            }
        }
    }

    /// Implement behavious for applied item
    pub fn do_applications(game_state: &mut EngineState) {
        let ecs_world = &mut game_state.ecs_world;
        let mut entities_applied: Vec<Entity> = Vec::new();
        let mut entities_to_turn_on: Vec<Entity> = Vec::new();
        let mut entities_to_turn_off: Vec<Entity> = Vec::new();
        let mut entities_cured: Vec<(Entity, Vec<DiseaseType>)> = Vec::new();
        let player_id = Player::get_entity_id();

        // Scope for keeping borrow checker quiet
        {
            //Log all the applications
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

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

            // Turn off item
            for (turnable, (_, named, in_backback)) in &mut applyables_turned_on {
                entities_to_turn_off.push(turnable);
                entities_applied.push(turnable);
                println!(
                    "ApplySystem::do_applications turnable {} is turned off",
                    turnable.id()
                );

                if player_id == in_backback.owner.id() {
                    game_log
                        .entries
                        .push(format!("You turn off your {}", named.name));
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
                            game_log
                                .entries
                                .push(format!("Your {} is too wet to be turned on", named.name));
                        }
                        continue;
                    } else if fuel.fuel_counter < 1 {
                        if player_id == in_backback.owner.id() {
                            game_log
                                .entries
                                .push(format!("Your {} has no fuel", named.name));
                        }
                        continue;
                    }
                }

                entities_to_turn_on.push(turnable);
                if player_id == in_backback.owner.id() {
                    game_log
                        .entries
                        .push(format!("You turn on your {}", named.name));
                }
            }

            // Curing diseases with an applied item
            for (entity, (cure, named, in_backback)) in &mut applyables_cured {
                entities_applied.push(entity);
                entities_cured.push((in_backback.owner, cure.diseases.clone()));

                if player_id == in_backback.owner.id() {
                    game_log
                        .entries
                        .push(format!("You apply the {} on yourself", named.name));
                } else {
                    //TODO would be nice to log npc
                }
            }
        }

        for entity in entities_applied {
            let _ = ecs_world.remove_one::<Applied>(entity);
        }

        for entity in entities_to_turn_on {
            let _ = ecs_world.exchange_one::<TurnedOff, TurnedOn>(entity, TurnedOn {});
            Player::force_view_recalculation(ecs_world);
        }

        for entity in entities_to_turn_off {
            let _ = ecs_world.exchange_one::<TurnedOn, TurnedOff>(entity, TurnedOff {});
            Player::force_view_recalculation(ecs_world);
        }

        for (entity, diseases) in entities_cured {
            let _ = ecs_world.insert_one(entity, Cured { diseases });
        }
    }
}
