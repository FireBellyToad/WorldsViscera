use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, Grappled, SufferingDamage},
        common::{Experience, Hates, Named, Position, ProduceCorpse, Species, SpeciesEnum},
        health::{CanAutomaticallyHeal, DiseaseType, Paralyzed},
        items::{Deadly, Edible},
        monster::{DiseaseBearer, SingleSnakeCreature, SnakeBody, SnakeHead, Venomous},
    },
    constants::{AUTO_ADVANCE_EXP_COUNTER_START, MAX_STAMINA_HEAL_TICK_COUNTER},
    engine::state::{GameState, RunState},
    maps::zone::{DecalType, Zone},
    spawning::spawner::{CorpseSpawnData, Spawn},
    systems::item_dropping::ItemDropping,
    utils::roll::Roll,
};

type DeadEntityData = (Entity, &'static str, (i32, i32), Option<Entity>, u32);

pub struct DamageManager {}

/// Damage manager system
impl DamageManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");

        // Transfer the SufferingDamage data of a SnakeBody to the head
        // After that, do normal damage system cycle.
        let mut damageable_snake_bodies = ecs_world
            .query::<(&mut SufferingDamage, &SnakeBody, &Position)>()
            .with::<&SingleSnakeCreature>();

        if damageable_snake_bodies.iter().len() > 0 {
            for (_, (damageable, snake_body, position)) in &mut damageable_snake_bodies {
                let mut head_damage = ecs_world
                    .get::<&mut SufferingDamage>(snake_body.head)
                    .expect("Head must have SufferingDamage");

                if damageable.damage_received > 0 {
                    head_damage.damage_received += damageable.damage_received;
                    damageable.damage_received = 0;
                    //Drench the tile with blood
                    zone.decals_tiles.insert(
                        Zone::get_index_from_xy(&position.x, &position.y),
                        DecalType::Blood,
                    );
                }

                // Venomous hits
                if damageable.toughness_damage_received > 0 {
                    head_damage.toughness_damage_received += damageable.toughness_damage_received;
                    damageable.toughness_damage_received = 0;
                }

                // Disease hits on dexterity
                if damageable.dexterity_damage_received > 0 {
                    head_damage.dexterity_damage_received += damageable.dexterity_damage_received;
                    damageable.dexterity_damage_received = 0;
                }
            }
        }

        let mut damageables = ecs_world.query::<(
            &mut SufferingDamage,
            &mut CombatStats,
            &Position,
            Option<&mut CanAutomaticallyHeal>,
        )>();

        for (damaged_entity, (damageable, stats, position, can_automatically_heal_opt)) in
            &mut damageables
        {
            let mut must_reset_heal_counter = false;
            if damageable.damage_received > 0 {
                // From now on, damaged entity will be hostile to its damager
                if let Some(damager) = damageable.damager
                    && let Ok(mut target_hates) = ecs_world.get::<&mut Hates>(damaged_entity)
                {
                    target_hates.list.insert(damager.id());
                }

                must_reset_heal_counter = true;
                stats.current_stamina -= damageable.damage_received;
                //Decrease stamina. If less then 0, delta is subtracted from toughness
                if stats.current_stamina < 0 {
                    // We add a negative value
                    stats.current_toughness += stats.current_stamina;
                    stats.current_stamina = max(0, stats.current_stamina);
                }

                //Drench the tile with blood
                zone.decals_tiles.insert(
                    Zone::get_index_from_xy(&position.x, &position.y),
                    DecalType::Blood,
                );
            }

            // Venomous hits
            if damageable.toughness_damage_received > 0 {
                must_reset_heal_counter = true;
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
                must_reset_heal_counter = true;
                stats.current_dexterity = max(
                    0,
                    stats.current_dexterity - damageable.dexterity_damage_received,
                );
            }

            // Reset heal counter for stamina regen
            if must_reset_heal_counter
                && let Some(can_heal) = can_automatically_heal_opt
                && stats.current_stamina < stats.max_stamina
            {
                can_heal.tick_counter = MAX_STAMINA_HEAL_TICK_COUNTER + 2;
            }
        }
    }

    /// Check which entities are dead and removes them. Returns true if Player is dead
    pub fn remove_dead_and_check_gameover(game_state: &mut GameState) -> bool {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");
        let mut dead_entities: Vec<DeadEntityData> = Vec::new();
        let mut paralyzed_entities: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
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
                            named.name,
                            (position.x, position.y),
                            damageable.damager,
                            stats.level,
                        ));
                    }

                    if entity.id() == player_id {
                        if is_killed {
                            game_state.game_log.add_entry("You die!");
                        } else {
                            game_state.game_log.add_entry("You stagger in pain!");
                        }
                    } else if zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)]
                    {
                        // Log npc deaths only if visible by player
                        if is_killed {
                            game_state
                                .game_log
                                .add_entry(&format!("{} dies!", named.name));
                        } else if stats.current_toughness > 0 {
                            game_state
                                .game_log
                                .add_entry(&format!("{} staggers in pain!", named.name));
                        }
                    }
                }

                if stats.current_dexterity == 0 && paralyzed_opt.is_none() {
                    paralyzed_entities.push(entity);

                    if entity.id() == player_id {
                        game_state.game_log.add_entry("You are paralyzed!");
                    } else {
                        game_state
                            .game_log
                            .add_entry(&format!("{} is paralyzed!", named.name));
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
            if killed_entity.id() == player_id {
                //Game over!
                game_state.run_state = RunState::GameOver;
                break;
            }

            // Award experience points to player if the victim was killed by him
            if let Some(damager) = damager_opt
                && damager.id() == player_id
            {
                let mut experience = ecs_world
                    .get::<&mut Experience>(damager)
                    .expect("Player must have Experience component");

                experience.value += victim_level.pow(2);
                experience.auto_advance_counter += AUTO_ADVANCE_EXP_COUNTER_START;
            }

            DamageManager::handle_snake_entity_death(ecs_world, zone, killed_entity, damager_opt);
            DamageManager::handle_grappler_death(ecs_world, killed_entity);

            ItemDropping::drop_all_of(killed_entity, ecs_world, x, y);

            // Create corpse if has "ProduceCorpse" component
            // Change nutrition based on monster
            // The corpse carries the venom of the disease that the monster had (scorpions and beasts like that)
            // TODO it would be cool to make the corpse carry on the poison that killed him...
            if ecs_world
                .satisfies::<&ProduceCorpse>(killed_entity)
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

                let is_venomous = ecs_world
                    .satisfies::<&Venomous>(killed_entity)
                    .unwrap_or(false);
                let is_deadly = ecs_world
                    .satisfies::<&Deadly>(killed_entity)
                    .unwrap_or(false);
                let mut disease_type_opt: Option<DiseaseType> = None;
                if let Ok(disease_bearer) = ecs_world.get::<&DiseaseBearer>(killed_entity) {
                    disease_type_opt = Some(disease_bearer.disease_type.clone());
                };
                let is_undead = if let Ok(species) = ecs_world.get::<&Species>(killed_entity)
                    && species.value == SpeciesEnum::Undead
                {
                    true
                } else {
                    false
                };

                Spawn::corpse(
                    ecs_world,
                    CorpseSpawnData {
                        x,
                        y,
                        name,
                        edible,
                        is_venomous,
                        is_deadly,
                        disease_type_opt,
                        is_undead,
                    },
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

    /// Handle snake entity death.
    /// Is is SingleSnakeCreature, kill head and despawn body parts..
    /// Else, is a moltitude of creatures (like a Stonedust cultist procession):
    /// just free them all and let them act normally (will hate the killer, though)
    fn handle_snake_entity_death(
        ecs_world: &mut hecs::World,
        zone: &mut Zone,
        killed_entity: Entity,
        damager_opt: Option<Entity>,
    ) {
        let mut body_temp_vec: Vec<(Entity, bool, bool)> = Vec::new();
        let is_single_creature = ecs_world
            .satisfies::<&SingleSnakeCreature>(killed_entity)
            .unwrap_or(false);
        if let Ok(head) = ecs_world.get::<&SnakeHead>(killed_entity) {
            for &body_entity in head.body.iter() {
                body_temp_vec.push((body_entity, is_single_creature, true));
                let position = ecs_world
                    .get::<&Position>(body_entity)
                    .expect("Snake body should have position");
                //Drench the tile of the body with blood
                zone.decals_tiles.insert(
                    Zone::get_index_from_xy(&position.x, &position.y),
                    DecalType::Blood,
                );
            }
        } else if !is_single_creature
            && let Ok(body) = ecs_world.get::<&SnakeBody>(killed_entity)
            && let Ok(head) = ecs_world.get::<&SnakeHead>(body.head)
        {
            for &body_entity in head.body.iter() {
                body_temp_vec.push((body_entity, is_single_creature, true));
            }
            body_temp_vec.push((body.head, is_single_creature, false));
        }

        for (snake_entity, is_single_creature, is_body_part) in body_temp_vec {
            if is_single_creature {
                // Despawn the snake body parts if is just a single creature
                let _ = ecs_world.despawn(snake_entity);
            } else {
                // Other members of the snake will now hate the damager
                if let Some(damager) = damager_opt
                    && let Ok(mut target_hates) = ecs_world.get::<&mut Hates>(snake_entity)
                {
                    target_hates.list.insert(damager.id());
                }
                // And they will be free to act by themselves
                if is_body_part {
                    let _ = ecs_world.remove_one::<SnakeBody>(snake_entity);
                } else {
                    let _ = ecs_world.remove_one::<SnakeHead>(snake_entity);
                }
            }
        }
    }

    /// Handles the death of a grappler, removing any Grappled components from entities.
    fn handle_grappler_death(ecs_world: &mut World, killed_entity: Entity) {
        let mut grappled_by_killed: Vec<Entity> = Vec::new();
        // Scope to keep the borrow checker quiet
        {
            let mut grappled = ecs_world.query::<&Grappled>();

            for (entity, grappled) in &mut grappled.iter() {
                if grappled.by.id() == killed_entity.id() {
                    grappled_by_killed.push(entity);
                }
            }
        }

        for grappler in grappled_by_killed {
            let _ = ecs_world.remove_one::<Grappled>(grappler);
        }
    }
}
