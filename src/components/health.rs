use crate::systems::hunger_check::HungerStatus;


pub struct CanAutomaticallyHeal {
    pub tick_counter: i32,
}

pub struct Hunger {
    pub tick_counter: i32,
    pub current_status: HungerStatus,
}
