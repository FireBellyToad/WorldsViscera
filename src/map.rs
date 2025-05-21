use std::collections::HashMap;

use macroquad::{prelude::*, rand::rand};

use crate::{assets::TextureName, constants::*};

/// Map Struct and implementations
pub struct Map {
    pub tiles: Vec<TileType>,
}
#[derive(Clone, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
}

impl Map {
    /// Create new empty map
    pub fn new() -> Self {
        let mut map = Map {
            tiles: vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize],
        };

        // Create bondaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x == 0 || y == 0 || x == MAP_WIDTH - 1 || y == MAP_HEIGHT - 1 {
                    let index = map.get_index_from_xy(x, y);
                    map.tiles[index] = TileType::Wall
                }
            }
        }

        // TEST RANDOM WALLS
        for _i in 0..400 {
            let x = rand::gen_range(0, MAP_WIDTH - 1);
            let y = rand::gen_range(0, MAP_HEIGHT - 1);
            let idx =  map.get_index_from_xy(x, y);
            if idx !=  map.get_index_from_xy(MAP_HEIGHT/2, MAP_HEIGHT/2) {
                map.tiles[idx] = TileType::Wall;
            }
        }
        map
    }

    /// Draws map
    pub fn draw_map(&self, assets: &HashMap<TextureName, Texture2D>) {
        let texture_to_render = assets.get(&TextureName::Tiles).expect("Texture not found");

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile_to_draw = self.get_index_from_xy(x, y);
                let tile_index = self.get_tile_index(&self.tiles[tile_to_draw]) * TILE_SIZE as f32;

                // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                draw_texture_ex(
                    texture_to_render,
                    (UI_BORDER + (x * TILE_SIZE)) as f32,
                    (UI_BORDER + (y * TILE_SIZE)) as f32,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect {
                            x: tile_index,
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

    fn get_tile_index(&self, tile_type: &TileType) -> f32 {
        match tile_type {
            TileType::Floor => 0.0,
            TileType::Wall => 1.0,
        }
    }

    pub fn get_index_from_xy(&self, x: i32, y: i32) -> usize {
        ((y * MAP_WIDTH) + x) as usize
    }
}
