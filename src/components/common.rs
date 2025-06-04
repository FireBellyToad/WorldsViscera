use macroquad::math::Rect;

use crate::utils::assets::TextureName;

pub struct Position {
    pub x: i32,
    pub y: i32,
}
pub struct Renderable {
    pub texture_name: TextureName,
    pub texture_region: Rect
}
pub struct Viewshed {
    pub visible_tiles: Vec<(i32,i32)>,
    pub range: i32,
    pub must_recalculate: bool,
}
pub struct Named {
    pub name: String,
}
pub struct BlocksTile {}

pub struct GameLog {
    pub entries: Vec<String>,
}
