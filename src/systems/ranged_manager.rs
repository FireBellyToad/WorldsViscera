use crate::{components::items::InBackback, utils::common::AmmunitionInBackpack};
use std::{cmp::max, panic};

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage, WantsToShoot, WantsToZap},
        common::{GameLog, Hates, Named, Position},
        items::{Armor, Equipped, Eroded, RangedWeapon},
        player::Player,
    },
    constants::BOLT_PARTICLE_TYPE,
    maps::zone::Zone,
    utils::{
        common::Utils, effect_manager::EffectManager, particle_animation::ParticleAnimation,
        roll::Roll,
    },
};

pub struct RangedManager {}

impl RangedManager {
    pub fn run(ecs_world: &mut World) {
        let mut wants_to_shoot_list: Vec<(Entity, i32)> = Vec::new();
        let mut particle_animations: Vec<ParticleAnimation> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to shoot at stuff
            let mut shooters =
                ecs_world.query::<(&WantsToZap, &WantsToShoot, &Position, &CombatStats)>();

            let mut ammo_in_backpack = ecs_world.query::<AmmunitionInBackpack>();

            //Log all the shootings
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

            let mut equipped_armors = ecs_world.query::<(&Armor, &Equipped, Option<&Eroded>)>();

            let player_id = Player::get_entity_id(ecs_world);

            for (shooter, (wants_to_zap, wants_to_shoot, shooter_position, stats)) in &mut shooters
            {
                let weapon_stats = ecs_world
                    .get::<&RangedWeapon>(wants_to_shoot.weapon)
                    .expect("Entity has no RangedWeapon"); // TODO maybe refactor this with InflictsDamage component;

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
                        let index = Zone::get_index_from_xy(x, y);

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
                            if zone.visible_tiles[Zone::get_index_from_xy(x, y)] {
                                game_log.entries.push(
                                    "The projectile gets stuck into something solid".to_string(),
                                );
                            }
                            must_truncate_line_at = (true, i + 1);
                        }
                    }

                    if must_truncate_line_at.0 {
                        line_effect.truncate(must_truncate_line_at.1);
                    }

                    // TODO use particle type given by ranged weapon
                    if zone.visible_tiles
                        [Zone::get_index_from_xy(wants_to_zap.target.0, wants_to_zap.target.1)]
                    {
                        particle_animations.push(ParticleAnimation::new_projectile(
                            line_effect,
                            BOLT_PARTICLE_TYPE,
                        ));
                    }
                } else {
                    // Only one if zapping himself
                    let index =
                        Zone::get_index_from_xy(wants_to_zap.target.0, wants_to_zap.target.1);
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
                        let damage_roll =
                            max(0, Roll::dice(1, weapon_stats.attack_dice) - target_armor);
                        target_damage.damage_received += damage_roll;

                        if shooter.id() == player_id {
                            if target.id() == player_id {
                                game_log
                                    .entries
                                    .push(format!("You shoot yourself for {} damage", damage_roll));
                            } else {
                                game_log.entries.push(format!(
                                    "You shoot the {} for {} damage",
                                    named_target.name, damage_roll
                                ));
                            }
                        } else if target.id() == player_id {
                            game_log.entries.push(format!(
                                "{} shoot you for {} damage",
                                named_attacker.name, damage_roll
                            ));
                        } else if zone.visible_tiles
                            [Zone::get_index_from_xy(wants_to_zap.target.0, wants_to_zap.target.1)]
                        {
                            game_log.entries.push(format!(
                                "{} shoot the {} for {} damage",
                                named_attacker.name, named_target.name, damage_roll
                            ));
                        }
                    };
                }

                // prepare lists for removal
                wants_to_shoot_list.push((shooter, stats.speed));
                // ranged_list.push(wants_to_shoot.weapon); // TODO ammunitions count
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
    pub fn check_ammo_counts(ecs_world: &mut World) {
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
                    if weapon_backpack.owner.id() == ammo_backpack.owner.id() {
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
