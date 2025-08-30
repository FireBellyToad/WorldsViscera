use crate::components::items::BodyLocation;

pub struct Utils {}

impl Utils {
    /// Pythagorean distance
    pub fn distance(x1: i32, x2: i32, y1: i32, y2: i32) -> f32 {
        ((x1.abs_diff(x2).pow(2) + y1.abs_diff(y2).pow(2)) as f32).sqrt()
    }

    pub fn occupies_same_location(b1: &BodyLocation, b2: &BodyLocation) -> bool{
        if b1 == b2{
            return true;
        } 
        
        match b1 {
            BodyLocation::Hands => {
                return b2 == &BodyLocation::LeftHand ||  b2 == &BodyLocation::RightHand 
            },
            BodyLocation::LeftHand | BodyLocation::RightHand  =>{
                return b2 == &BodyLocation::Hands
            },
            _ => {}
        }

        false
    }
}
