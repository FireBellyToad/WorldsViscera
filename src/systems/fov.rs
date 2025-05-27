use adam_fov_rs::{IVec2, compute_fov};
use hecs::World;

use crate::{
    components::{common::*, player::Player},
    constants::{MAP_HEIGHT, MAP_WIDTH},
    map::{Map, get_index_from_xy},
    utils::point::Point,
};

pub struct FovSystem {}

impl FovSystem {
    pub fn calculate_fov(ecs_world: &World) {
        let mut player_query = ecs_world.query::<(&Player)>();
        let (player_entity, _player) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        let mut map_query = ecs_world.query::<&mut Map>();
        let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

        //Deconstruct data into tuple
        let mut viewsheds = ecs_world.query::<(&mut Viewshed, &Position)>();
        //For each Entity with Components Viewshed and Position
        for (entity, (viewshed, position)) in &mut viewsheds {
            
            if viewshed.must_recalculate {
                viewshed.must_recalculate = false;
                viewshed.visible_tiles.clear();

                let fov_result = Self::compute_adam_fov(position.x, position.y, map);

                //recalculate rendered view if entity is Player 
                //TODO CHECK IF IS CORRECT
                if entity.id() == player_entity.id() {
                    map.visible_tiles.fill(false);
                    for (index, &is_tile_visible) in fov_result.iter().enumerate() {
                        if is_tile_visible {
                            map.revealed_tiles[index] = true;
                            map.visible_tiles[index] = true;
                        }
                    }
                }
            }
        }
    }

    fn compute_adam_fov(x_origin:i32, y_origin:i32, map: &mut Map) -> Vec<bool> {
        let mut visible_tiles_result = vec![false; (MAP_HEIGHT * MAP_WIDTH) as usize];

        let is_opaque = |position: IVec2| map.is_tile_opaque(position[0], position[1]);
        let set_to_visible = |position: IVec2| {
            let index = get_index_from_xy(position[0], position[1]);
            visible_tiles_result[index] = true;
        };

        compute_fov(
            Point { x: x_origin, y: y_origin },
            5,
            [MAP_WIDTH, MAP_HEIGHT],
            is_opaque,
            set_to_visible,
        );

        visible_tiles_result
    }
}
