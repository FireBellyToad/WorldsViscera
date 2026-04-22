use macroquad::math::Rect;

use crate::{
    constants::{CRYSTAL_GROWTH_COUNTER_START, MAP_HEIGHT, MAP_WIDTH},
    maps::{Zone, ZoneFeatureBuilder, zone::TileType},
    spawning::spawner::Spawn,
    utils::roll::Roll,
};

pub struct CrystalPatchBuilder {}

impl ZoneFeatureBuilder for CrystalPatchBuilder {
    fn build(zone: &mut Zone, ecs_world: &mut hecs::World) -> Vec<usize> {
        // 1. search for free spaces to build the field in
        let mut tiles: Vec<usize> = Vec::new();
        //2 Create a potentialspace
        let size = 3;
        let x = Roll::dice(1, MAP_WIDTH - size) - 1;
        let y = Roll::dice(1, MAP_HEIGHT - size) - 1;
        let field_rect = Rect::new_from_i32(x, y, size, size);

        for y in field_rect.y as i32..(field_rect.y + field_rect.h) as i32 {
            for x in field_rect.x as i32..(field_rect.x + field_rect.w) as i32 {
                tiles.push(Zone::get_index_from_xy(&x, &y));
            }
        }
        for (vec_pos, &index) in tiles.iter().enumerate() {
            if vec_pos == tiles.iter().len() / 2 {
                zone.tiles[index] = TileType::LittleCrystal;
            } else {
                zone.tiles[index] = TileType::MiniCrystal;
            }
            zone.special_tile_counter[index] = CRYSTAL_GROWTH_COUNTER_START;
            let (x, y) = Zone::get_xy_from_index(index);
            Spawn::tile_entity(ecs_world, x, y, &zone.tiles[index]);
        }

        tiles
    }
}
