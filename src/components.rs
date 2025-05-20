use bracket_lib::{
    color::RGB,
    prelude::{FontCharType, Point},
};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub must_recalculate: bool
}

//Position
#[derive(Component)] // Macro for deriving all the needed data for Component (think of something like @Component in Java)
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub foreground: RGB,
    pub background: RGB,
}

#[derive(Component)]
struct LeftMover {}

#[derive(Component, Debug)]
pub struct Monster {}


#[derive(Component)]
pub struct Name {
    pub name: String
}

// This is a component that will be used to mark entities that can be targeted by attacks
#[derive(Component)]
pub struct Targetable {
    pub target_position: Position
}

#[derive(Component, Debug)]
pub struct BlocksTile {}