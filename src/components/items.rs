use hecs::Entity;

pub struct Item {}

pub struct Edible {
    pub nutrition_amount: i32,
}

pub struct InBackback {
    pub owner: Entity
}

pub struct WantsItem {
    pub collected_by: Entity,
    pub item: Entity
}