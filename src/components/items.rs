use hecs::Entity;

pub struct Item {
    pub item_tile_index: i32,
}

pub struct Edible {
    pub nutrition_dice_number: i32,
    pub nutrition_dice_size: i32,
}

pub struct Quaffable {
    pub thirst_dice_number: i32,
    pub thirst_dice_size: i32,
}

pub struct InBackback {
    pub owner: Entity,
    pub assigned_char: char,
}

pub struct WantsItem {
    pub item: Entity,
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

pub struct Invokable {}

pub struct Perishable {
    pub rot_counter: i32,
}

pub struct Rotten {}
