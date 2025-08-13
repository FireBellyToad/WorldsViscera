use std::collections::{HashMap, HashSet};

use hecs::Entity;
use macroquad::math::Rect;

use crate::constants::{MAP_HEIGHT, MAP_WIDTH};

#[derive(Clone, Debug, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    DownPassage,
    UpPassage,
    Brazier,
    Water
}
pub enum ParticleType {
    Blood,
    Vomit,
}

/// Zone Struct
pub struct Zone {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub lit_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
    pub particle_tiles: HashMap<usize, ParticleType>,
    pub depth: i32,
    pub player_spawn_point: usize,
    pub monster_spawn_points: HashSet<usize>,
    pub item_spawn_points: HashSet<usize>,
}

/// Zone Simplementations
impl Zone {
    /// Create new empty zone
    pub fn new(depth: i32) -> Zone {
        Zone {
            tiles: vec![TileType::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize],
            rooms: Vec::new(),
            revealed_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            visible_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            lit_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            blocked_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            tile_content: vec![Vec::new(); (MAP_WIDTH * MAP_HEIGHT) as usize],
            player_spawn_point: 0,
            depth,
            particle_tiles: HashMap::new(),
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
                    // Safety check is needed for zone borders
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
                TileType::DownPassage | TileType::UpPassage | TileType::Floor => {
                    self.blocked_tiles[index] = false
                }
                _ => self.blocked_tiles[index] = true,
            }
        }
    }

    /// Return true if cannot see through a tile
    pub fn is_tile_opaque(&self, x: i32, y: i32) -> bool {
        let index = Self::get_index_from_xy(x, y);
        self.tiles[index] == TileType::Wall
    }

    /// Clears content index for this zone
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Return a index inside the tile sheet
    pub fn get_tile_sprite_sheet_index(tile_type: &TileType) -> (f32,f32) {
        match tile_type {
            TileType::Floor => (0.0,0.0),
            TileType::Wall => (1.0,0.0),
            TileType::DownPassage => (2.0,0.0),
            TileType::UpPassage => (3.0,0.0),
            TileType::Brazier => (4.0,0.0),
            TileType::Water => (0.0,1.0),
        }
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
        let x = index as i32 % MAP_WIDTH;
        let y = index as i32 / MAP_WIDTH;
        (x as i32, y as i32)
    }
}
