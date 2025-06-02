use adam_fov_rs::{IVec2, compute_fov};
use hecs::World;

use crate::{
    components::{common::*, player::Player},
    constants::{MAP_HEIGHT, MAP_WIDTH},
    map::{get_index_from_xy, Map},
    utils::point::Point,
};

pub struct FovCalculator {}

impl FovCalculator {
    pub fn run(ecs_world: &World) {

        let player_entity_id = Player::get_player_id(ecs_world);

        let mut map_query = ecs_world.query::<&mut Map>();
        let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

        //Deconstruct data into tuple
        let mut viewsheds = ecs_world.query::<(&mut Viewshed, &Position)>();
        //For each Entity with Components Viewshed and Position
        for (entity, (viewshed, position)) in &mut viewsheds {
            if viewshed.must_recalculate {
                viewshed.must_recalculate = false;
                viewshed.visible_tiles.clear();

                // Utility lambda for opaque tiles
                let is_opaque = |position: IVec2| map.is_tile_opaque(position[0], position[1]);
                // Utility lambda for setting visible tiles
                let set_to_visible = |position: IVec2| {
                    viewshed.visible_tiles.push(Point {
                        x: position[0],
                        y: position[1],
                    });
                };

                // Calculate Fov
                compute_fov(
                    Point {
                        x: position.x,
                        y: position.y,
                    },
                    viewshed.range as usize,
                    [MAP_WIDTH, MAP_HEIGHT],
                    is_opaque,
                    set_to_visible,
                );

                //recalculate rendered view if entity is Player
                if entity.id() == player_entity_id {
                    map.visible_tiles.fill(false);
                    for &tile in viewshed.visible_tiles.iter() {
                        let index = get_index_from_xy(tile.x, tile.y);
                        map.revealed_tiles[index] = true;
                        map.visible_tiles[index] = true;
                    }
                }
            }
        }
    }
}
