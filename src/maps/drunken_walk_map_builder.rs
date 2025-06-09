use crate::{
    constants::*,
    maps::{
        GameMapBuilder,
        game_map::{GameMap, TileType},
    },
    utils::roll::Roll,
};

/// Builds a cavern like map with Drunken walk algorithm
pub struct DrunkenWalkMapBuilder {}

impl GameMapBuilder for DrunkenWalkMapBuilder {
    /// Create new dungeon map (needed?)
    fn build() -> GameMap {
        let mut map = GameMap::new();

        // Simple Drunken walk
        let mut current_position = (MAP_WIDTH / 2, MAP_HEIGHT / 2);
        for _ in 0..DRUNKEN_WALK_MAX_ITERATIONS {
            map.tiles[GameMap::get_index_from_xy(current_position.0, current_position.1)] =
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
                if dest_x <= 0 || dest_x >= MAP_WIDTH - 1 || dest_y <= 0 || dest_y >= MAP_HEIGHT - 1
                {
                    continue;
                }

                let index = GameMap::get_index_from_xy(dest_x, dest_y);
                if map.tiles[index] == TileType::Wall {
                    map.tiles[index] = TileType::Floor;
                }
                life_counter += 1;
                current_position = (dest_x, dest_y);
            }
            current_position = (Roll::dice(1, MAP_WIDTH - 1), Roll::dice(1, MAP_HEIGHT - 1));
        }

        // Make sure whe have boundaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1 {
                    let index = GameMap::get_index_from_xy(x, y);
                    map.tiles[index] = TileType::Wall
                }
            }
        }

        // Random starting point for palyer
        let (mut try_x, mut try_y);
        map.player_spawn_point = map.tiles.len() / 2;
        // TODO stairs
        while map.tiles[map.player_spawn_point] == TileType::Wall {
            try_x = Roll::dice(1, MAP_WIDTH - 1);
            try_y = Roll::dice(1, MAP_HEIGHT - 1);
            map.player_spawn_point = GameMap::get_index_from_xy(try_x, try_y);
        }

        // Generate monster and items spawn points within each room
        let monster_number = Roll::dice(1, MAX_MONSTERS_ON_ROOM_START) + 2;
        let items_number = Roll::dice(1, MAX_ITEMS_ON_ROOM_START) + 3;

        for _m in 0..monster_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH as i32 - 1) as f32;
                let y = Roll::dice(1, MAP_HEIGHT as i32 - 1) as f32;
                let index = GameMap::get_index_from_xy_f32(x, y);

                // avoid duplicate spawnpoints
                if map.tiles[GameMap::get_index_from_xy_f32(x, y)] != TileType::Wall {
                    if map.monster_spawn_points.insert(index) {
                        break;
                    }
                }
            }
        }

        for _i in 0..items_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH as i32 - 1) as f32;
                let y = Roll::dice(1, MAP_HEIGHT as i32 - 1) as f32;
                let index = GameMap::get_index_from_xy_f32(x, y);

                // avoid duplicate spawnpoints
                if map.tiles[GameMap::get_index_from_xy_f32(x, y)] != TileType::Wall {
                    if map.item_spawn_points.insert(index) {
                        break;
                    }
                }
            }
        }

        map
    }
}
