use std::ops::Index;

use adam_fov_rs::{IVec2, compute_fov};

use crate::{
    components::{common::*, health::Blind},
    constants::{MAP_HEIGHT, MAP_WIDTH},
    engine::state::GameState,
    maps::zone::Zone,
    utils::common::Utils,
};

use adam_fov_rs::GridPoint;

pub struct FieldOfViewManager {}

impl FieldOfViewManager {
    pub fn calculate(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");

        let mut remove_blindness_list = Vec::new();

        // Scope to keep the borrow checker happy
        {
            //Deconstruct data into tuple
            let mut viewsheds = ecs_world.query::<(&mut Viewshed, &Position, Option<&mut Blind>)>();
            //For each Entity with Components Viewshed and Position
            for (entity, (viewshed, position, blind_opt)) in &mut viewsheds {
                let is_player = entity.id() == player_entity_id;

                // Do not calculate FOV for blind entities
                if let Some(blind) = blind_opt {
                    // Can see only the tile were is standing
                    let index = Zone::get_index_from_xy(&position.x, &position.y);
                    viewshed.visible_tiles.clear();
                    viewshed.visible_tiles.push(index);
                    // After a while the blindness will be removed
                    blind.tick_counter -= 1;
                    if blind.tick_counter <= 0 {
                        remove_blindness_list.push(entity);
                        if is_player {
                            game_state
                                .game_log
                                .entries
                                .push("You can see again".to_string());
                            viewshed.must_recalculate = true;
                        }
                    }
                    if is_player {
                        //Show only the tile were is standing to the player
                        zone.visible_tiles.fill(false);
                        zone.visible_tiles[index] = true;
                        zone.revealed_tiles[index] = true;
                    }
                } else if viewshed.must_recalculate {
                    viewshed.must_recalculate = false;
                    viewshed.visible_tiles.clear();

                    FieldOfViewManager::compute(zone, viewshed, position.x, position.y);

                    //recalculate rendered view if entity is Player
                    if is_player {
                        zone.visible_tiles.fill(false);
                        for &index in viewshed.visible_tiles.iter() {
                            let (x, y) = Zone::get_xy_from_index(index);
                            let distance = Utils::distance(&x, &position.x, &y, &position.y);

                            // if is lit, that we can show and reveal
                            // Adiacent tiles are always visible
                            if zone.lit_tiles[index] || distance < 2.0 {
                                zone.revealed_tiles[index] = true;
                                zone.visible_tiles[index] = true;
                            }
                        }
                    }
                }
            }
        }

        for entity in remove_blindness_list {
            let _ = ecs_world.remove_one::<Blind>(entity);
        }
    }

    /// Wrapper to riutilize standard compute fov everywhere, given a viewshed
    pub fn compute(zone: &mut Zone, viewshed: &mut Viewshed, x: i32, y: i32) {
        // Utility lambda for opaque tiles
        let is_opaque = |position: IVec2| zone.is_tile_opaque(&position[0], &position[1]);

        // Utility lambda for setting visible tiles
        let set_to_visible = |position: IVec2| {
            viewshed
                .visible_tiles
                .push(Zone::get_index_from_xy(&position[0], &position[1]));
        };

        // Calculate Fov
        compute_fov(
            Point { x, y },
            viewshed.range as usize,
            [MAP_WIDTH, MAP_HEIGHT],
            is_opaque,
            set_to_visible,
        );

        // Sort visible tiles by distance from origin. Needed for better IA handling
        viewshed.visible_tiles.sort_by(|&a_index, &b_index| {
            let (ax, ay) = Zone::get_xy_from_index(a_index);
            let (bx, by) = Zone::get_xy_from_index(b_index);
            Utils::distance(&x, &ax, &y, &ay).total_cmp(&Utils::distance(&x, &bx, &y, &by))
        });
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl GridPoint for Point {
    fn xy(&self) -> adam_fov_rs::IVec2 {
        IVec2::new(self.x, self.y)
    }
}
