
use specs::prelude::*;
use specs_derive::Component;
use bracket_lib::prelude::Point;

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles : Vec<Point>,
    pub range : i32
}