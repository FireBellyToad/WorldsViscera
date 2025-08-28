use hecs::Entity;

use crate::components::items::BodyLocation;

pub struct WantsItem {
    pub item: Entity,
}

pub struct WantsToEquip {
    pub item: Entity,
    pub body_location: BodyLocation,
}

pub struct WantsToEat {
    pub item: Entity,
}

pub struct WantsToDrop {
    pub item: Entity,
}

pub struct WantsToDrink {
    pub item: Entity,
}

pub struct WantsToInvoke {
    pub item: Entity,
}

pub struct WantsToFuel {
    pub item: Entity,
    pub with: Option<Entity>,
}

pub struct WantsToSmell {
    pub target: (i32, i32),
}
