use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{GameLog, Named, Position, ProduceCorpse},
        health::CanAutomaticallyHeal,
        items::Edible,
        player::Player,
    },
    constants::MAX_STAMINA_HEAL_TICK_COUNTER,
    maps::game_map::GameMap,
    spawner::Spawn,
    utils::roll::Roll,
};

pub struct DamageManager {}

impl DamageManager {
    ///
    pub fn run(ecs_world: &World) {
        let mut damageables =
            ecs_world.query::<(&mut SufferingDamage, &mut CombatStats, &Position)>();

        let mut map_query = ecs_world.query::<&mut GameMap>();
        let (_e, map) = map_query
            .iter()
            .last()
            .expect("GameMap is not in hecs::World");

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
                map.bloodied_tiles
                    .insert(GameMap::get_index_from_xy(position.x, position.y));
            }
        }
    }

    /// Check which entities are dead and removes them. Returns true if Player is dead
    pub fn remove_dead(ecs_world: &mut World) -> bool {
        let mut dead_entities: Vec<(Entity, String, i32, (i32, i32))> = Vec::new();
        let player_entity_id = Player::get_player_id(ecs_world);

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

                    game_log.entries.push(format!(
                        "{} saving with {} against toughness {} / {}",
                        named.name, saving_throw_roll, stats.current_toughness, stats.max_toughness
                    ));
                    if stats.current_toughness < 1 || saving_throw_roll > stats.current_toughness {
                        //Prepare data for potential corpse
                        let mut corpse_probability = 0;
                        let produce_corpse = ecs_world.get::<&ProduceCorpse>(entity);
                        if produce_corpse.is_ok() {
                            corpse_probability = produce_corpse.unwrap().probability;
                        }

                        dead_entities.push((
                            entity,
                            named.name.clone(),
                            corpse_probability,
                            (position.x, position.y),
                        ));
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
        for (ent, name, corpse_probability, (x, y)) in dead_entities {
            if ent.id() == player_entity_id {
                //Game over!
                return true;
            }

            if Roll::d100() < corpse_probability {
                // Create corpse
                Spawn::corpse(
                    ecs_world,
                    x,
                    y,
                    name,
                    Edible {
                        nutrition_dice_number: 3,
                        nutrition_dice_size: 12,
                    },
                );
            }

            ecs_world
                .despawn(ent)
                .expect(&format!("Cannot despawn entity {:?}", ent));
        }

        false
    }
}
