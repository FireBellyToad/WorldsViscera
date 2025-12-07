use std::cmp::{max, min};

use hecs::World;
use macroquad::prelude::*;

use crate::{
    constants::*,
    maps::{
        ZoneBuilder,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

/// Builds a simple dungeon-like zone made of rooms and corridors
#[allow(dead_code)]
pub struct DungeonZoneBuilder {}

impl ZoneBuilder for DungeonZoneBuilder {
    /// Create new dungeon zone (needed?)
    fn build(depth: u32, _: &mut World) -> Zone {
        let mut zone = Zone::new(depth);

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
            for other_room in zone.rooms.iter() {
                if new_room.overlaps(other_room) {
                    room_not_overlaps = false
                }
            }
            if room_not_overlaps {
                Self::apply_room_to_map(&mut zone, &new_room);

                if !zone.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center_to_i32_tuple();
                    let (prev_x, prev_y) = zone.rooms[zone.rooms.len() - 1].center_to_i32_tuple();
                    if rand::gen_range(0, 2) == 1 {
                        Self::apply_horizontal_corridor(&mut zone, prev_x, new_x, prev_y);
                        Self::apply_vertical_corridor(&mut zone, prev_y, new_y, new_x);
                    } else {
                        Self::apply_vertical_corridor(&mut zone, prev_y, new_y, prev_x);
                        Self::apply_horizontal_corridor(&mut zone, prev_x, new_x, new_y);
                    }
                }

                zone.rooms.push(new_room);
            }
        }

        // Set player spawn point
        let first_room_center = zone.rooms[0].center();
        zone.player_spawn_point =
            Zone::get_index_from_xy_f32(first_room_center[0], first_room_center[1]);

        // Generate monster and items spawn points within each room
        for &room in zone.rooms.iter().skip(1) {
            let monster_number = Roll::dice(1, MAX_MONSTERS_IN_ZONE) - 1;
            let items_number = Roll::dice(1, MAX_ITEMS_IN_ZONE) - 1;

            for _ in 0..monster_number {
                for _ in 0..MAX_SPAWN_TENTATIVES {
                    let x = room.x + Roll::dice(1, room.w as i32 - 1) as f32;
                    let y = room.y + Roll::dice(1, room.h as i32 - 1) as f32;
                    let index = Zone::get_index_from_xy_f32(x, y);

                    // avoid duplicate spawnpoints
                    if zone.monster_spawn_points.insert(index) {
                        break;
                    }
                }
            }

            for _ in 0..items_number {
                for _ in 0..MAX_SPAWN_TENTATIVES {
                    let x = room.x + Roll::dice(1, room.w as i32 - 1) as f32;
                    let y = room.y + Roll::dice(1, room.h as i32 - 1) as f32;
                    let index = Zone::get_index_from_xy_f32(x, y);

                    // avoid duplicate spawnpoints
                    if zone.item_spawn_points.insert(index) {
                        break;
                    }
                }
            }
        }

        zone
    }
}

/// Other
#[allow(dead_code)]
impl DungeonZoneBuilder {
    fn apply_room_to_map(game_map: &mut Zone, room: &Rect) {
        for y in room.y as i32 + 1..(room.y + room.h) as i32 {
            for x in room.x as i32 + 1..(room.x + room.w) as i32 {
                game_map.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_corridor(game_map: &mut Zone, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = Zone::get_index_from_xy(&x, &y);
            if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
                game_map.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_corridor(game_map: &mut Zone, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = Zone::get_index_from_xy(&x, &y);
            if idx > 0 && idx < (MAP_WIDTH * MAP_HEIGHT) as usize {
                game_map.tiles[idx] = TileType::Floor;
            }
        }
    }
}
