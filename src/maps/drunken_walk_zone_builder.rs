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

        RiverBuilder::build(&mut zone);

        // Generate monster and items spawn points within each room
        let monster_number = Roll::dice(1, MAX_MONSTERS_ON_ROOM_START) + 2;
        let items_number = Roll::dice(1, MAX_ITEMS_ON_ROOM_START) + 3;
        let braziers_number = Roll::dice(1, MAX_BRAZIER_IN_ZONE);

        for _m in 0..monster_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH as i32 - 2) as f32;
                let y = Roll::dice(1, MAP_HEIGHT as i32 - 2) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid walls, player and duplicate spawnpoints
                if index != zone.player_spawn_point {
                    if zone.tiles[Zone::get_index_from_xy_f32(x, y)] != TileType::Wall {
                        if zone.monster_spawn_points.insert(index) {
                            break;
                        }
                    }
                }
            }
        }

        for _i in 0..items_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH as i32 - 2) as f32;
                let y = Roll::dice(1, MAP_HEIGHT as i32 - 2) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid walls, player and duplicate spawnpoints
                if index != zone.player_spawn_point {
                    if zone.tiles[Zone::get_index_from_xy_f32(x, y)] == TileType::Floor {
                        if zone.item_spawn_points.insert(index) {
                            break;
                        }
                    }
                }
            }
        }

        // Random braziers
        // FIXME not sure if its a good idea... for now, leave them
        for _i in 0..braziers_number {
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

        // Up passage is were player starts
        // TODO uncomment later when we can guarantee persistance
        // zone.tiles[zone.player_spawn_point] = TileType::UpPassage;

        zone
    }
}
