use hecs::Entity;

pub struct Item {}

pub struct Edible {
    pub nutrition_amount: i32,
}

pub struct InBackback {
    pub owner: Entity
}

pub struct WantsItem {
    pub item: Entity
}
pub struct WantsToEat {
    pub item: Entity
}
pub struct WantsToDrop {
    pub item: Entity
}