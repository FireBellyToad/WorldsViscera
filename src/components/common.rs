use adam_fov_rs::{GridPoint, IVec2};
use macroquad::math::Rect;

use crate::{assets::TextureName, utils::point::Point};

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Renderable {
    pub texture_name: TextureName,
    pub texture_region: Rect,
}

pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub must_recalculate: bool
}