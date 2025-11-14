use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::{
        ZoneFeatureBuilder, ZoneFeatureBuilderOrigin,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

///River Builder
pub struct RiverBuilder {}

impl ZoneFeatureBuilder for RiverBuilder {
    fn build(zone: &mut Zone) -> Vec<usize> {
        let mut river_tiles = Vec::new();
        let origin: ZoneFeatureBuilderOrigin;

        //1 - select a start point with X or Y = 1
        let mut current_position = (1, 1);
        if Roll::d100() <= 50 {
            current_position.1 = Roll::dice(1, MAP_HEIGHT - 1) + 1;
            origin = ZoneFeatureBuilderOrigin::Left;
        } else {
            current_position.0 = Roll::dice(1, MAP_WIDTH - 1) + 1;
            origin = ZoneFeatureBuilderOrigin::Top;
        }

        //2 - if point is X = MAP_WIDTH-1 or Y = MAP_HEIGHT-1, stop
        while current_position.0 < MAP_WIDTH - 1 && current_position.1 < MAP_HEIGHT - 1 {
            //3 - draw a water tile there
            let index = Zone::get_index_from_xy(&current_position.0, &current_position.1);
            zone.tiles[index] = TileType::Water;
            river_tiles.push(index);

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
                println!("RiverBuilder - Try again");
                continue;
            }

            current_position = (dest_x, dest_y);
            //5 - go to step 2
        }

        //6 - return the river tiles
        river_tiles
    }
}
