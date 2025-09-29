use hecs::Entity;

pub struct Monster {}

pub struct Aquatic {}

pub struct Venomous {}

pub struct IsSmart{}

pub struct WantsToApproach {
    pub target_x: i32,
    pub target_y: i32,
}
