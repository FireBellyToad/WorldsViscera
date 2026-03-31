use crate::{
    components::{
        common::{GrownIfSteppedOn, MyTurn, Position, SmellIntensity, Smellable},
        monster::{LeaveTrail, TrailCounter},
    },
    constants::CRYSTAL_GROWTH_COUNTER_START,
    engine::state::GameState,
    maps::zone::{DecalType, TileType, Zone},
};

pub struct SpecialTilesSystem {}

impl SpecialTilesSystem {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity = game_state.current_player_entity.expect("Must have player");
        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");

        // Scope for keeping borrow checker quiet
        {
            // List of entities that has stats
            let mut grown_if_stepped_on = ecs_world.query::<(&mut GrownIfSteppedOn, &Position)>();

            for (_, (grow_if_step, position)) in &mut grown_if_stepped_on {
                let pos_idx = Zone::get_index_from_xy(&position.x, &position.y);
                // if player is on the tile, decrement the counter
                if !zone.tile_content[pos_idx].is_empty()
                    && zone.tile_content[pos_idx]
                        .iter()
                        .any(|e| e.id() == player_entity.id())
                {
                    grow_if_step.counter_to_next_state -= 1;
                    if grow_if_step.counter_to_next_state == 0 {
                        grow_if_step.counter_to_next_state = CRYSTAL_GROWTH_COUNTER_START;

                        // Increase crystal growth stage when counter reaches 0
                        match zone.tiles[pos_idx] {
                            TileType::MiniCrystal => zone.tiles[pos_idx] = TileType::LittleCrystal,
                            TileType::LittleCrystal => {
                                zone.tiles[pos_idx] = TileType::MediumCrystal
                            }
                            TileType::MediumCrystal => zone.tiles[pos_idx] = TileType::BigCrystal,
                            _ => {}
                        };
                    }
                }
            }
        }
    }
}
