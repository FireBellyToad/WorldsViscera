

use bracket_lib::prelude::field_of_view;
use bracket_lib::prelude::Point;
use specs::System;
use specs::prelude::*;

use crate::components::common::{Position,Viewshed};
use crate::map::Map;
use crate::player::Player;

pub struct FovSystem {}

// FOV system
impl<'a> System<'a> for FovSystem {

    // SystemData is an alias of the tuple 
        type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    // System Trait implementation
    fn run(&mut self, data: Self::SystemData) {
        //Deconstruct data into tuple
        let (mut map, entities, mut viewsheds, positions, player) = data;

        //For each Entity with Components Viewshed and Position
        for (entity, viewshed, position) in (&entities, &mut viewsheds, &positions).join() {
            if viewshed.must_recalculate {
                viewshed.must_recalculate = false;
                viewshed.visible_tiles.clear();

                // get the field of view
                // NdF "map" is a pointer, so we need to take the value and then get a reference of it
                let map_instance_reference = &*map;
                viewshed.visible_tiles = field_of_view(
                    Point::new(position.x, position.y),
                    viewshed.range,
                    map_instance_reference,
                );

                // Keep only the points inside the map
                // |p| is a lambda
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // Reveal what the player can see
                // if entity is a player, calculate view
                let player_entity: Option<&Player> = player.get(entity);
                // reset current visible tiles
                map.visible_tiles.fill(false);
                if player_entity.is_some() {
                    for visible_tile in viewshed.visible_tiles.iter() {
                        let index = map.get_index_from_xy(visible_tile.x, visible_tile.y);
                        map.revealed_tiles[index] = true;
                        map.visible_tiles[index] = true;
                    }
                }
            }
        }
    }
}
