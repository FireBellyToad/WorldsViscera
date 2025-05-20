use macroquad::prelude::*;
use macroquad::texture::Texture2D;

use crate::constants::*;

/// Map Struct and implementations
pub struct Map {
    pub tileset: Texture2D,
}

enum TileType {
    Floor,
    Wall,
}

impl Map {
    /// Draws map
    pub fn draw_map(&self) {
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile_to_draw;
                
                // TODO REMOVE
                if x == 0 || y == 0 || x == MAP_WIDTH-1 || y == MAP_HEIGHT -1 {
                    tile_to_draw = TileType::Wall
                } else {
                    tile_to_draw = TileType::Floor
                }

                let tiles_index =  self.get_tile_index(tile_to_draw) * TILE_SIZE as f32;

                draw_texture_ex(
                    &self.tileset,
                    (UI_BORDER + (x * TILE_SIZE)) as f32,
                    (UI_BORDER + (y * TILE_SIZE)) as f32,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect {
                            x: tiles_index,
                            y: 0.0,
                            w: TILE_SIZE as f32,
                            h: TILE_SIZE as f32,
                        }),
                        ..Default::default()
                    },
                );
            }
        }
    }

    fn get_tile_index(&self, tile_type: TileType) -> f32 {
        match tile_type {
            TileType::Floor => 0.0,
            TileType::Wall => 1.0,
        }
    }
}
