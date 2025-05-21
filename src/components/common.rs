use macroquad::math::Rect;

use crate::assets::TextureName;

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Renderable {
    pub texture_name: TextureName,
    pub texture_region: Rect,
}
