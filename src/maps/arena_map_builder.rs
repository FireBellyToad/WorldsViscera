use crate::{
    constants::*,
    maps::{
        GameMapBuilder,
        game_map::{GameMap, TileType},
    },
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
                if x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1 {
                    let index = GameMap::get_index_from_xy(x, y);
                    map.tiles[index] = TileType::Wall
                }
            }
        }

        map
    }
}
