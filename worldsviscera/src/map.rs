use std::cmp::{max, min};

use bracket_lib::{
    color::{BLACK, GRAY50, GREEN1, RGB},
    prelude::{to_cp437, Algorithm2D, BTerm, BaseMap, Point, Rect},
    random::RandomNumberGenerator,
};
use specs::World;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 50;

// Copy Trait will handle a "=" operator as a copy instead of a moving (x = y will leave both variables valid)
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
}

impl Map {
    // Create new dungeon-like map and returns a vector of rooms and the map itself
    pub fn new_map_rooms_and_corridors() -> Map {
        //vec! is a factory macro for Vectors (ArrayList in Java)
        //here, vec! creates a 4000 (80 x 50) items long vector full of "Wall" for tiles
        //and a 4000 items long vector full of "false" for revealed_tiles
        let mut map = Map {
            tiles: vec![TileType::Wall; (MAP_WIDTH * MAP_HEIGHT) as usize],
            rooms: Vec::new(),
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            revealed_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
            visible_tiles: vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize],
        };

        //Empty vector of rooms
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            //Place random sized rooms in map
            let room_width = rng.range(MIN_SIZE, MAX_SIZE);
            let room_height = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - room_width - 1) - 1;
            let y = rng.roll_dice(1, map.height - room_height - 1) - 1;
            let new_room = Rect::with_size(x, y, room_width, room_height);
            let mut new_room_does_not_intersect = true;

            //Avoid intersecting rooms
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    new_room_does_not_intersect = false;
                }
            }

            if new_room_does_not_intersect {
                // Apply room
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    // get centers from the new room and the last inserted room
                    let new_center = new_room.center();
                    let prev_center = map.rooms[map.rooms.len() - 1].center();

                    // Draw corridors from the previous retrieved coordinates
                    // Do casual order
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_corridor(prev_center.x, new_center.x, prev_center.y);
                        map.apply_vertical_corridor(prev_center.y, new_center.y, new_center.x);
                    } else {
                        map.apply_vertical_corridor(prev_center.y, new_center.y, prev_center.x);
                        map.apply_horizontal_corridor(prev_center.x, new_center.x, new_center.y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }

    // Get vector index from x and y position on map
    pub fn get_index_from_xy(&self, x: i32, y: i32) -> usize {
        (y as usize * (self.width as usize)) + x as usize // Remember this is returned, no semicolon
    }

    // Inserts a room inside a given map
    // NdF: this fuction is ok, but we will use bracket-lib::Rect
    fn apply_room_to_map(&mut self, room: &Rect) {
        //Fill the room space with "Floor" tiles
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let index = self.get_index_from_xy(x, y);
                self.tiles[index] = TileType::Floor
            }
        }
    }

    fn apply_horizontal_corridor(&mut self, x1: i32, x2: i32, y: i32) {
        //From the lowest between x1 and x2 to the greatest between x1 and x2
        //fill a line of "Floor" tiles
        for x in min(x1, x2)..=max(x1, x2) {
            let index = self.get_index_from_xy(x, y);

            if index > 0 && index < (self.width * self.height) as usize {
                self.tiles[index as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_corridor(&mut self, y1: i32, y2: i32, x: i32) {
        //From the lowest between y1 and y2 to the greatest between y1 and y2
        //fill a line of "Floor" tiles
        for y in min(y1, y2)..=max(y1, y2) {
            let index = self.get_index_from_xy(x, y);

            if index > 0 && index < (self.width * self.height) as usize {
                self.tiles[index as usize] = TileType::Floor;
            }
        }
    }

    ///
    /// Check if tile in position x,y of map is passable
    ///
    pub fn is_tile_passable(&self, x: i32, y: i32) -> bool {
        let index = self.get_index_from_xy(x, y);
        match self.tiles[index] {
            TileType::Wall => false,
            _ => true,
        }
    }

    pub fn draw_map(&self, context: &mut BTerm) {
        for x in 0..self.width {
            for y in 0..self.height {
                let index = self.get_index_from_xy(x, y);
                // If visible to the player, render tile
                if self.revealed_tiles[index] {
                    let glyph_to_render;
                    let mut foreground_color;
                    match self.tiles[index] {
                        TileType::Floor => {
                            glyph_to_render = to_cp437('.');
                            foreground_color = RGB::named(GRAY50);
                        }
                        TileType::Wall => {
                            glyph_to_render = to_cp437('#');
                            foreground_color = RGB::named(GREEN1)
                        }
                    }

                    // If not visible, add fog of war (gray color)
                    if !self.visible_tiles[index] {
                        foreground_color = foreground_color.to_greyscale()
                    }
                    context.set(x, y, foreground_color, RGB::named(BLACK), glyph_to_render)
                }
            }
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> bracket_lib::prelude::Point {
        Point::new(self.width, self.height)
    }
}

// BaseMap is a bracketlib struct which implementations will support pathfinding
impl BaseMap for Map {
    // Return true if cannot see through
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}
