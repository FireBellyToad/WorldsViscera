use crate::{
    components::{common::Position, items::TurnedOff},
    constants::{FLAME_PARTICLE_TYPE, STANDARD_ACTION_MULTIPLIER},
    engine::state::GameState,
    utils::{common::Utils, particle_animation::ParticleAnimation},
};
use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToFuel,
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, Named},
        items::{InBackback, MustBeFueled, Refiller, TurnedOn},
        player::Player,
    },
    utils::roll::Roll,
};

type Fueler<'a> = (
    &'a Named,
    &'a WantsToFuel,
    &'a Position,
    &'a CombatStats,
    &'a mut SufferingDamage,
);

/// System for managing fuel consumption and refueling.
pub struct FuelManager {}

impl FuelManager {
    pub fn check_fuel(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state
            .current_player_entity
            .expect("Player should be set");

        let mut entities_to_turn_off: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of light producers that use fuel
            let mut turned_on_lighters = ecs_world
                .query::<(&mut MustBeFueled, &Named, Option<&InBackback>)>()
                .with::<&TurnedOn>()
                .without::<&Refiller>();

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (lighter, (fuel, named, entity_in_backpack)) in &mut turned_on_lighters {
                // Log fuel change for lantern used by player
                if let Some(in_backback) = entity_in_backpack {
                    // Log messages for fuel status
                    if player_entity.id() == in_backback.owner.id() {
                        match fuel.fuel_counter {
                            30 => {
                                game_log
                                    .entries
                                    .push(format!("Your {} is flickering", named.name));
                            }
                            1 => {
                                game_log
                                    .entries
                                    .push(format!("Your {} goes out", named.name));
                                entities_to_turn_off.push(lighter);
                            }
                            _ => {}
                        }
                    }
                }

                //If fuel is less then 1, the lighter will not produce light
                if fuel.fuel_counter > 0 {
                    fuel.fuel_counter -= 1;
                }
            }
        }

        for entity in entities_to_turn_off {
            let _ = game_state
                .ecs_world
                .exchange_one::<TurnedOn, TurnedOff>(entity, TurnedOff {});
            Player::force_view_recalculation(game_state);
        }
    }

    pub fn do_refills(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let mut refillers_and_items_used: Vec<(Entity, Entity, i32)> = Vec::new();
        let mut particle_animations: Vec<ParticleAnimation> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of light producers with fuel
            let mut query = ecs_world.query::<Fueler>();
            let wants_to_refill_list: Vec<(Entity, Fueler)> = query
                .iter()
                .filter(|(_, (_, w, _, _, _))| w.item.is_some())
                .collect();

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (
                refiller,
                (named_fueler, wants_to_refill, position, fueler_stats, fueler_damage),
            ) in wants_to_refill_list
            {
                let target = wants_to_refill
                    .item
                    .expect("item to refill must not be None");
                let item_used = wants_to_refill.with;

                // Components for refiller item
                let item_used_fuel = ecs_world
                    .get::<&Refiller>(item_used)
                    .expect("Entity has not MustBeFueled");

                // Components for refilled item
                let mut target_fuel_query = ecs_world
                    .query_one::<(&mut MustBeFueled, &Named, Option<&TurnedOn>)>(target)
                    .expect("At least one result");
                let target_fuel_optional = target_fuel_query.get();

                match target_fuel_optional {
                    Some((target_fuel, named_target, turned_on)) => {
                        // Bad idea to refill a lit lantern!
                        if turned_on.is_some() {
                            if player_id == refiller.id() {
                                game_log.entries.push(format!(
                                    "The {} is lit! Flaming oil spills on your skin",
                                    named_target.name
                                ));
                            }
                            // show fire particle on burned guy
                            particle_animations.push(ParticleAnimation::simple_particle(
                                position.x,
                                position.y,
                                FLAME_PARTICLE_TYPE,
                            ));

                            // Dextery Save made halves damage
                            let saving_throw_roll = Roll::d20();
                            let damage_roll = Roll::dice(1, 6);
                            if saving_throw_roll > fueler_stats.current_dexterity {
                                fueler_damage.damage_received += damage_roll;
                            } else {
                                fueler_damage.damage_received += damage_roll / 2;
                                if target.id() == player_id {
                                    game_log
                                        .entries
                                        .push("You duck some of the damage!".to_string());
                                } else {
                                    game_log.entries.push(format!(
                                        "{} ducks some of the damage!",
                                        named_fueler.name
                                    ));
                                }
                            }
                        } else {
                            // Refill!
                            target_fuel.fuel_counter = item_used_fuel.fuel_counter;

                            // Show appropriate log messages
                            let named_item_used = ecs_world
                                .get::<&Named>(item_used)
                                .expect("Entity is not Named");

                            if player_id == refiller.id() {
                                game_log.entries.push(format!(
                                    "You refill the {} with the {}",
                                    named_target.name, named_item_used.name
                                ));
                            } else {
                                game_log.entries.push(format!(
                                    "{} refills the {} with the {}",
                                    named_fueler.name, named_target.name, named_item_used.name
                                ));
                            }
                        }
                    }
                    None => {
                        game_log
                            .entries
                            .push("This item cannot be refilled!".to_string());
                    }
                }

                // Remember refiller and what has been used to cleanup later
                refillers_and_items_used.push((refiller, item_used, fueler_stats.speed));
            }
        }

        //Cleanup
        for (refiller, item_used, speed) in refillers_and_items_used {
            let _ = ecs_world.remove_one::<WantsToFuel>(refiller);
            let _ = ecs_world.despawn(item_used);

            Utils::wait_after_action(ecs_world, refiller, speed * STANDARD_ACTION_MULTIPLIER);
        }

        for particle in particle_animations {
            let _ = ecs_world.spawn((true, particle));
        }
    }
}
