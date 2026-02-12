use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{Experience, GameLog, Hates, Named, Position},
        health::{CanAutomaticallyHeal, DiseaseType, Paralyzed},
        items::{Deadly, DontLeaveCorpse, Edible},
        monster::{DiseaseBearer, Venomous},
        player::Player,
    },
    constants::{AUTO_ADVANCE_EXP_COUNTER_START, MAX_STAMINA_HEAL_TICK_COUNTER},
    engine::state::{GameState, RunState},
    maps::zone::{DecalType, Zone},
    spawning::spawner::Spawn,
    systems::item_dropping::ItemDropping,
    utils::roll::Roll,
};

type DeadEntityData = (Entity, String, (i32, i32), Option<Entity>, u32);

pub struct DamageManager {}

/// Damage manager system
impl DamageManager {
    pub fn run(ecs_world: &World) {
        let mut damageables =
            ecs_world.query::<(&mut SufferingDamage, &mut CombatStats, &Position)>();

        let mut zone_query = ecs_world.query::<&mut Zone>();
        let (_, zone) = zone_query
            .iter()
            .last()
            .expect("Zone is not in hecs::World");

        for (damaged_entity, (damageable, stats, position)) in &mut damageables {
            if damageable.damage_received > 0 {
                // From now on, damaged entity will be hostile to its damager
                if let Some(damager) = damageable.damager
                    && let Ok(mut target_hates) = ecs_world.get::<&mut Hates>(damaged_entity)
                {
                    target_hates.list.insert(damager.id());
                }

                stats.current_stamina -= damageable.damage_received;
                //Decrease stamina. If less then 0, delta is subtracted from toughness
                if stats.current_stamina < 0 {
                    // We add a negative value
                    stats.current_toughness += stats.current_stamina;
                    stats.current_stamina = max(0, stats.current_stamina);
                }

                // If can heal stamina, reset counter
                if let Ok(mut regen) = ecs_world.get::<&mut CanAutomaticallyHeal>(damaged_entity) {
                    regen.tick_counter = MAX_STAMINA_HEAL_TICK_COUNTER + 2;
                }

                //Drench the tile with blood
                zone.decals_tiles.insert(
                    Zone::get_index_from_xy(&position.x, &position.y),
                    DecalType::Blood,
                );
            }

            // Venomous hits
            if damageable.toughness_damage_received > 0 {
                stats.current_toughness = max(
                    0,
                    stats.current_toughness - damageable.toughness_damage_received,
                );

                if stats.current_toughness < 1 {
                    stats.current_stamina = 0;
                }
            }

            // Disease hits on dexterity
            if damageable.dexterity_damage_received > 0 {
                stats.current_dexterity = max(
                    0,
                    stats.current_dexterity - damageable.dexterity_damage_received,
                );
            }
        }
    }

