pub struct Utils {}

impl Utils {
    /// Pythagorean distance
    pub fn distance(x1: i32, x2: i32, y1: i32, y2: i32) -> f32 {
        ((x1.abs_diff(x2).pow(2) + y1.abs_diff(y2).pow(2)) as f32).sqrt()
    }
}
