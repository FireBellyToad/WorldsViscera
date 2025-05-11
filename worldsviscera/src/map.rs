use std::cmp::{max, min};

use crate::rect::Rect;
use bracket_lib::{
    color::{BLACK, GRAY50, GREEN1, RGB},
    prelude::{BTerm, to_cp437},
    random::RandomNumberGenerator,
};

use super::get_index_from_xy;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 50;

// Copy Trait will handle a "=" operator as a copy instead of a moving (x = y will leave both variables valid)
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn draw_map(map: &[TileType], context: &mut BTerm) {
    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            match map[get_index_from_xy(x, y)] {
                TileType::Floor => {
                    context.set(x, y, RGB::named(GRAY50), RGB::named(BLACK), to_cp437('.'))
                }
                TileType::Wall => {
                    context.set(x, y, RGB::named(GREEN1), RGB::named(BLACK), to_cp437('#'))
                }
            }
        }
    }
}

// Create new dungeon-like map and returns a vector of rooms and the map itself 
pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    //vec! is a factory macro for Vectors (ArrayList in Java)
    //here, vec! creates a 4000 (80 x 50) items long vector full of "Wall" tiles
    let mut map = vec![TileType::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize];

    //Empty vector of rooms
    let mut rooms: Vec<Rect> = Vec::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for _ in 0..MAX_ROOMS {
        //Place random sized rooms in map
        let room_width = rng.range(MIN_SIZE, MAX_SIZE);
        let room_height = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, MAP_WIDTH - room_width - 1) - 1;
        let y = rng.roll_dice(1, MAP_HEIGHT - room_height - 1) - 1;
        let new_room = Rect::new(x, y, room_width, room_height);
        let mut new_room_does_not_intersect = true;

        //Avoid intersecting rooms
        for other_room in rooms.iter() {
            if new_room.intersects_with(other_room) {
                new_room_does_not_intersect = false;
            }
        }

        if new_room_does_not_intersect {
            // Apply room
            apply_room_to_map(&new_room, &mut map);

            if !rooms.is_empty() {
                // get centers from the new room and the last inserted room
                let (new_x, new_y) = new_room.get_center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].get_center();

                // Draw corridors from the previous retrieved coordinates
                // Do casual order
                if rng.range(0, 2) == 1 {
                    apply_horizontal_corridor(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_corridor(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_corridor(&mut map, prev_y, new_y, prev_x);
                    apply_horizontal_corridor(&mut map, prev_x, new_x, new_y);
                }
            }

            rooms.push(new_room);
        }
    }

    (rooms, map)
}

// Inserts a room inside a given map
// NdF: this fuction is ok, but we will use bracket-lib::Rect
fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    //Fill the room space with "Floor" tiles
    for y in room.y1 + 1..room.y2 {
        for x in room.x1 + 1..room.x2 {
            map[get_index_from_xy(x, y)] = TileType::Floor
        }
    }
}

fn apply_horizontal_corridor(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    //From the lowest between x1 and x2 to the greatest between x1 and x2
    //fill a line of "Floor" tiles
    for x in min(x1, x2)..=max(x1, x2) {
        let index = get_index_from_xy(x, y);

        if index > 0 && index < (MAP_WIDTH * MAP_HEIGHT) as usize {
            map[index as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_corridor(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    //From the lowest between y1 and y2 to the greatest between y1 and y2
    //fill a line of "Floor" tiles
    for y in min(y1, y2)..=max(y1, y2) {
        let index = get_index_from_xy(x, y);

        if index > 0 && index < (MAP_WIDTH * MAP_HEIGHT) as usize {
            map[index as usize] = TileType::Floor;
        }
    }
}

/// test map generator, do not use in real situation
pub fn new_map_test() -> Vec<TileType> {
    //vec! is a factory macro for Vectors (ArrayList in Java)
    //here, vec! creates a 4000 (80 x 50) items long vector full of "Floor" tiles
    let mut map = vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize];

    //Make boundary walls
    for x in 0..MAP_WIDTH {
        map[get_index_from_xy(x, 0)] = TileType::Wall;
        map[get_index_from_xy(x, MAP_HEIGHT - 1)] = TileType::Wall;
    }

    for y in 0..MAP_HEIGHT {
        map[get_index_from_xy(0, y)] = TileType::Wall;
        map[get_index_from_xy(MAP_WIDTH - 1, y)] = TileType::Wall;
    }

    //Get thread local RNG
    let mut rng = RandomNumberGenerator::new();

    //Random walls around the map
    for _i in 0..400 {
        let wall_x = rng.roll_dice(1, MAP_WIDTH - 1);
        let wall_y = rng.roll_dice(1, MAP_HEIGHT - 1);
        let index = get_index_from_xy(wall_x, wall_y);

        //Place wall in map unless x and y are on player position
        if index != get_index_from_xy(MAP_WIDTH / 2, MAP_HEIGHT / 2) {
            map[index] = TileType::Wall
        }
    }

    //Return new map
    map
}
