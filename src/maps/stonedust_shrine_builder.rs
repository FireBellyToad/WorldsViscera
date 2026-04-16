use hecs::{Entity, World};
use macroquad::math::Rect;

use crate::{
    components::{
        common::Immobile,
        items::{ShopOwner, Tradable},
    },
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::{
        ZoneFeatureBuilder,
        zone::{TileType, Zone},
    },
    spawning::spawner::Spawn,
    utils::roll::Roll,
};

pub struct StonedustShrineBuilder {}

/// Builds a shop in the given zone.
impl ZoneFeatureBuilder for StonedustShrineBuilder {
    fn build(zone: &mut Zone, ecs_world: &mut World) -> Vec<usize> {
        // 1. search for free spaces to build the field in
        let mut tiles: Vec<usize> = Vec::new();
        //2 Create a potential fertilized space from 3x3 to 6x6
        let size = 4;
        let mut x = Roll::dice(1, MAP_WIDTH - size) - 1;
        let mut y = Roll::dice(1, MAP_HEIGHT - size) - 1;
        let mut field_rect = Rect::new_from_i32(x, y, size, size);

        for _ in 0..150 {
            // 2.1 check if the space is free
            let mut is_free = true;
            for y in field_rect.y as i32..(field_rect.y + field_rect.h) as i32 {
                for x in field_rect.x as i32..(field_rect.x + field_rect.w) as i32 {
                    if zone.tiles[Zone::get_index_from_xy(&x, &y)] != TileType::Floor {
                        is_free = false;
                        tiles.clear();
                        break;
                    } else {
                        tiles.push(Zone::get_index_from_xy(&x, &y));
                    }
                }

                if !is_free {
                    break;
                }
            }

            if is_free {
                break;
            }

            // 2.2  next iteration
            x = Roll::dice(1, MAP_WIDTH - size);
            y = Roll::dice(1, MAP_HEIGHT - size);
            field_rect = Rect::new_from_i32(x, y, size, size);
        }

        // If still there is no free space, just end
        if !tiles.is_empty() {
            //3 Create a border of fences and fill the rest with the actual field
            let mut has_opening = false;
            let mut counter = 0;
            let mut owner_opt: Option<Entity> = None;
            for &index in &tiles {
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
                        zone.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::BrickWall;
                    } else if !has_opening && (counter >= (size) || Roll::dice(1, 4) == 1) {
                        // Guarantee an open space in the fence
                        has_opening = true;
                        owner_opt = Some(Spawn::stonedust_abbot(ecs_world, x, y));
                    } else {
                        zone.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::BrickWall;
                    }
                } else {
                    let index = Zone::get_index_from_xy(&x, &y);
                    zone.tiles[index] = TileType::StoneFloor;
                    // Put curing paste!
                    if Roll::dice(1, 2) == 1 {
                        Spawn::curing_paste(ecs_world, x, y);
                    }
                }
            }

            // Insert Mushroom field into ECS to be used as shop
            if let Some(owner) = owner_opt {
                let _ = ecs_world.insert(
                    owner,
                    (ShopOwner {
                        shop_tiles: tiles.clone(),
                        wanted_items: vec![Tradable::RawGold],
                    },),
                );
            } else {
                panic!("Cannot create Stonedust Shrine without owner!");
            }
        }

        tiles
    }
}
