use std::{
    cmp::{max, min},
    collections::HashMap,
};

use macroquad::prelude::*;

use crate::{assets::TextureName, constants::*};

/// Map Struct and implementations
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>
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
            rooms: Vec::new()
        };

        // Generate new seed, or else it will always generate the same layout
        rand::srand(macroquad::miniquad::date::now() as _);

        const MAX_ROOMS: i32 = 20;
        const MIN_SIZE: i32 = 4;
        const MAX_SIZE: i32 = 8;

        for _ in 0..MAX_ROOMS {
            let w = rand::gen_range(MIN_SIZE, MAX_SIZE) as f32;
            let h = rand::gen_range(MIN_SIZE, MAX_SIZE) as f32;
            let x = (rand::gen_range(1.0, MAP_WIDTH as f32 - w - 1.0) - 1.0) as f32;
            let y = (rand::gen_range(1.0, MAP_HEIGHT as f32 - h - 1.0) - 1.0) as f32;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.overlaps(other_room) {
                    ok = false
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let new_xy = new_room.center();
                    let prev_xy = map.rooms[map.rooms.len() - 1].center();
                    if rand::gen_range(0, 2) == 1 {
                        map.apply_horizontal_corridor(prev_xy[0] as i32, new_xy[0]  as i32, prev_xy[1]  as i32);
                        map.apply_vertical_corridor(prev_xy[1]  as i32, new_xy[1]  as i32, new_xy[0]  as i32);
                    } else {
                        map.apply_vertical_corridor(prev_xy[1]  as i32, new_xy[1]  as i32, prev_xy[0]  as i32);
                        map.apply_horizontal_corridor(prev_xy[0]  as i32, new_xy[0]  as i32, new_xy[1]  as i32);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y as i32 + 1..=(room.y + room.h) as i32 {
            for x in room.x as i32 + 1..=(room.x + room.w) as i32 {
                let index = self.get_index_from_xy(x, y);
                self.tiles[index] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_corridor(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.get_index_from_xy(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_corridor(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.get_index_from_xy(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    /// Create new empty test map
    pub fn new_test_map() -> Self {
        let mut map = Map {
            tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
            rooms: Vec::new()
        };

        // Create bondaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1 {
                    let index = map.get_index_from_xy(x, y);
                    map.tiles[index] = TileType::Wall
                }
            }
        }

        map
    }

    /// Draws map
    pub fn draw_map(&self, assets: &HashMap<TextureName, Texture2D>) {
        let texture_to_render = assets.get(&TextureName::Tiles).expect("Texture not found");

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile_to_draw = self.get_index_from_xy(x, y);
                let tile_index = self.get_tile_index(&self.tiles[tile_to_draw]) * TILE_SIZE as f32;

                // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                draw_texture_ex(
                    texture_to_render,
                    (UI_BORDER + (x * TILE_SIZE)) as f32,
                    (UI_BORDER + (y * TILE_SIZE)) as f32,
                    WHITE,
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

    fn get_tile_index(&self, tile_type: &TileType) -> f32 {
        match tile_type {
            TileType::Floor => 0.0,
            TileType::Wall => 1.0,
        }
    }

    pub fn get_index_from_xy(&self, x: i32, y: i32) -> usize {
        ((y * MAP_WIDTH) + x) as usize
    }
}
