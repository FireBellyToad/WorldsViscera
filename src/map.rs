use std::{
    cmp::{max, min},
    collections::HashMap,
};

use hecs::Entity;
use macroquad::prelude::*;

use crate::{assets::TextureName, constants::*};

/// Map Struct and implementations
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
}
#[derive(Clone, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
}

impl Map {
    /// Create new dungeon map (needed?)
    pub fn new_dungeon_map() -> Self {
        let mut map = Map {
            tiles: vec![TileType::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize],
            rooms: Vec::new(),
            revealed_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            visible_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            blocked_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            tile_content: vec![Vec::new(); (MAP_WIDTH * MAP_HEIGHT) as usize],
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 3;
        const MAX_SIZE: i32 = 8;

        for _ in 0..MAX_ROOMS {
            let w = rand::gen_range(MIN_SIZE, MAX_SIZE);
            let h = rand::gen_range(MIN_SIZE, MAX_SIZE);
            let x = rand::gen_range(1, MAP_WIDTH - w - 1) - 1;
            let y = rand::gen_range(1, MAP_HEIGHT - h - 1) - 1;
            let new_room = Rect::new_from_i32(x, y, w, h);
            let mut room_not_overlaps = true;
            for other_room in map.rooms.iter() {
                if new_room.overlaps(other_room) {
                    room_not_overlaps = false
                }
            }
            if room_not_overlaps {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center_to_i32_tuple();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center_to_i32_tuple();
                    if rand::gen_range(0, 2) == 1 {
                        map.apply_horizontal_corridor(prev_x, new_x, prev_y);
                        map.apply_vertical_corridor(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_corridor(prev_y, new_y, prev_x);
                        map.apply_horizontal_corridor(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y as i32 + 1..(room.y + room.h) as i32 {
            for x in room.x as i32 + 1..(room.x + room.w) as i32 {
                self.tiles[get_index_from_xy(x, y)] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_corridor(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = get_index_from_xy(x, y);
            if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_corridor(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = get_index_from_xy(x, y);
            if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

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
                    if !self.blocked_tiles[get_index_from_xy(x, y)] {
                        adjacent_passable_tiles.push((x, y));
                    }
                }
            }
        }

        adjacent_passable_tiles
    }

    pub fn populate_blocked(&mut self) {
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            match tile {
                TileType::Floor => self.blocked_tiles[index] = false,
                _ => self.blocked_tiles[index] = true,
            }
        }
    }

    /// Create new empty test map
    pub fn _new_arena_map() -> Self {
        let mut map = Map {
            tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
            rooms: Vec::new(),
            revealed_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            visible_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            blocked_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            tile_content: vec![Vec::new(); (MAP_WIDTH * MAP_HEIGHT) as usize],
        };

        // Create bondaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1 {
                    let index = get_index_from_xy(x, y);
                    map.tiles[index] = TileType::Wall
                }
            }
        }

        map
    }

    /// Draws map
    pub fn draw(&self, assets: &HashMap<TextureName, Texture2D>) {
        let texture_to_render = assets.get(&TextureName::Tiles).expect("Texture not found");

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile_to_draw = get_index_from_xy(x, y);
                let tile_index =
                    get_tile_sprite_sheet_index(&self.tiles[tile_to_draw]) * TILE_SIZE as f32;

                if self.revealed_tiles[tile_to_draw] {
                    let mut alpha = DARKGRAY;
                    if self.visible_tiles[tile_to_draw] {
                        alpha = WHITE;
                    }
                    // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                    draw_texture_ex(
                        texture_to_render,
                        (UI_BORDER + (x * TILE_SIZE)) as f32,
                        (UI_BORDER + (y * TILE_SIZE)) as f32,
                        alpha,
                        DrawTextureParams {
                            source: Some(Rect {
                                x: tile_index,
                                y: 0.0,
                                w: TILE_SIZE as f32,
                                h: TILE_SIZE as f32,
                            }),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    /// Return true if cannot see through a tile
    pub fn is_tile_opaque(&self, x: i32, y: i32) -> bool {
        let index = get_index_from_xy(x, y);
        self.tiles[index] == TileType::Wall
    }

    // Clears content index for this map
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }
}

/// Return a index inside the tile sheet
fn get_tile_sprite_sheet_index(tile_type: &TileType) -> f32 {
    match tile_type {
        TileType::Floor => 0.0,
        TileType::Wall => 1.0,
    }
}

pub fn get_index_from_xy(x: i32, y: i32) -> usize {
    ((y * MAP_WIDTH) + x) as usize
}
