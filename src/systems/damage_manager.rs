use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, Named, Position},
        health::CanAutomaticallyHeal,
        items::Edible,
        player::Player,
    },
    constants::MAX_STAMINA_HEAL_TICK_COUNTER,
    maps::zone::{ParticleType, Zone},
    spawner::Spawn,
    utils::roll::Roll,
};

pub struct DamageManager {}

impl DamageManager {
    ///
    pub fn run(ecs_world: &World) {
        let mut damageables =
            ecs_world.query::<(&mut SufferingDamage, &mut CombatStats, &Position)>();

        let mut zone_query = ecs_world.query::<&mut Zone>();
        let (_e, zone) = zone_query.iter().last().expect("Zone is not in hecs::World");

        for (damaged_entity, (damageable, stats, position)) in &mut damageables {
            if damageable.damage_received > 0 {
                stats.current_stamina -= damageable.damage_received;

                //Decrease stamina. If less then 0, delta is subtracted from toughness
                if stats.current_stamina < 0 {
                    // We add a negative value
                    stats.current_toughness += stats.current_stamina;
                    stats.current_stamina = max(0, stats.current_stamina);
                }

                // If can heal stamina, reset counter
                let regen = ecs_world.get::<&mut CanAutomaticallyHeal>(damaged_entity);
                if regen.is_ok() {
                    regen.unwrap().tick_counter = MAX_STAMINA_HEAL_TICK_COUNTER + 2;
                }

                //Drench the tile with blood
                zone.particle_tiles.insert(
                    Zone::get_index_from_xy(position.x, position.y),
                    ParticleType::Blood,
                );
            }
        }
    }

    /// Check which entities are dead and removes them. Returns true if Player is dead
    pub fn remove_dead_and_check_gameover(ecs_world: &mut World) -> bool {
        let mut dead_entities: Vec<(Entity, String, (i32, i32))> = Vec::new();
        let player_entity_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut damageables =
                ecs_world.query::<(&CombatStats, &Named, &mut SufferingDamage, &Position)>();
            for (entity, (stats, named, damageable, position)) in &mut damageables {
                // if has been damaged and Stamina is 0, do a thougness saving throw or die.
                // On 0 or less toughness, die anyway
                if stats.current_stamina == 0 && damageable.damage_received > 0 {
                    let saving_throw_roll = Roll::d20();
                    if stats.current_toughness < 1 || saving_throw_roll > stats.current_toughness {
                        dead_entities.push((entity, named.name.clone(), (position.x, position.y)));
                        game_log.entries.push(format!("{} dies!", named.name));
                    } else if stats.current_toughness > 0 {
                        game_log
                            .entries
                            .push(format!("{} staggers in pain!", named.name));
                    }
                }
                // Reset damage_received
                damageable.damage_received = 0;
            }
        }

        //Remove all dead entities, stop game if player is dead
        for (ent, name, (x, y)) in dead_entities {
            if ent.id() == player_entity_id {
                //Game over!
                return true;
            }

            // Create corpse
            // TODO change nutrition based on monster
            Spawn::corpse(
                ecs_world,
                x,
                y,
                name,
                Edible {
                    nutrition_dice_number:5,
                    nutrition_dice_size: 20,
                },
            );

            ecs_world
                .despawn(ent)
                .expect(&format!("Cannot despawn entity {:?}", ent));
        }

        false
    }
}
