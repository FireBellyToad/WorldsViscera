use std::collections::LinkedList;

use hecs::Entity;

use crate::{components::health::DiseaseType, maps::zone::DecalType};

pub struct Monster {}

pub struct Aquatic {}

pub struct Venomous {}

pub struct DiseaseBearer {
    pub disease_type: DiseaseType,
}

pub struct Smart {}

pub struct Small {}

pub struct Prey {}

pub struct LeaveTrail {
    pub of: DecalType,
    pub trail_lifetime: u32,
}

pub struct TrailCounter {
    pub trail_counter: u32,
}

pub struct WantsToApproach {
    pub target_x: i32,
    pub target_y: i32,
    pub counter: u32,
}

pub struct SnakeHead {
    pub body: LinkedList<Entity>,
    pub is_single_creature: bool,
}

pub struct SnakeBody {
    pub head: Entity,
}
