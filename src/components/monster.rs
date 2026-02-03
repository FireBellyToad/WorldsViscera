use crate::{components::health::DiseaseType, maps::zone::DecalType};

pub struct Monster {}

pub struct Aquatic {}

pub struct Venomous {}

pub struct DiseaseBearer {
    pub disease_type: DiseaseType,
}

pub struct Smart {}

pub struct Small {}

pub struct IsPrey {}

pub struct LeaveTrail {
    pub of: DecalType,
    pub trail_lifetime: u32,
}

pub struct TrailPlaceholder {
    pub trail_counter: u32,
    pub trail_pos_idx: usize,
}

pub struct WantsToApproach {
    pub target_x: i32,
    pub target_y: i32,
    pub counter: u32,
}
