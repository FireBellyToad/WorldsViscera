use std::ops::Index;

use macroquad::{math::Rect, prelude::rand};
use pathfinding::num_traits::pow;

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::{
        ZoneFeatureBuilder,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

pub struct MushroomFieldBuilder {}

/// Builds a shop in the given zone.
impl ZoneFeatureBuilder for MushroomFieldBuilder {
    /**
    * Rendering minimo spazio libero 3 x 3

    ▫️▫️▫️▫️
    ▫️🟫🟫▫️
    ▫️🟫🟫▫️
    ▫️▫️▫️▫️

    Rendering massimo spazio libero 6 x 6

    ▫️▫️▫️▫️▫️▫️▫️
    ▫️🟫🟫🟫🟫🟫▫️
    ▫️🟫🟫🟫🟫🟫▫️
    ▫️🟫🟫🟫🟫🟫▫️
    ▫️🟫🟫🟫🟫🟫▫️
    ▫️🟫🟫🟫🟫🟫▫️
    ▫️▫️▫️▫️▫️▫️▫️


       1.a. Capire quanto si può espandere mantenendo una forma quadrata con un massimo di 6x6
    3. scegli un 🟫 casuale e impianta 1 🟢,🔴 o 🍄, 🍄‍🟫
    4. Fino a che tutti gli 🟫 sono stati esplorati, ripeti:
       3.a. Prendi uno 🟫 in ordine da top left
       3.b. Se libero, Con 1 su d4 impianta 1 🟢,🔴 o 🍄, 🍄‍🟫
    5. Piazza il coltivatore in uno degli ▫️ liberi
    6. riempi gli altri ▫️ con 🌑

    *
    */
    fn build(zone: &mut Zone) -> Vec<usize> {
        //1. Decidere se è un campo di 🟢,🔴 o 🍄, 🍄‍🟫

        // TODO only do this in free spaces
        //2. Create a fertilized space of  4x4 (max 6x6)
        let size: i32 = Roll::dice(1, 3) + 4;
        let x = Roll::dice(1, MAP_WIDTH - size - 1) - 1;
        let y = Roll::dice(1, MAP_HEIGHT - size - 1) - 1;
        let field = Rect::new_from_i32(x, y, size, size);
        let mut not_closed = true;
        let mut counter = 0;

        //2.1 Create a border of fences
        for y in field.y as i32 + 1..(field.y + field.h) as i32 {
            for x in field.x as i32 + 1..(field.x + field.w) as i32 {
                if not_closed && (counter == (size * 4) - 1 || Roll::dice(1, size * 4) == 1) {
                    not_closed = false
                } else {
                    zone.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::FieldFence;
                }
                counter += 1;
            }
        }
        //2.2 Create the actual field
        for y in field.y as i32 + 2..(field.y + field.h) as i32 - 1 {
            for x in field.x as i32 + 2..(field.x + field.w) as i32 - 1 {
                zone.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::MushroomField;
            }
        }

        Vec::new()
    }
}
