use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::{
        ZoneFeatureBuilder,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

///River Builder
pub struct RiverBuilder {}

impl ZoneFeatureBuilder for RiverBuilder {
    fn build(zone: &mut Zone) {
        //TODO

        //1 - select a start point with X or Y = 1
        let mut current_position = (1, 1);
        if Roll::d100() <= 50 {
            current_position.1 = Roll::dice(1, MAP_HEIGHT / 2 - 1) + 1
        } else {
            current_position.0 = Roll::dice(1, MAP_WIDTH / 2 - 1) + 1
        }

        //2 - if point is X = MAP_WIDTH-1 or Y = MAP_HEIGHT-1, stop
        while current_position.0 < MAP_WIDTH-1 && current_position.1 < MAP_HEIGHT-1 {
            //3 - draw a water tile there
            zone.tiles[Zone::get_index_from_xy(current_position.0, current_position.1)] =
                TileType::Water;

            //4 - move to next tile down, right or left from previous tile.
            let new_direction_roll = Roll::dice(1, 2);
            let (mut dest_x, mut dest_y) = current_position;

            match new_direction_roll {
                1 => dest_x += 1,
                2 => dest_y += 1,
                _ => {}
            }

            current_position = (dest_x, dest_y);
            println!("current_position {:?}",current_position);
            println!("MAP_WIDTH {} MAP_HEIGHT {}", MAP_WIDTH, MAP_HEIGHT);
            //5 - go to step 2
        }
    }
}
