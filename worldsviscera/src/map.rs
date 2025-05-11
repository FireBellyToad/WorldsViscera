use bracket_lib::{
    color::{BLACK, GRAY50, GREEN1, RGB},
    prelude::{to_cp437, BTerm}, random::RandomNumberGenerator
};

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 50;

// Copy Trait will handle a "=" operator as a copy instead of a moving (x = y will leave both variables valid)
/// Tiles Types
#[derive(PartialEq, Copy, Clone)] 
pub enum TileType {
    Wall,
    Floor,
}

/// Generates a new map
pub fn new_map() -> Vec<TileType> {
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

/// Draws a given map inside a bracket-lib::prelude::BTerm context
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

/// Gets a index from x and y, to be used for retriving a tile in a map  
pub fn get_index_from_xy(x: i32, y: i32) -> usize{
    (y as usize * (MAP_WIDTH as usize)) + x as usize
}

///
/// Check if tile in position x,y of map is passable
///
pub fn is_tile_passable(map: &[TileType], x: i32, y: i32) -> bool{
    match  map[get_index_from_xy(x, y)] {
        TileType::Wall => false,
        _ => true,
    }
}