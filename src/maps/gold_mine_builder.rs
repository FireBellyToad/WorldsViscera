use std::cmp::max;

use hecs::World;

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH, MAX_GOLD_IN_ZONE},
    maps::{
        ZoneFeatureBuilder,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

///Gold Mines Builder
pub struct GoldMineBuilder {}

impl ZoneFeatureBuilder for GoldMineBuilder {
    fn build(zone: &mut Zone, _: &mut World) -> Vec<usize> {
        let mut gold_mine_tiles = Vec::new();
        let gold_mines_number = max(0, Roll::dice(1, MAX_GOLD_IN_ZONE) - 3);

        for _ in 0..gold_mines_number {
            let random_x = Roll::dice(1, MAP_WIDTH - 1);
            let random_y = Roll::dice(1, MAP_HEIGHT - 1);
            let index = Zone::get_index_from_xy(&random_x, &random_y);
            // if somehow reachable, place the mine
            if !zone
                .get_adjacent_passable_tiles(&random_x, &random_y, false, false)
                .is_empty()
            {
                zone.tiles[index] = TileType::GoldMine;
                gold_mine_tiles.push(index);
            }
        }

        gold_mine_tiles
    }
}
