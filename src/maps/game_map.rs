use std::collections::HashSet;

use hecs::Entity;
use macroquad::math::Rect;

use crate::constants::{MAP_HEIGHT, MAP_WIDTH};

#[derive(Clone, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
}

/// GameMap Struct
pub struct GameMap {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
    pub bloodied_tiles: HashSet<usize>,
    pub player_spawn_point: usize,
    pub monster_spawn_points: HashSet<usize>,
    pub item_spawn_points: HashSet<usize>,
}

/// GameMap Simplementations
impl GameMap {
    /// Create new empty map
    pub fn new() -> GameMap {
        GameMap {
            tiles: vec![TileType::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize],
            rooms: Vec::new(),
            revealed_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            visible_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            blocked_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            tile_content: vec![Vec::new(); (MAP_WIDTH * MAP_HEIGHT) as usize],
            player_spawn_point: 0,
            bloodied_tiles: HashSet::new(),
            monster_spawn_points: HashSet::new(),
            item_spawn_points: HashSet::new(),
        }
    }

    /// Gets which tile adjacent from a x,y position is passable
    pub fn get_adjacent_passable_tiles(
        &self,
        x_pos: i32,
        y_pos: i32,
        use_manhattan_distance: bool,
    ) -> Vec<(i32, i32)> {
        let mut adjacent_passable_tiles = Vec::new();

        for x in x_pos - 1..=x_pos + 1 {
            for y in y_pos - 1..=y_pos + 1 {
                //Manhattan Distance
                if !use_manhattan_distance || (x == x_pos || y == y_pos) {
                    let index = Self::get_index_from_xy(x, y);
                    // Safety check is needed for map borders
                    if self.blocked_tiles.len() > index && !self.blocked_tiles[index] {
                        adjacent_passable_tiles.push((x, y));
                    }
                }
            }
        }

        adjacent_passable_tiles
    }

    /// Populates the blocked tiles vector appropiately (true = is blocked )
    pub fn populate_blocked(&mut self) {
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            match tile {
                TileType::Floor => self.blocked_tiles[index] = false,
                _ => self.blocked_tiles[index] = true,
            }
        }
    }

    /// Return true if cannot see through a tile
    pub fn is_tile_opaque(&self, x: i32, y: i32) -> bool {
        let index = Self::get_index_from_xy(x, y);
        self.tiles[index] == TileType::Wall
    }

    /// Clears content index for this map
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Return a index inside the tile sheet
    pub fn get_tile_sprite_sheet_index(map: &GameMap, tile_to_draw: usize, x: i32, y: i32) -> f32 {
        match &map.tiles[tile_to_draw] {
            TileType::Floor => 0.0,
            TileType::Wall => GameMap::get_wall_tile(map, x, y),
        }
    }

    fn is_revealed_and_wall(map: &GameMap, x: i32, y: i32) -> bool {
        let idx = GameMap::get_index_from_xy(x, y);
        map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
    }

    fn get_wall_tile(map: &GameMap, x: i32, y: i32) -> f32 {
        // Boundaries
        if x < 1 || x > MAP_WIDTH - 2 || y < 1 || y > MAP_HEIGHT - 2 as i32 {
            return 17.0;
        }

        let mut mask: u8 = 0;

        if GameMap::is_revealed_and_wall(map, x, y - 1) {
            mask += 1;
        }
        if GameMap::is_revealed_and_wall(map, x, y + 1) {
            mask += 2;
        }
        if GameMap::is_revealed_and_wall(map, x - 1, y) {
            mask += 4;
        }
        if GameMap::is_revealed_and_wall(map, x + 1, y) {
            mask += 8;
        }

        return 1.0 + mask as f32;

        // match mask {
        //     0 => 1.0,  // Pillar because we can't see neighbors
        //     1 => 186,  // Wall only to the north
        //     2 => 186,  // Wall only to the south
        //     3 => 186,  // Wall to the north and south
        //     4 => 205,  // Wall only to the west
        //     5 => 188,  // Wall to the north and west
        //     6 => 187,  // Wall to the south and west
        //     7 => 185,  // Wall to the north, south and west
        //     8 => 205,  // Wall only to the east
        //     9 => 200,  // Wall to the north and east
        //     10 => 201, // Wall to the south and east
        //     11 => 204, // Wall to the north, south and east
        //     12 => 205, // Wall to the east and west
        //     13 => 202, // Wall to the east, west, and south
        //     14 => 203, // Wall to the east, west, and north
        //     15 => 206, // ╬ Wall on all sides
        //     _ => 1.0,  // We missed one?
        // }
    }

    /// trasfroms x,y position into a vector index
    pub fn get_index_from_xy(x: i32, y: i32) -> usize {
        ((y * MAP_WIDTH) + x) as usize
    }

    /// trasfroms x,y position into a vector index, using usizes
    pub fn get_index_from_xy_f32(x: f32, y: f32) -> usize {
        ((y as i32 * MAP_WIDTH) + x as i32) as usize
    }

    /// trasfroms x,y position into a vector index, using usizes
    pub fn get_xy_from_index(index: usize) -> (i32, i32) {
        let x = index % MAP_WIDTH as usize;
        let y = index / MAP_WIDTH as usize;
        (x as i32, y as i32)
    }
}
