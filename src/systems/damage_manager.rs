use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, CanAutomaticallyHeal, SufferingDamage},
        common::{GameLog, Named},
        player::Player,
    },
    constants::MAX_STAMINA_HEAL_COUNTER,
    utils::roll::Roll,
};

pub struct DamageManager {}

impl DamageManager {
    ///
    pub fn run(ecs_world: &World) {
        let mut damageables = ecs_world.query::<(&mut SufferingDamage, &mut CombatStats)>();

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
        let player_entity_id = Player::get_player_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut damageables = ecs_world.query::<(&CombatStats, &Named, &mut SufferingDamage)>();
            for (entity, (stats, named, damageable)) in &mut damageables {
                // if has been damaged and Stamina is 0, do a thougness saving throw or die.
                // On 0 or less toughness, die anyway
                if stats.current_stamina == 0 && damageable.damage_received > 0 {
                    let saving_throw_roll = Roll::d20();

                    game_log.entries.push(format!(
                        "{} saving with {} against thougness {} / {}",
                        named.name, saving_throw_roll, stats.current_toughness, stats.max_toughness
                    ));
                    if stats.current_toughness < 1 || saving_throw_roll > stats.current_toughness {
                        dead_entities.push(entity);
                        game_log.entries.push(format!("{} dies!", named.name));
                    } else if stats.current_toughness > 0 {
                        game_log
                            .entries
                            .push(format!("{} staggers in pain!", named.name));
                    }
                    
                    // If can heal stamina, reset counter
                    let regen = ecs_world.get::<&mut CanAutomaticallyHeal>(entity);
                    if regen.is_ok() {
                        regen.unwrap().counter = MAX_STAMINA_HEAL_COUNTER+2;
                    }
                }
                // Reset damage_received
                damageable.damage_received = 0;
            }
        }

        //Remove all dead entities, stop game if player is dead
        for ent in dead_entities {
            if ent.id() == player_entity_id {
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
