use std::cmp::{max, min};

use macroquad::prelude::*;

use crate::{
    constants::*,
    maps::{
        GameMapBuilder,
        game_map::{GameMap, TileType},
    },
};

/// Builds a simple dungeon-like map made of rooms and corridors
pub struct DungeonMapBuilder {}

impl GameMapBuilder for DungeonMapBuilder {
    /// Create new dungeon map (needed?)
    fn build() -> GameMap {
        let mut map = GameMap::new();

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
                Self::apply_room_to_map(&mut map, &new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center_to_i32_tuple();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center_to_i32_tuple();
                    if rand::gen_range(0, 2) == 1 {
                        Self::apply_horizontal_corridor(&mut map, prev_x, new_x, prev_y);
                        Self::apply_vertical_corridor(&mut map, prev_y, new_y, new_x);
                    } else {
                        Self::apply_vertical_corridor(&mut map, prev_y, new_y, prev_x);
                        Self::apply_horizontal_corridor(&mut map, prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

/// Other
impl DungeonMapBuilder {
    fn apply_room_to_map(game_map: &mut GameMap, room: &Rect) {
        for y in room.y as i32 + 1..(room.y + room.h) as i32 {
            for x in room.x as i32 + 1..(room.x + room.w) as i32 {
                game_map.tiles[GameMap::get_index_from_xy(x, y)] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_corridor(game_map: &mut GameMap, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = GameMap::get_index_from_xy(x, y);
            if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
                game_map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_corridor(game_map: &mut GameMap, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = GameMap::get_index_from_xy(x, y);
            if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
                game_map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }
}
