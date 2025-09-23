use std::cmp::max;

use crate::{
    constants::*,
    maps::{
        ZoneBuilder, ZoneFeatureBuilder,
        river_builder::RiverBuilder,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

/// Builds a cavern like zone with Drunken walk algorithm
pub struct DrunkenWalkZoneBuilder {}

impl ZoneBuilder for DrunkenWalkZoneBuilder {
    /// Create new dungeon zone (needed?)
    fn build(depth: i32) -> Zone {
        let mut zone = Zone::new(depth);

        // Simple Drunken walk
        let mut current_position = (MAP_WIDTH / 2, MAP_HEIGHT / 2);
        for _ in 0..DRUNKEN_WALK_MAX_ITERATIONS {
            zone.tiles[Zone::get_index_from_xy(current_position.0, current_position.1)] =
                TileType::Floor;

            let mut life_counter = 0;
            let mut unblock_tentatives = 10;
            while life_counter < DRUNKEN_WALK_LIFE_MAX {
                let new_direction_roll = Roll::dice(1, 4);
                let (mut dest_x, mut dest_y) = current_position;

                match new_direction_roll {
                    1 => dest_x += 1,
                    2 => dest_y += 1,
                    3 => dest_x -= 1,
                    4 => dest_y -= 1,
                    _ => {}
                }

                // Avoid boundaries, or else skip iteration
                if dest_x <= 1 || dest_x >= MAP_WIDTH - 1 || dest_y <= 1 || dest_y >= MAP_HEIGHT - 1
                {
                    if unblock_tentatives < 0 {
                        unblock_tentatives = 10;
                        current_position = (MAP_WIDTH / 2, MAP_HEIGHT / 2);
                    } else {
                        unblock_tentatives -= 1;
                        println!{"DrunkenWalkZoneBuilder - unblock_tentatives {}",unblock_tentatives};
                    }
                    continue;
                }

                let index = Zone::get_index_from_xy(dest_x, dest_y);
                if zone.tiles[index] == TileType::Wall {
                    zone.tiles[index] = TileType::Floor;
                }
                life_counter += 1;
                current_position = (dest_x, dest_y);
            }
            current_position = (Roll::dice(1, MAP_WIDTH - 2), Roll::dice(1, MAP_HEIGHT - 2));
        }

        // Random starting point for player
        let (mut try_x, mut try_y);
        zone.player_spawn_point = zone.tiles.len() / 2;
        while zone.tiles[zone.player_spawn_point] == TileType::Wall {
            try_x = Roll::dice(1, MAP_WIDTH - 2);
            try_y = Roll::dice(1, MAP_HEIGHT - 2);
            zone.player_spawn_point = Zone::get_index_from_xy(try_x, try_y);
        }

        let river_number = max(1, Roll::dice(0, MAX_RIVERS_IN_ZONE + (depth / 3)) - 3);
        for _ in 0..river_number {
            RiverBuilder::build(&mut zone);
        }
        // Populate water tiles here, needed for correct aquatic monster spawning
        zone.populate_water();

        // Generate monster and items spawn points within each room
        let monster_number = Roll::dice(1, MAX_MONSTERS_IN_ZONE) + depth + 1;
        let items_number = Roll::dice(1, MAX_ITEMS_IN_ZONE) + 3;
        let braziers_number = Roll::dice(1, MAX_BRAZIER_IN_ZONE);

        for _ in 0..monster_number {
            for _ in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH - 2) as f32;
                let y = Roll::dice(1, MAP_HEIGHT - 2) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid walls, player and duplicate spawnpoints
                if index != zone.player_spawn_point
                    && zone.tiles[Zone::get_index_from_xy_f32(x, y)] != TileType::Wall
                    && zone.monster_spawn_points.insert(index)
                {
                    break;
                }
            }
        }

        for _ in 0..items_number {
            for _ in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH - 2) as f32;
                let y = Roll::dice(1, MAP_HEIGHT - 2) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid walls, player and duplicate spawnpoints
                if index != zone.player_spawn_point
                    && zone.tiles[Zone::get_index_from_xy_f32(x, y)] == TileType::Floor
                    && zone.item_spawn_points.insert(index)
                {
                    break;
                }
            }
        }

        // Random braziers
        // FIXME not sure if its a good idea... for now, leave them
        for _ in 0..braziers_number {
            let mut brazier_index = zone.tiles.len() / 2;
            while zone.tiles[brazier_index] != TileType::Floor {
                try_x = Roll::dice(1, MAP_WIDTH - 2);
                try_y = Roll::dice(1, MAP_HEIGHT - 2);
                brazier_index = Zone::get_index_from_xy(try_x, try_y);
            }
            zone.tiles[brazier_index] = TileType::Brazier;
        }

        // Random point for DownPassage
        let mut passage_index = zone.tiles.len() / 2;
        while zone.tiles[passage_index] != TileType::Floor {
            try_x = Roll::dice(1, MAP_WIDTH - 2);
            try_y = Roll::dice(1, MAP_HEIGHT - 2);
            passage_index = Zone::get_index_from_xy(try_x, try_y);
        }
        zone.tiles[passage_index] = TileType::DownPassage;

        zone
    }
}
