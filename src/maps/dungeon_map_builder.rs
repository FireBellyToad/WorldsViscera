use std::{
    cmp::{max, min},
    collections::HashSet,
};

use macroquad::prelude::*;

use crate::{
    constants::*,
    maps::game_map::{GameMap, TileType},
};

impl GameMap {
    /// Create new dungeon map (needed?)
    pub fn new_dungeon_map() -> Self {
        let mut map = GameMap {
            tiles: vec![TileType::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize],
            rooms: Vec::new(),
            revealed_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            visible_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            blocked_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            tile_content: vec![Vec::new(); (MAP_WIDTH * MAP_HEIGHT) as usize],
            bloodied_tiles: HashSet::new(),
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
                self.tiles[Self::get_index_from_xy(x, y)] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_corridor(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = Self::get_index_from_xy(x, y);
            if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_corridor(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = Self::get_index_from_xy(x, y);
            if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    /// Create new empty test map
    pub fn _new_arena_map() -> Self {
        let mut map = GameMap {
            tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
            rooms: Vec::new(),
            revealed_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            visible_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            blocked_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            tile_content: vec![Vec::new(); (MAP_WIDTH * MAP_HEIGHT) as usize],
            bloodied_tiles: HashSet::new(),
        };

        // Create bondaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1 {
                    let index = Self::get_index_from_xy(x, y);
                    map.tiles[index] = TileType::Wall
                }
            }
        }

        map
    }
}
