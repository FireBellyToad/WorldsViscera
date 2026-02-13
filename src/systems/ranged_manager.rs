use crate::{
    components::{combat::InflictsDamage, items::InBackback},
    engine::state::GameState,
    utils::common::AmmunitionInBackpack,
};
use std::{cmp::max, panic};

use hecs::Entity;

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage, WantsToShoot, WantsToZap},
        common::{GameLog, Hates, Named, Position},
        items::{Armor, Equipped, Eroded, RangedWeapon},
    },
    maps::zone::Zone,
    utils::{
        common::Utils, effect_manager::EffectManager, particle_animation::ParticleAnimation,
        roll::Roll,
    },
};

pub struct RangedManager {}

impl RangedManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let mut wants_to_shoot_list: Vec<(Entity, i32)> = Vec::new();
        let mut particle_animations: Vec<ParticleAnimation> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to shoot at stuff
            let mut shooters =
                ecs_world.query::<(&WantsToZap, &WantsToShoot, &Position, &CombatStats)>();

            let mut ammo_in_backpack = ecs_world.query::<AmmunitionInBackpack>();

            //Log all the shootings

            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");
            let mut equipped_armors = ecs_world.query::<(&Armor, &Equipped, Option<&Eroded>)>();

            for (shooter, (wants_to_zap, wants_to_shoot, shooter_position, stats)) in &mut shooters
            {
                let mut weapon_stats_query = ecs_world
                    .query_one::<(&RangedWeapon, &InflictsDamage, Option<&Eroded>)>(
                        wants_to_shoot.weapon,
                    )
                    .expect("Entity must be RangedWeapon and could InflictDamage");
                let (weapon_stats, inflicts_damage, eroded_opt) = weapon_stats_query
                    .get()
                    .expect("weapon_stats_query should have result"); // TODO maybe refactor this with InflictsDamage component;

                // If the shooter has the ammo for at least one equipped ranged weapon, it can shoot!
                let ammo: Vec<(Entity, AmmunitionInBackpack)> = ammo_in_backpack
                    .iter()
                    .filter(|(_, (in_backpack, ammo))| {
                        in_backpack.owner.id() == shooter.id()
                            && ammo.ammo_type == weapon_stats.ammo_type
                    })
                    .collect();

                if ammo.is_empty() {
                    panic!("No ammo for weapon, why are we trying to shoot?");
                } else {
                    // Decrease ammo count
                    if let Some((_, (_, ammo_count))) = ammo.into_iter().next() {
                        ammo_count.ammo_count -= 1;
                    }
                }

                let mut target_opt: Option<Entity> = None;

                // Do not draw if shooter is himself
                if shooter_position.x != wants_to_zap.target.0
                    || shooter_position.y != wants_to_zap.target.1
                {
                    //Projectile will be shot in line
                    let mut line_effect = EffectManager::new_line(
                        (shooter_position.x, shooter_position.y),
                        (wants_to_zap.target.0, wants_to_zap.target.1),
                    );

                    // get first entity or blocked tile in line (Exclude shooter)
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
                                game_state.game_log.entries.push(
                                    "The projectile gets stuck into something solid".to_string(),
                                );
                            }
                            must_truncate_line_at = (true, i + 1);
                        }
                    }

                    if must_truncate_line_at.0 {
                        line_effect.truncate(must_truncate_line_at.1);
                    }

                    // Use particle type given by ranged weapon
                    if zone.visible_tiles
                        [Zone::get_index_from_xy(&wants_to_zap.target.0, &wants_to_zap.target.1)]
                    {
                        particle_animations.push(ParticleAnimation::new_projectile(
                            line_effect,
                            weapon_stats.ammo_type.particle(),
                        ));
                    }
                } else {
                    // Only one if shooting himself
                    let index =
                        Zone::get_index_from_xy(&wants_to_zap.target.0, &wants_to_zap.target.1);
                    target_opt = Some(zone.tile_content[index][0]);
                }

                if let Some(target) = target_opt {
                    // Hit target now hates shooter Entity
                    if let Ok(mut target_hates) = ecs_world.get::<&mut Hates>(target) {
                        target_hates.list.insert(shooter.id());
                    }

                    //Sum damage, keeping in mind that could not have SufferingDamage component
                    if let Ok(mut target_damage) = ecs_world.get::<&mut SufferingDamage>(target) {
                        let target_stats = ecs_world
                            .get::<&CombatStats>(target)
                            .expect("Entity has no CombatStats");

                        // Show appropriate log messages
                        let named_attacker = ecs_world
                            .get::<&Named>(shooter)
                            .expect("Entity is not Named");
                        let named_target = ecs_world
                            .get::<&Named>(target)
                            .expect("Entity is not Named");

                        // Ranged weapons damage is subjected to armor
                        let target_armor = Utils::get_armor_value(
                            target_stats.base_armor,
                            target.id(),
                            &mut equipped_armors,
                        );

                        let mut erosion_malus = 0;
                        if let Some(eroded) = eroded_opt {
                            erosion_malus = eroded.value as i32;
                        }
                        let damage_roll = max(
                            0,
                            Roll::dice(inflicts_damage.number_of_dices, inflicts_damage.dice_size)
                                - target_armor
                                - erosion_malus,
                        );
                        target_damage.damage_received += damage_roll;
                        target_damage.damager = Some(shooter);

                        if shooter.id() == player_id {
                            if target.id() == player_id {
                                game_state
                                    .game_log
                                    .entries
                                    .push(format!("You shoot yourself for {} damage", damage_roll));
                            } else {
                                game_state.game_log.entries.push(format!(
                                    "You shoot the {} for {} damage",
                                    named_target.name, damage_roll
                                ));
                            }
                        } else if target.id() == player_id {
                            game_state.game_log.entries.push(format!(
                                "{} shoot you for {} damage",
                                named_attacker.name, damage_roll
                            ));
                        } else if zone.visible_tiles[Zone::get_index_from_xy(
                            &wants_to_zap.target.0,
                            &wants_to_zap.target.1,
                        )] {
                            game_state.game_log.entries.push(format!(
                                "{} shoot the {} for {} damage",
                                named_attacker.name, named_target.name, damage_roll
                            ));
                        }
                    };
                }

                // prepare lists for removal
                wants_to_shoot_list.push((shooter, stats.speed));
            }
        }

        // Remove owner's will to invoke and zap
        for (shooter, speed) in wants_to_shoot_list {
            let _ = ecs_world.remove::<(WantsToShoot, WantsToZap)>(shooter);
            Utils::wait_after_action(ecs_world, shooter, speed);
        }

        for particle in particle_animations {
            let _ = ecs_world.spawn((true, particle));
        }
    }

    /// Manage ammo in backpack, despawn empty ammo entities
    pub fn check_ammo_counts(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let mut empty_ammo_list: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut ammo_in_backpack = ecs_world.query::<AmmunitionInBackpack>();
            let mut ranged_weapons = ecs_world.query::<(&InBackback, &mut RangedWeapon)>();

            // set all totals to 0
            for (_, (_, ranged_weapon)) in &mut ranged_weapons {
                ranged_weapon.ammo_count_total = 0
            }

            // Update ammo count for each carried weapon and despawn empty ammo entities
            for (ammo_entity, (ammo_backpack, ammo_component)) in &mut ammo_in_backpack {
                for (_, (weapon_backpack, ranged_weapon)) in &mut ranged_weapons {
                    if weapon_backpack.owner.id() == ammo_backpack.owner.id()
                        && ranged_weapon.ammo_type == ammo_component.ammo_type
                    {
                        ranged_weapon.ammo_count_total += ammo_component.ammo_count;
                    }
                }
                if ammo_component.ammo_count == 0 {
                    empty_ammo_list.push(ammo_entity);
                }
            }
        }

        for entity in empty_ammo_list {
            let _ = ecs_world.despawn(entity);
        }
    }
}
