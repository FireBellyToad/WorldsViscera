use adam_fov_rs::{GridPoint, IVec2};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl GridPoint for Point {
    fn xy(&self) -> adam_fov_rs::IVec2 {
        IVec2::new(self.x, self.y)
    }
}
