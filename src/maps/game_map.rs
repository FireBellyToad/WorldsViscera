use std::collections::{HashMap, HashSet};

use hecs::Entity;
use macroquad::{
    color::{Color, DARKGRAY, WHITE},
    math::Rect,
    shapes::draw_circle,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE, TILE_SIZE_F32, UI_BORDER},
    utils::assets::TextureName,
};

#[derive(Clone, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
}

/// GameMap Struct and implementations
pub struct GameMap {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked_tiles: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,
    pub bloodied_tiles: HashSet<usize>,
}

impl GameMap {
    pub fn get_adjacent_passable_tiles(
        &self,
        x_pos: i32,
        y_pos: i32,
        use_manhattan_distance: bool,
    ) -> Vec<(i32, i32)> {
        let mut adjacent_passable_tiles = Vec::new();

        for x in x_pos - 1..=x_pos + 1 {
            for y in y_pos - 1..=y_pos + 1 {
                //Manhattan Distance
                if !use_manhattan_distance || (x == x_pos || y == y_pos) {
                    if !self.blocked_tiles[Self::get_index_from_xy(x, y)] {
                        adjacent_passable_tiles.push((x, y));
                    }
                }
            }
        }

        adjacent_passable_tiles
    }

    pub fn populate_blocked(&mut self) {
        for (index, tile) in self.tiles.iter_mut().enumerate() {
            match tile {
                TileType::Floor => self.blocked_tiles[index] = false,
                _ => self.blocked_tiles[index] = true,
            }
        }
    }

    /// Draws map
    pub fn draw(&self, assets: &HashMap<TextureName, Texture2D>) {
        let texture_to_render = assets.get(&TextureName::Tiles).expect("Texture not found");

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile_to_draw = Self::get_index_from_xy(x, y);
                let tile_index =
                    Self::get_tile_sprite_sheet_index(&self.tiles[tile_to_draw]) * TILE_SIZE_F32;

                if self.revealed_tiles[tile_to_draw] {
                    let mut alpha = DARKGRAY;

                    if self.visible_tiles[tile_to_draw] {
                        alpha = WHITE;
                        if self.bloodied_tiles.contains(&tile_to_draw) {
                            Self::draw_blood_blots(x, y);
                        }
                    }

                    // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                    draw_texture_ex(
                        texture_to_render,
                        (UI_BORDER + (x * TILE_SIZE)) as f32,
                        (UI_BORDER + (y * TILE_SIZE)) as f32,
                        alpha,
                        DrawTextureParams {
                            source: Some(Rect {
                                x: tile_index,
                                y: 0.0,
                                w: TILE_SIZE_F32,
                                h: TILE_SIZE_F32,
                            }),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    /// Return true if cannot see through a tile
    pub fn is_tile_opaque(&self, x: i32, y: i32) -> bool {
        let index = Self::get_index_from_xy(x, y);
        self.tiles[index] == TileType::Wall
    }

    // Clears content index for this map
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Return a index inside the tile sheet
    fn get_tile_sprite_sheet_index(tile_type: &TileType) -> f32 {
        match tile_type {
            TileType::Floor => 0.0,
            TileType::Wall => 1.0,
        }
    }

    pub fn get_index_from_xy(x: i32, y: i32) -> usize {
        ((y * MAP_WIDTH) + x) as usize
    }

    fn draw_blood_blots(x: i32, y: i32) {
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 - 6.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 - 7.0,
            2.0,
            Color::from_rgba(255, 10, 10, 32),
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 + 4.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 - 4.0,
            2.0,
            Color::from_rgba(255, 10, 10, 32),
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 - 5.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 + 4.0,
            1.0,
            Color::from_rgba(255, 10, 10, 32),
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 + 3.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 + 5.0,
            2.0,
            Color::from_rgba(255, 10, 10, 32),
        );

        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32,
            4.0,
            Color::from_rgba(255, 10, 10, 32),
        );
    }
}
