use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::{
        ZoneFeatureBuilder, ZoneFeatureBuilderOrigin,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

///Cracked wall paths Builder
pub struct CracksBuilder {}

impl ZoneFeatureBuilder for CracksBuilder {
    fn build(zone: &mut Zone) -> Vec<usize> {
        //1 - Start from the down passage
        let origin: ZoneFeatureBuilderOrigin;
        let mut touched_tiles = Vec::new();
        let start_index = zone
            .tiles
            .iter()
            .position(|tile| tile == &TileType::DownPassage);

        // If no down passage is available, start from player spawn point
        let mut current_position = if let Some(idx) = start_index {
            Zone::get_xy_from_index(idx)
        } else {
            Zone::get_xy_from_index(zone.player_spawn_point)
        };

        if Roll::d100() <= 50 {
            current_position.1 = Roll::dice(1, MAP_HEIGHT - 1) + 1;
            origin = ZoneFeatureBuilderOrigin::Left;
        } else {
            current_position.0 = Roll::dice(1, MAP_WIDTH - 1) + 1;
            origin = ZoneFeatureBuilderOrigin::Top;
        }
        //2 - if point is X = MAP_WIDTH-1 or Y = MAP_HEIGHT-1, stop
        while current_position.0 < MAP_WIDTH - 1 && current_position.1 < MAP_HEIGHT - 1 {
            //3 - draw a crack tile there if there is a wall tile
            let current_index = Zone::get_index_from_xy(&current_position.0, &current_position.1);
            touched_tiles.push(current_index);
            if zone.tiles[current_index] == TileType::Wall {
                zone.tiles[current_index] = TileType::CrackedWall;
            }

            //4 - move to next tile down, right or left from previous tile.
            let new_direction_roll = Roll::dice(1, 3);
            let (mut dest_x, mut dest_y) = current_position;

            // Avoid river turning back on itself
            match origin {
                ZoneFeatureBuilderOrigin::Top => match new_direction_roll {
                    1 => dest_x += 1,
                    2 => dest_y += 1,
                    3 => dest_x -= 1,
                    _ => {}
                },
                ZoneFeatureBuilderOrigin::Left => match new_direction_roll {
                    1 => dest_x += 1,
                    2 => dest_y += 1,
                    3 => dest_y -= 1,
                    _ => {}
                },
            }

            // Avoid boundaries, or else skip iteration
            if dest_x <= 1 || dest_x >= MAP_WIDTH || dest_y <= 1 || dest_y >= MAP_HEIGHT {
                println!("CracksBuilder - Try again");
                continue;
            }

            current_position = (dest_x, dest_y);
            //5 - go to step 2
        }

        touched_tiles
    }
}
