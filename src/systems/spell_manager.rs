use crate::{
    components::{
        combat::{InflictsDamage, WantsToCast},
        common::{MyTurn, SpellList},
        health::Stunned,
        items::{Spell, SpellType},
    },
    constants::STONE_FELL_PARTICLE_TYPE,
    engine::state::GameState,
};
use std::cmp::max;

use hecs::Entity;

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage, WantsToZap},
        common::{Hates, Named, Position},
    },
    maps::zone::Zone,
    utils::{
        common::Utils, effect_manager::EffectManager, particle_animation::ParticleAnimation,
        roll::Roll,
    },
};

pub struct SpellManager {}

impl SpellManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let mut wants_to_cast_list: Vec<(Entity, i32)> = Vec::new();
        let mut stunned_list: Vec<(Entity, i32)> = Vec::new();
        let mut particle_animations: Vec<ParticleAnimation> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to cast a spell at stuff
            let mut casters =
                ecs_world.query::<(&WantsToZap, &WantsToCast, &Position, &CombatStats)>();

            //Log all the castings
            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");

            for (caster, (wants_to_zap, wants_to_cast, caster_position, stats)) in &mut casters {
                let mut spell_query = ecs_world
                    .query_one::<(
                        &mut Spell,
                        &Named,
                        Option<&InflictsDamage>,
                        Option<&Stunned>,
                    )>(wants_to_cast.spell)
                    .expect("Spell");
                let (spell, named_spell, inflicts_damage_opt, stunned_opt) =
                    spell_query.get().expect("spell_query should have result");

                // Set spell countdown to a random value between 11 and 16
                spell.spell_cooldown = (Roll::d6() + 10) as u32;

                let mut target_opt: Option<Entity> = None;

                // Do not draw if caster is casting on himself
                if caster_position.x != wants_to_zap.target.0
                    || caster_position.y != wants_to_zap.target.1
                {
                    // Use particle type given by ranged spell
                    if zone.visible_tiles
                        [Zone::get_index_from_xy(&wants_to_zap.target.0, &wants_to_zap.target.1)]
                    {
                        // Select particle type based on spell type
                        match spell.spell_type {
                            SpellType::StoneFell => {
                                // show particle on striken guy
                                particle_animations.push(ParticleAnimation::simple_particle(
                                    wants_to_zap.target.0,
                                    wants_to_zap.target.1,
                                    STONE_FELL_PARTICLE_TYPE,
                                ));
                                let index = Zone::get_index_from_xy(
                                    &wants_to_zap.target.0,
                                    &wants_to_zap.target.1,
                                );
                                if !zone.tile_content[index].is_empty() {
                                    //Get the first damageable entity
                                    for &entity in &zone.tile_content[index] {
                                        if ecs_world
                                            .satisfies::<&SufferingDamage>(entity)
                                            .unwrap_or(false)
                                        {
                                            target_opt = Some(entity);
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => {
                                //Spell will be shot in line
                                let mut line_effect = EffectManager::new_line(
                                    (caster_position.x, caster_position.y),
                                    (wants_to_zap.target.0, wants_to_zap.target.1),
                                );

                                // get first entity or blocked tile in line (Exclude caster)
                                // you cannot shoot something behind a barrier or another npc
                                let mut must_truncate_line_at = (false, 0);
                                for (i, &(x, y)) in line_effect.iter().skip(1).enumerate() {
                                    let index = Zone::get_index_from_xy(&x, &y);

                                    if !zone.tile_content[index].is_empty() {
                                        //Get the first damageable entity
                                        for &entity in &zone.tile_content[index] {
                                            if ecs_world
                                                .satisfies::<&SufferingDamage>(entity)
                                                .unwrap_or(false)
                                            {
                                                target_opt = Some(entity);
                                                must_truncate_line_at = (true, i + 1);
                                                break;
                                            }
                                        }
                                        //If target is found, break the loop
                                        if target_opt.is_some() {
                                            break;
                                        }
                                    }

                                    // If no valid target is found, check for solid obstacle
                                    if target_opt.is_none() && zone.blocked_tiles[index] {
                                        // Log only if visible
                                        if zone.visible_tiles[Zone::get_index_from_xy(&x, &y)] {
                                            game_state
                                                .game_log
                                                .add_entry("The spell bounces on something solid");
                                        }
                                        must_truncate_line_at = (true, i + 1);

                                        break;
                                    }
                                }

                                if must_truncate_line_at.0 {
                                    line_effect.truncate(must_truncate_line_at.1);
                                }

                                particle_animations.push(ParticleAnimation::new_projectile(
                                    line_effect,
                                    spell.spell_type.particle(),
                                ));
                            }
                        }
                    }
                } else {
                    // Only one if shooting himself
                    let index =
                        Zone::get_index_from_xy(&wants_to_zap.target.0, &wants_to_zap.target.1);
                    target_opt = Some(zone.tile_content[index][0]);
                }

                if let Some(target) = target_opt {
                    // Hit target now hates caster Entity
                    if let Ok(mut target_hates) = ecs_world.get::<&mut Hates>(target) {
                        target_hates.list.insert(caster.id());
                    }

                    // Show appropriate log messages
                    let target_stats = Utils::get_target_stats(ecs_world, target);
                    let named_attacker = ecs_world
                        .get::<&Named>(caster)
                        .expect("Entity is not Named");
                    let named_target = ecs_world
                        .get::<&Named>(target)
                        .expect("Entity is not Named");

                    if Roll::d20() <= target_stats.current_dexterity {
                        if target.id() == player_id {
                            game_state.game_log.add_entry(&format!(
                                "You avoid the spell cast by {}",
                                named_attacker.name
                            ));
                        } else if zone.visible_tiles[Zone::get_index_from_xy(
                            &wants_to_zap.target.0,
                            &wants_to_zap.target.1,
                        )] {
                            game_state.game_log.add_entry(&format!(
                                "{} avoids the spell cast by {}",
                                named_target.name, named_attacker.name,
                            ));
                        }
                    } else {
                        //Sum damage, keeping in mind that could not have SufferingDamage component
                        if let Ok(mut target_damage) = ecs_world.get::<&mut SufferingDamage>(target)
                        {
                            // If the spell inflicts damage, calculate it
                            if let Some(inflicts_damage) = inflicts_damage_opt {
                                let damage_roll = max(
                                    0,
                                    Roll::dice(
                                        inflicts_damage.number_of_dices,
                                        inflicts_damage.dice_size,
                                    ),
                                );
                                target_damage.damage_received += damage_roll;
                                target_damage.damager = Some(caster);

                                if caster.id() == player_id {
                                    if target.id() == player_id {
                                        game_state.game_log.add_entry(&format!(
                                            "You {} yourself for {} damage",
                                            named_spell.attack_verb.unwrap_or("zap"),
                                            damage_roll
                                        ));
                                    } else {
                                        game_state.game_log.add_entry(&format!(
                                            "You {} the {} for {} damage",
                                            named_spell.attack_verb.unwrap_or("zap"),
                                            named_target.name,
                                            damage_roll
                                        ));
                                    }
                                } else if target.id() == player_id {
                                    game_state.game_log.add_entry(&format!(
                                        "{} {}s you for {} damage",
                                        named_attacker.name,
                                        named_spell.attack_verb.unwrap_or("zap"),
                                        damage_roll
                                    ));
                                } else if zone.visible_tiles[Zone::get_index_from_xy(
                                    &wants_to_zap.target.0,
                                    &wants_to_zap.target.1,
                                )] {
                                    game_state.game_log.add_entry(&format!(
                                        "{} {}s the {} for {} damage",
                                        named_attacker.name,
                                        named_spell.attack_verb.unwrap_or("zap"),
                                        named_target.name,
                                        damage_roll
                                    ));
                                }
                            }

                            //Stunning through spell
                            if let Some(stun) = stunned_opt {
                                stunned_list.push((target, stun.tick_counter));
                                if caster.id() == player_id {
                                    if target.id() == player_id {
                                        game_state.game_log.add_entry(&format!(
                                            "You {} yourself",
                                            named_spell.attack_verb.unwrap_or("zaps"),
                                        ));
                                    } else {
                                        game_state.game_log.add_entry(&format!(
                                            "You {} the {}",
                                            named_spell.attack_verb.unwrap_or("zap"),
                                            named_target.name
                                        ));
                                    }
                                } else if target.id() == player_id {
                                    game_state.game_log.add_entry(&format!(
                                        "{} {}s you",
                                        named_attacker.name,
                                        named_spell.attack_verb.unwrap_or("zap"),
                                    ));
                                } else if zone.visible_tiles[Zone::get_index_from_xy(
                                    &wants_to_zap.target.0,
                                    &wants_to_zap.target.1,
                                )] {
                                    game_state.game_log.add_entry(&format!(
                                        "{} {} the {}",
                                        named_attacker.name,
                                        named_spell.attack_verb.unwrap_or("zap"),
                                        named_target.name
                                    ));
                                }
                            }
                        };
                    }
                }

                // prepare lists for removal
                wants_to_cast_list.push((caster, stats.speed));
            }
        }

        // Remove owner's will to invoke and zap
        for (caster, speed) in wants_to_cast_list {
            let _ = ecs_world.remove::<(WantsToCast, WantsToZap)>(caster);
            Utils::wait_after_action(ecs_world, caster, speed);
        }

        for particle in particle_animations {
            let _ = ecs_world.spawn((true, particle));
        }

        for (target, tick_counter) in stunned_list {
            let _ = ecs_world.insert_one(target, Stunned { tick_counter });
        }
    }

    /// On spellcaster turn, decrease casting cooldowns for all spells
    pub fn decrease_cooldowns(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;

        let mut spellcasters = ecs_world.query::<&SpellList>().with::<&MyTurn>();

        for (_, spell_list) in &mut spellcasters {
            for spell_opt in &spell_list.spells {
                let mut spell = ecs_world
                    .get::<&mut Spell>(*spell_opt)
                    .expect("Must have Spell Component");

                // Decrease spell cooldown if it's greater than 0
                if spell.spell_cooldown > 0 {
                    spell.spell_cooldown -= 1;
                }
            }
        }
    }
}
