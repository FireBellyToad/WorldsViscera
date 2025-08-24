use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToInvoke,
        combat::{CombatStats, InflictsDamage, SufferingDamage, WantsToZap},
        common::{GameLog, Named, Position, Wet},
        items::{Invokable, InvokablesEnum},
        player::Player,
    },
    constants::AUTOFAIL_SAVING_THROW,
    maps::zone::Zone,
    utils::{effect_manager::EffectManager, particle_animation::ParticleAnimation, roll::Roll},
};

pub struct ZapManager {}

impl ZapManager {
    pub fn run(ecs_world: &mut World) {
        let mut wants_to_zap_list: Vec<Entity> = Vec::new();
        let mut invokable_list: Vec<Entity> = Vec::new();
        let mut particle_animations: Vec<ParticleAnimation> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to zap stuff
            let mut zappers =
                ecs_world.query::<(&WantsToZap, &WantsToInvoke, &Position, Option<&Wet>)>();

            //Log all the zappings
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

            let player_id = Player::get_entity_id(ecs_world);

            for (zapper, (wants_zap, wants_invoke, zapper_position, wet)) in &mut zappers {
                let mut target_list: Vec<&Vec<Entity>> = Vec::new();
                // Could be needed...
                let zapper_wrapper = vec![zapper];
                let is_lightning_wand = ecs_world
                    .get::<&Invokable>(wants_invoke.item)
                    .unwrap()
                    .invokable_type
                    == InvokablesEnum::LightningWand;

                // If wet, Lightning wand makes zap himself too!
                if wet.is_some() && is_lightning_wand {
                    target_list.push(&zapper_wrapper);
                    game_log.entries.push(format!(
                        "Using the Lightning wand while wet was a bad idea..."
                    ));
                }

                // Do not draw if zapping himself
                if zapper_position.x != wants_zap.target.0
                    || zapper_position.y != wants_zap.target.1
                {
                    //TODO what if the effect is not a line?
                    let line_effect = EffectManager::new_line(
                        (zapper_position.x, zapper_position.y),
                        (wants_zap.target.0, wants_zap.target.1),
                    );

                    // Zap all entities in line (Excluding first)!
                    for &(x, y) in line_effect.iter().skip(1) {
                        let index = Zone::get_index_from_xy(x, y);
                        target_list.push(&zone.tile_content[index]);
                    }

                    particle_animations.push(ParticleAnimation::new_line(line_effect));
                } else {
                    // Only one if zapping himself
                    let index = Zone::get_index_from_xy(wants_zap.target.0, wants_zap.target.1);
                    target_list.push(&zone.tile_content[index]);
                }

                for &targets in &target_list {
                    for &target in targets {
                        let target_damage = ecs_world.get::<&mut SufferingDamage>(target);

                        //Sum damage, keeping in mind that could not have SufferingDamage component
                        if target_damage.is_ok() {
                            let target_stats = ecs_world.get::<&CombatStats>(target).unwrap();
                            let target_wet = ecs_world.get::<&Wet>(target);
                            let item_damage =
                                ecs_world.get::<&InflictsDamage>(wants_invoke.item).unwrap();
                            let damage_roll =
                                Roll::dice(item_damage.number_of_dices, item_damage.dice_size);

                            let mut saving_throw_roll = AUTOFAIL_SAVING_THROW;

                            // If target is wet while targeted by lightning wand , autofail the saving throw!
                            if target_wet.is_err() && is_lightning_wand {
                                saving_throw_roll = Roll::d20();
                            }

                            // Show appropriate log messages
                            let named_attacker = ecs_world.get::<&Named>(zapper).unwrap();
                            let named_target = ecs_world.get::<&Named>(target).unwrap();

                            // Dextery Save made halves damage
                            if saving_throw_roll > target_stats.current_dexterity {
                                target_damage.unwrap().damage_received += damage_roll;
                            } else {
                                target_damage.unwrap().damage_received += damage_roll / 2;
                                if target.id() == player_id {
                                    game_log.entries.push(format!("You duck some of the blow!"));
                                } else {
                                    game_log.entries.push(format!(
                                        "{} ducks some of the blow!",
                                        named_target.name
                                    ));
                                }
                            }

                            if zapper.id() == player_id {
                                if target.id() == player_id {
                                    game_log
                                        .entries
                                        .push(format!("You zap yourself for {} damage", damage_roll));
                                } else {
                                    game_log.entries.push(format!(
                                        "You zap the {} for {} damage",
                                        named_target.name, damage_roll
                                    ));
                                }
                            } else if target.id() == player_id {
                                game_log.entries.push(format!(
                                    "{} zaps you for {} damage",
                                    named_attacker.name, damage_roll
                                ));
                            } else {
                                game_log.entries.push(format!(
                                    "{} zaps the {} for {} damage",
                                    named_attacker.name, named_target.name, damage_roll
                                ));
                            }
                        };
                    }
                }

                // prepare lists for removal
                wants_to_zap_list.push(zapper);
                invokable_list.push(wants_invoke.item)
            }
        }

        // Remove owner's will to invoke and zap
        for zapper in wants_to_zap_list {
            let _ = ecs_world.remove_one::<WantsToInvoke>(zapper);
            let _ = ecs_world.remove_one::<WantsToZap>(zapper);
        }

        for particle in particle_animations {
            let _ = ecs_world.spawn((true, particle));
        }

        // Remove invokable item: is consumed!
        for invokable in invokable_list {
            let _ = ecs_world.despawn(invokable);
        }
    }
}
