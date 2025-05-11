// Common components module

use specs::prelude::*;
use specs_derive::Component;
use bracket_lib::{color::RGB, prelude::FontCharType};

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
