use hecs::World;

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::{
        ZoneBuilder,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

pub struct CrystalCaveBuilder {}

/// Builder for the Crystal Cave specialzone.
impl ZoneBuilder for CrystalCaveBuilder {
    fn build(depth: u32, ecs_world: &mut World) -> Zone {
        let mut zone = Zone::new(depth, TileType::BigCrystal);

        // Create boundaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x != 0 && y != 0 && x != MAP_WIDTH - 1 && y != MAP_HEIGHT - 1 {
                    let index = Zone::get_index_from_xy(&x, &y);

                    if Roll::d20() <= 3 {
                        zone.tiles[index] = TileType::LittleCrystal;
                    } else if Roll::d20() == 1 {
                        zone.tiles[index] = TileType::MediumCrystal;
                    } else {
                        zone.tiles[index] = TileType::MiniCrystal;
                    }
                }
            }
        }

        zone.player_spawn_point = Zone::get_index_from_xy(&(MAP_WIDTH / 2), &(MAP_HEIGHT / 2));

        zone
    }
}
