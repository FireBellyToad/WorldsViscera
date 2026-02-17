use std::collections::HashMap;

use crate::systems::{hunger_check::HungerStatus, thirst_check::ThirstStatus};

pub struct CanAutomaticallyHeal {
    pub tick_counter: i32,
}

pub struct Hunger {
    pub tick_counter: i32,
    pub current_status: HungerStatus,
}

pub struct Thirst {
    pub tick_counter: i32,
    pub current_status: ThirstStatus,
}

pub struct Diseased {
    pub tick_counters: HashMap<DiseaseType, (i32, bool)>,
}

pub struct Cured {
    pub diseases: Vec<DiseaseType>,
}

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum DiseaseType {
    FleshRot,
    Fever,
    Calcification,
}

pub struct Paralyzed {}

pub struct Blind {
    pub tick_counter: i32,
}