    /// Check which entities are dead and removes them. Returns true if Player is dead
    pub fn remove_dead_and_check_gameover(game_state: &mut GameState) -> bool {
        let ecs_world = &mut game_state.ecs_world;
        let mut dead_entities: Vec<DeadEntityData> = Vec::new();
        let mut paralyzed_entities: Vec<Entity> = Vec::new();
        let player_entity_id = Player::get_entity_id();

        // Scope for keeping borrow checker quiet
        {
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut zone_query = ecs_world.query::<&mut Zone>();
            let (_, zone) = zone_query
                .iter()
                .last()
                .expect("Zone is not in hecs::World");

            let mut damageables = ecs_world.query::<(
                &CombatStats,
                &Named,
                &mut SufferingDamage,
                &Position,
                Option<&Paralyzed>,
            )>();
            for (entity, (stats, named, damageable, position, paralyzed_opt)) in &mut damageables {
                // if has been damaged and Stamina is 0, do a thougness saving throw or die.
                // On 0 or less toughness, die anyway
                if stats.current_stamina <= 0
                    && (damageable.damage_received > 0 || damageable.toughness_damage_received > 0)
                {
                    let saving_throw_roll = Roll::d20();
                    let is_killed =
                        stats.current_toughness < 1 || saving_throw_roll > stats.current_toughness;

                    // If killed, add to dead entities list with all necessary information
                    if is_killed {
                        dead_entities.push((
                            entity,
                            named.name.clone(),
                            (position.x, position.y),
                            damageable.damager,
                            stats.level,
                        ));
                    }

                    if entity.id() == player_entity_id {
                        if is_killed {
                            game_log.entries.push("You die!".to_string());
                        } else {
                            game_log.entries.push("You stagger in pain!".to_string());
                        }
                    } else if zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)]
                    {
                        // Log npc deaths only if visible by player
                        if is_killed {
                            game_log.entries.push(format!("{} dies!", named.name));
                        } else if stats.current_toughness > 0 {
                            game_log
                                .entries
                                .push(format!("{} staggers in pain!", named.name));
                        }
                    }
                }

                if stats.current_dexterity == 0 && paralyzed_opt.is_none() {
                    paralyzed_entities.push(entity);

                    if entity.id() == player_entity_id {
                        game_log.entries.push("You are paralyzed!".to_string());
                    } else {
                        game_log
                            .entries
                            .push(format!("{} is paralyzed!", named.name));
                    }
                }
                // Reset SufferingDamage component
                damageable.damage_received = 0;
                damageable.toughness_damage_received = 0;
                damageable.dexterity_damage_received = 0;
                damageable.damager = None;
            }
        }

        //Remove all dead entities, stop game if player is dead
        for (killed_entity, name, (x, y), damager_opt, victim_level) in dead_entities {
            if killed_entity.id() == player_entity_id {
                //Game over!
                game_state.run_state = RunState::GameOver;
                break;
            }

            // Award experience points to player if the victim was killed by him
            if let Some(damager) = damager_opt
                && damager.id() == player_entity_id
            {
                let mut experience = ecs_world
                    .get::<&mut Experience>(damager)
                    .expect("Player must have Experience component");

                experience.value += victim_level.pow(2);
                experience.auto_advance_counter += AUTO_ADVANCE_EXP_COUNTER_START;
            }

            ItemDropping::drop_all_of(killed_entity, ecs_world, x, y);

            // Create corpse if has no "DontLeaveCorpse" component
            // Change nutrition based on monster
            // The corpse carries the venom of the disease that the monster had (scorpions and beasts like that)
            // TODO it would be cool to make the corpse carry on the poison that killed him...
            if !ecs_world
                .satisfies::<&DontLeaveCorpse>(killed_entity)
                .unwrap_or(false)
            {
                let edible;
                // Scope for keeping borrow checker quiet
                {
                    let edible_ref = ecs_world
                        .get::<&Edible>(killed_entity)
                        .expect("killed_entity must be Edible");
                    edible = Edible {
                        nutrition_dice_number: edible_ref.nutrition_dice_number,
                        nutrition_dice_size: edible_ref.nutrition_dice_size,
                    }
                }

                let is_venomous = ecs_world.get::<&Venomous>(killed_entity).is_ok();
                let deadly = ecs_world.get::<&Deadly>(killed_entity).is_ok();
                let mut disease_type_opt: Option<DiseaseType> = None;
                if let Ok(disease_bearer) = ecs_world.get::<&DiseaseBearer>(killed_entity) {
                    disease_type_opt = Some(disease_bearer.disease_type.clone());
                };
                Spawn::corpse(
                    ecs_world,
                    (x, y, name, edible, is_venomous, deadly, disease_type_opt),
                );
            }

            //Despawn the killed entity anyway
            ecs_world
                .despawn(killed_entity)
                .expect("Cannot despawn entity");
        }

        // Handle paralysis
        for entity in paralyzed_entities {
            let _ = ecs_world.insert_one(entity, Paralyzed {});
        }

        false
    }
}
