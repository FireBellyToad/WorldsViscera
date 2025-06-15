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
