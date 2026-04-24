use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{Immunity, ImmunityTypeEnum, Named, Position, Species},
    },
    constants::CRYSTAL_GROWTH_COUNTER_START,
    engine::state::GameState,
    maps::zone::{TileType, Zone},
    spawning::spawner::Spawn,
    utils::roll::Roll,
};

/// Handles special tile interactions
pub struct SpecialTilesSystem {}

impl SpecialTilesSystem {
    pub fn grow_on_step_tiles(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state.current_player_entity.expect("must have Player");
        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");
        let mut spawn_tile_entities: Vec<(&TileType, usize)> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut live_entities = ecs_world
                .query::<(
                    &Position,
                    &mut SufferingDamage,
                    &CombatStats,
                    &Named,
                    &Immunity,
                )>()
                .with::<&Species>();

            for (
                stepper,
                (position, stepper_damage, stepper_stats, stepper_named, stepper_immunity),
            ) in &mut live_entities
            {
                let pos_idx = Zone::get_index_from_xy(&position.x, &position.y);

                if zone.tiles[pos_idx] == TileType::BigCrystal {
                    // if the Entity is stuck between BigCrystals, is crushed to death
                    if zone
                        .get_adjacent_passable_tiles(&position.x, &position.y, false, false)
                        .is_empty()
                    {
                        stepper_damage.damage_received =
                            stepper_stats.current_stamina + stepper_stats.current_toughness;
                        if stepper.id() == player_entity.id() {
                            game_state.game_log.entries.push(
                                "The crystals crush your body into a bloody pulp!".to_string(),
                            );
                            break;
                        } else if zone.visible_tiles[pos_idx] {
                            game_state.game_log.entries.push(format!(
                                "The crystals crush the {}'s body into a bloody pulp!",
                                stepper_named.name,
                            ));
                        }
                    }
                }
                // If there is a GrownIfSteppedOn component on the tile...
                // find the first entity with the component and get a mutable reference to it
                else if zone.tiles[pos_idx] == TileType::MiniCrystal
                    || zone.tiles[pos_idx] == TileType::LittleCrystal
                    || zone.tiles[pos_idx] == TileType::MediumCrystal
                {
                    // if player or NPC is on the tile, decrement the counter
                    // Get the entity and the name of the stepper. If present, decrement the counter
                    // We do not do wrapping subtraction here - the counter should never go below 0!!!
                    zone.special_tile_counter[pos_idx] =
                        zone.special_tile_counter[pos_idx].saturating_sub(1);

                    if zone.special_tile_counter[pos_idx] == 0 {
                        zone.special_tile_counter[pos_idx] = CRYSTAL_GROWTH_COUNTER_START;
                        // Increase crystal growth stage when counter reaches 0
                        match zone.tiles[pos_idx] {
                            TileType::MiniCrystal => {
                                zone.tiles[pos_idx] = TileType::LittleCrystal;
                                // Spread crystal growth to adjacent floor tiles
                                for x_diff in -1..2 {
                                    for y_diff in -1..2 {
                                        let pos_idx = Zone::get_index_from_xy(
                                            &(position.x + x_diff),
                                            &(position.y + y_diff),
                                        );
                                        if zone.tiles[pos_idx] == TileType::Floor {
                                            zone.tiles[pos_idx] = TileType::MiniCrystal;
                                            zone.special_tile_counter[pos_idx] =
                                                CRYSTAL_GROWTH_COUNTER_START;
                                        }
                                    }
                                }
                            }
                            TileType::LittleCrystal => {
                                zone.tiles[pos_idx] = TileType::MediumCrystal;
                            }
                            TileType::MediumCrystal => {
                                zone.tiles[pos_idx] = TileType::BigCrystal;
                                zone.special_tile_counter[pos_idx] = 0;
                                spawn_tile_entities.push((&TileType::BigCrystal, pos_idx));
                            }
                            _ => {}
                        };

                        // log crystal growth.
                        // If the stepper has not any immunity to damaging floors,
                        // inflicts damage to the stepper.
                        if stepper_immunity
                            .to
                            .contains_key(&ImmunityTypeEnum::DamagingFloor)
                        {
                            if stepper.id() == player_entity.id() {
                                game_state
                                    .game_log
                                    .entries
                                    .push("The crystals grow under your feet".to_string());
                            } else if zone.visible_tiles[pos_idx] {
                                game_state.game_log.entries.push(format!(
                                    "The crystals grow under {}'s feet",
                                    stepper_named.name,
                                ));
                            }
                        } else if Roll::d20() > stepper_stats.current_dexterity {
                            stepper_damage.damage_received += 1;
                            if stepper.id() == player_entity.id() {
                                game_state
                                    .game_log
                                    .entries
                                    .push("The crystals stings your bare feet".to_string());
                                break;
                            } else if zone.visible_tiles[pos_idx] {
                                game_state.game_log.entries.push(format!(
                                    "The crystals stings {}'s bare feet",
                                    stepper_named.name,
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Spawn tile entities if something changed
        for (tile_type, pos_idx) in spawn_tile_entities {
            let (x, y) = Zone::get_xy_from_index(pos_idx);
            Spawn::tile_entity(ecs_world, x, y, tile_type);
        }
    }
}
