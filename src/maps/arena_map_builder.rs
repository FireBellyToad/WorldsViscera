use crate::{
    constants::*,
    maps::{
        GameMapBuilder,
        game_map::{GameMap, TileType},
    },
    utils::roll::Roll,
};

/// Builds a simple arena map, with only the boundary walls
pub struct ArenaMapBuilder {}

impl GameMapBuilder for ArenaMapBuilder {
    /// Create new dungeon map (needed?)
    fn build() -> GameMap {
        let mut map = GameMap::new();

        // Create boundaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x != 0 && y != 0 && x != MAP_WIDTH - 1 && y != MAP_HEIGHT - 1 {
                    let index = GameMap::get_index_from_xy(x, y);
                    map.tiles[index] = TileType::Floor
                }
            }
        }

        map.player_spawn_point = GameMap::get_index_from_xy(MAP_WIDTH / 2, MAP_HEIGHT / 2);

        // Generate monster and items spawn points within each room
        let monster_number = Roll::dice(1, MAX_MONSTERS_ON_ROOM_START);
        let items_number = Roll::dice(1, MAX_ITEMS_ON_ROOM_START);

        for _m in 0..monster_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = (Roll::dice(1, MAP_WIDTH as i32 - 1) as f32) as usize;
                let y = (Roll::dice(1, MAP_HEIGHT as i32 - 1) as f32) as usize;
                let index = (y * MAP_WIDTH as usize) + x;

                // avoid duplicate spawnpoints
                if map.monster_spawn_points.insert(index) {
                    break;
                }
            }
        }

        for _i in 0..items_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = (Roll::dice(1, MAP_WIDTH as i32 - 1) as f32) as usize;
                let y = (Roll::dice(1, MAP_HEIGHT as i32 - 1) as f32) as usize;
                let index = (y * MAP_WIDTH as usize) + x;

                // avoid duplicate spawnpoints
                if map.item_spawn_points.insert(index) {
                    break;
                }
            }
        }

        map
    }
}
