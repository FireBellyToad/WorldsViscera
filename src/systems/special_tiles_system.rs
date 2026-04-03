use hecs::Entity;

use crate::{
    components::{
        combat::{CombatStats, SufferingDamage},
        common::{GrownIfSteppedOn, Immunity, ImmunityTypeEnum, Named, Position},
    },
    constants::CRYSTAL_GROWTH_COUNTER_START,
    engine::state::GameState,
    maps::zone::{TileType, Zone},
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

        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut grown_if_stepped_on = ecs_world.query::<(&mut GrownIfSteppedOn, &Position)>();
            let mut live_entities =
                ecs_world.query::<(&mut SufferingDamage, &CombatStats, &Named, &Immunity)>();

            for (_, (grow_if_step, position)) in &mut grown_if_stepped_on {
                let pos_idx = Zone::get_index_from_xy(&position.x, &position.y);

                // if player or NPC is on the tile, decrement the counter
                // Get the entity and the name of the stepper. If present, decrement the counter
                let stepper_search_result: Option<(
                    Entity,
                    &mut SufferingDamage,
                    &CombatStats,
                    &Named,
                    &Immunity,
                )> = live_entities
                    .iter()
                    .filter_map(|(le, (damage, stats, named, immunity))| {
                        if zone.tile_content[pos_idx].iter().any(|e| le.id() == e.id()) {
                            Some((le, damage, stats, named, immunity))
                        } else {
                            None
                        }
                    })
                    .next();

                if let Some((
                    stepper,
                    stepper_damage,
                    stepper_stats,
                    stepper_named,
                    stepper_immunity,
                )) = stepper_search_result
                {
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
                    } else {
                        grow_if_step.counter_to_next_state -= 1;
                        if grow_if_step.counter_to_next_state == 0 {
                            grow_if_step.counter_to_next_state = CRYSTAL_GROWTH_COUNTER_START;

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
                                    break;
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
                            // Increase crystal growth stage when counter reaches 0
                            match zone.tiles[pos_idx] {
                                TileType::MiniCrystal => {
                                    zone.tiles[pos_idx] = TileType::LittleCrystal;
                                }
                                TileType::LittleCrystal => {
                                    zone.tiles[pos_idx] = TileType::MediumCrystal
                                }
                                TileType::MediumCrystal => {
                                    zone.tiles[pos_idx] = TileType::BigCrystal;
                                    grow_if_step.counter_to_next_state = 0;
                                }
                                _ => {}
                            };
                        }
                    }
                }
            }
        }
    }
}
