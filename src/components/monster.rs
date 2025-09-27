use hecs::Entity;

pub struct Monster {}

pub struct Aquatic {}

pub struct Venomous {}

pub struct WantsToApproach {
    pub target: Entity,
    pub move_to_x: i32,
    pub move_to_y: i32,
}
