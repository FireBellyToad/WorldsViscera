use hecs::Entity;

use crate::constants::{BOLT_PARTICLE_TYPE, STONE_PARTICLE_TYPE};

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
}

pub struct Invokable {
    pub invokable_type: InvokablesEnum,
}

pub struct Perishable {
    pub rot_counter: i32,
}

pub struct ToBeHarvested {}

pub struct Poisonous {}

pub struct Rotten {}

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

pub struct MeleeWeapon {}

pub struct RangedWeapon {
    pub ammo_type: AmmoType,
    pub ammo_count_total: u32, // this is used in readonly. Real ammo count update is done by the Ammo component
}

pub struct Ammo {
    pub ammo_type: AmmoType,
    pub ammo_count: u32,
}

#[derive(PartialEq, Debug)]
pub enum AmmoType {
    Crossbow,
    Slingshot,
}

impl AmmoType {
    pub fn particle(&self) -> u32 {
        match *self {
            AmmoType::Crossbow => BOLT_PARTICLE_TYPE,
            AmmoType::Slingshot => STONE_PARTICLE_TYPE,
        }
    }
}

pub struct Armor {
    pub value: i32,
}

pub struct Bulky {}

pub struct Metallic {}

pub struct Eroded {
    pub value: u32,
}

pub struct DiggingTool {}

pub struct ShopOwner {
    pub shop_tiles: Vec<usize>,
    pub wanted_items: Vec<Tradable>,
}

pub struct Corpse {}

pub enum Tradable {
    Corpse,
    Rotten,
}
