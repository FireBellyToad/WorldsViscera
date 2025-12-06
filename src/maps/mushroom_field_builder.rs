use macroquad::math::Rect;

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

        // 2. search for free spaces to build the field in
        let mut field_tiles: Vec<usize> = Vec::new();
        //2.1 Create a potential fertilized space from 4x4 to 7x7
        let mut size = Roll::dice(1, 3) + 3;
        let mut x = Roll::dice(1, MAP_WIDTH - size - 1);
        let mut y = Roll::dice(1, MAP_HEIGHT - size - 1);
        let mut field_rect = Rect::new_from_i32(x, y, size, size);

        for _ in 0..150 {
            // 2.2 check if the space is free
            let mut is_free = true;
            for y in field_rect.y as i32..(field_rect.y + field_rect.h) as i32 {
                for x in field_rect.x as i32..(field_rect.x + field_rect.w) as i32 {
                    if zone.tiles[Zone::get_index_from_xy(&x, &y)] != TileType::Floor {
                        is_free = false;
                        field_tiles.clear();
                        break;
                    } else {
                        field_tiles.push(Zone::get_index_from_xy(&x, &y));
                    }
                }

                if !is_free {
                    break;
                }
            }

            if is_free {
                break;
            }

            // 2.3 get new size and rect for next iteration
            size = Roll::dice(1, 3) + 3;
            x = Roll::dice(1, MAP_WIDTH - size - 1);
            y = Roll::dice(1, MAP_HEIGHT - size - 1);
            field_rect = Rect::new_from_i32(x, y, size, size);
        }

        //3 Create a border of fences and fill the rest with the actual field
        let mut has_opening = false;
        let mut counter = 0;
        for index in field_tiles {
            let (x, y) = Zone::get_xy_from_index(index);

            // Check if the tile is on the border, must be a fence
            if x == field_rect.x as i32
                || y == field_rect.y as i32
                || x == field_rect.x as i32 + size - 1
                || y == field_rect.y as i32 + size - 1
            {
                counter += 1;
                // In corners guarantee that the border has a fence
                if has_opening
                    || (((x - field_rect.x as i32) % (size - 1) == 0)
                        && ((y - field_rect.y as i32) % (size - 1) == 0))
                {
                    zone.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::FieldFence;
                } else if !has_opening && (counter >= (size * 3) || Roll::dice(1, 4) == 1) {
                    // Guarantee an open space in the fence
                    has_opening = true;
                } else {
                    zone.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::FieldFence;
                }
            } else {
                zone.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::MushroomField;
            }
        }

        Vec::new()
    }
}
