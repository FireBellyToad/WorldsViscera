use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, Damageable},
        common::Named,
        player::Player,
    },
    utils::random_util::RandomUtils,
};

pub struct DamageManager {}

impl DamageManager {
    ///
    pub fn manage_damage(ecs_world: &World) {
        let mut damageables = ecs_world.query::<(&mut Damageable, &mut CombatStats)>();

        for (_e, (damageable, stats)) in &mut damageables {
            stats.current_stamina -= damageable.damage_received;

            //Decrease stamina. If less then 0, delta is subtracted from toughness
            if stats.current_stamina < 0 {
                // We add a negative value
                stats.current_toughness += stats.current_stamina;
                stats.current_stamina = max(0, stats.current_stamina);
            }
        }
    }

    /// Check which entities are dead and removes them. Returns true if Player is dead
    pub fn remove_dead(ecs_world: &mut World) -> bool {
        let mut dead_entities: Vec<Entity> = Vec::new();
        let player_entity: Entity;

        // Scope for keeping borrow checker quiet
        {
            let mut damageables = ecs_world.query::<(&CombatStats, &Named, &mut Damageable)>();
            for (entity, (stats, named, damageable)) in &mut damageables {

                // if has been damaged and Stamina is 0, do a thougness saving throw or die.
                // On 0 or less toughness, die anyway
                if stats.current_stamina == 0 && damageable.damage_received > 0 {
                    let saving_throw_roll = RandomUtils::d20();
                    println!(
                        "{}  saving with {} against thougness {} / {}",
                        named.name, saving_throw_roll, stats.current_toughness, stats.max_toughness
                    );
                    if stats.current_toughness < 1 || saving_throw_roll > stats.current_toughness {
                        dead_entities.push(entity);
                        println!("{} dies!", named.name);
                    } else if stats.current_toughness > 0 {
                        println!("{} staggers in pain!", named.name)
                    }
                }
                // Reset damage_received
                damageable.damage_received = 0;
            }
        }

        // Scope for keeping borrow checker quiet
        {
            let mut player_query = ecs_world.query::<&Player>();
            let (entity, _player) = player_query
                .iter()
                .last()
                .expect("Player is not in hecs::World");
            player_entity = entity;
        }

        //Remove all dead entities, stop game if player is dead
        for ent in dead_entities {
            if ent.id() == player_entity.id() {
                //Game over!
                return true;
            }

            ecs_world
                .despawn(ent)
                .expect(&format!("Cannot despawn entity {:?}", ent));
        }

        false
    }
}
