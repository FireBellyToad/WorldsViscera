use hecs::World;

use crate::{
    constants::{CRYSTAL_GROWTH_COUNTER_START, GOLD_LOCK_COUNTER_START, MAP_HEIGHT, MAP_WIDTH},
    maps::{
        ZoneBuilder,
        zone::{TileType, Zone},
    },
    spawning::spawner::Spawn,
    utils::roll::Roll,
};

pub struct CrystalCaveBuilder {}

/// Builder for the Crystal Cave specialzone.
impl ZoneBuilder for CrystalCaveBuilder {
    fn build(depth: u32, ecs_world: &mut World) -> Zone {
        let mut zone = Zone::new(depth, TileType::BigCrystal);
        zone.special_tile_counter =
            [CRYSTAL_GROWTH_COUNTER_START; (MAP_WIDTH * MAP_HEIGHT) as usize];

        // Create boundaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x != 0 && y != 0 && x != MAP_WIDTH - 1 && y != MAP_HEIGHT - 1 {
                    let index = Zone::get_index_from_xy(&x, &y);

                    if Roll::d20() <= 6 {
                        zone.tiles[index] = TileType::LittleCrystal;
                    } else if Roll::d20() <= 3 {
                        zone.tiles[index] = TileType::MediumCrystal;
                    } else {
                        zone.tiles[index] = TileType::MiniCrystal;
                    }
                }
            }
        }

        let player_x = &((MAP_WIDTH / 2) - Roll::dice(2, 3));
        zone.player_spawn_point = Zone::get_index_from_xy(player_x, &1);

        zone.tiles[Zone::get_index_from_xy(&(MAP_WIDTH / 2), &(MAP_HEIGHT / 2))] =
            TileType::TripleGoldLock(GOLD_LOCK_COUNTER_START);

        let lock_entity = Spawn::tile_entity(
            ecs_world,
            MAP_WIDTH / 2,
            MAP_HEIGHT / 2,
            &TileType::TripleGoldLock(GOLD_LOCK_COUNTER_START),
        );

        // Place keys to activate down passage in the zone
        // Something like this:
        // |...k...|
        // |.......|
        // |k.....k|
        // TODO improve randomicity
        for (key_x, key_y) in [
            (MAP_WIDTH / 2, 1),
            (1, MAP_HEIGHT - 2),
            (MAP_WIDTH - 2, MAP_HEIGHT - 2),
        ] {
            let _ = Spawn::gold_key(
                ecs_world,
                key_x,
                key_y,
                lock_entity.expect("must have lock entity"),
            );
        }

        // place human refugees in random locations
        for _ in 0..6 {
            Spawn::refugee(
                ecs_world,
                Roll::dice(1, MAP_WIDTH - 2),
                Roll::dice(1, MAP_HEIGHT - 2),
            );
        }

        zone
    }
}
