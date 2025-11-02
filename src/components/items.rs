use hecs::Entity;

pub struct Item {
    pub item_tile: (i32, i32),
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

#[derive(PartialEq, Debug, Clone)]
#[allow(dead_code)]
pub enum BodyLocation {
    Arms,
    BothHands,
    LeftHand,
    RightHand,
    Torso,
    Head,
    Feet,
}

#[derive(PartialEq, Debug)]
pub enum InvokablesEnum {
    LightningWand,
    Crossbow, // TODO temporary
}

pub struct Invokable {
    pub invokable_type: InvokablesEnum,
}

pub struct Perishable {
    pub rot_counter: i32,
}

pub struct ToBeHarvested {}

pub struct Unsavoury {
    pub game_log: String,
}

pub struct Deadly {}

pub struct ProduceLight {
    pub radius: i32,
}

pub struct MustBeFueled {
    pub fuel_counter: i32,
}

pub struct Refiller {
    pub fuel_counter: i32,
}

pub struct TurnedOn {}
pub struct TurnedOff {}

pub struct Appliable {}
pub struct Applied {}

pub struct Equippable {
    pub body_location: BodyLocation,
}

pub struct Equipped {
    pub owner: Entity,
    pub body_location: BodyLocation,
}

pub struct MeleeWeapon {
    pub attack_dice: i32,
}

pub struct RangedWeapon {
    pub attack_dice: i32,
}

pub struct Armor {
    pub value: i32,
}
pub struct Bulky {}
pub struct Metallic {}
pub struct Eroded {
    pub value: u32,
}
