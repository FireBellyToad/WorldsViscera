use crate::{
    components::{
        common::{Position, Viewshed},
        items::{Ammo, Armor, Equippable, RangedWeapon, ShopOwner},
    },
    maps::zone::Zone,
};
use std::cmp::max;

use hecs::{Entity, World};

use crate::components::{
    common::{MyTurn, Named, SpeciesEnum, WaitingToAct},
    items::{
        BodyLocation, Equipped, Eroded, InBackback, Invokable, Metallic, MustBeFueled, TurnedOn,
    },
};

pub type ItemsInBackpack<'a> = (
    &'a Named,
    &'a InBackback,
    Option<&'a Invokable>,
    Option<&'a TurnedOn>,
    Option<&'a MustBeFueled>,
    Option<&'a Metallic>,
    Option<&'a Eroded>,
    Option<&'a Equippable>,
    Option<&'a Equipped>,
    Option<&'a RangedWeapon>,
);

pub type AmmunitionInBackpack<'a> = (&'a InBackback, &'a mut Ammo);

pub struct Utils {}

impl Utils {
    /// Pythagorean distance
    pub fn distance(x1: &i32, x2: &i32, y1: &i32, y2: &i32) -> f32 {
        ((x1.abs_diff(*x2).pow(2) + y1.abs_diff(*y2).pow(2)) as f32).sqrt()
    }
    /// Utility function to check if two Equippable occupy the same body location
    pub fn occupies_same_location(b1: &BodyLocation, b2: &BodyLocation) -> bool {
        if b1 == b2 {
            return true;
        }

        match b1 {
            BodyLocation::BothHands => {
                return b2 == &BodyLocation::LeftHand || b2 == &BodyLocation::RightHand;
            }
            BodyLocation::LeftHand | BodyLocation::RightHand => {
                return b2 == &BodyLocation::BothHands;
            }
            _ => {}
        }

        false
    }

    /// Utility function to make an entity wait after an action
    pub fn wait_after_action(ecs_world: &mut World, waiter: Entity, speed: i32) {
        let count = max(1, speed);
        println!("Entity {:?} must wait {} ticks", waiter, count);
        // TODO account speed penalties
        let _ = ecs_world.exchange_one::<MyTurn, WaitingToAct>(
            waiter,
            WaitingToAct {
                tick_countdown: count,
            },
        );
    }

    // Gets armor value
    pub fn get_armor_value(
        base_armor: i32,
        target_id: u32,
        equipped_armors: &mut hecs::QueryBorrow<'_, (&Armor, &Equipped, Option<&Eroded>)>,
    ) -> i32 {
        let mut total_armor = 0;
        // Use weapon dice when equipped
        for (_, (attacker_armor, equipped_to, eroded)) in equipped_armors.iter() {
            if equipped_to.owner.id() == target_id {
                if let Some(erosion) = eroded {
                    total_armor += max(0, attacker_armor.value - erosion.value as i32);
                } else {
                    total_armor += attacker_armor.value;
                }
            }
        }
        println!("base_armor: {}, total_armor {}", base_armor, total_armor);
        max(base_armor, total_armor)
    }

    /// Hate table by species
    //TODO change to static const!
    pub fn what_hates(hater: &SpeciesEnum) -> Vec<SpeciesEnum> {
        match hater {
            SpeciesEnum::Human => vec![
                SpeciesEnum::Fish,
                SpeciesEnum::Gastropod,
                SpeciesEnum::Gremlin,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::Undergrounder => vec![
                SpeciesEnum::Bug,
                SpeciesEnum::Gremlin,
                SpeciesEnum::DeepSpawn,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::Fish => vec![SpeciesEnum::Human, SpeciesEnum::Bug, SpeciesEnum::Gastropod],
            SpeciesEnum::Slime => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Undergrounder,
                SpeciesEnum::DeepSpawn,
            ],
            SpeciesEnum::Gastropod => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Myconid,
                SpeciesEnum::DeepSpawn,
            ],
            SpeciesEnum::Myconid => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Slime,
                SpeciesEnum::Bug,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::Bug => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Bug,
                SpeciesEnum::Fish,
                SpeciesEnum::Gastropod,
            ],
            SpeciesEnum::Gremlin => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Undergrounder,
                SpeciesEnum::DeepSpawn,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::DeepSpawn => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Fish,
                SpeciesEnum::Undergrounder,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::Undead => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Undergrounder,
                SpeciesEnum::DeepSpawn,
                SpeciesEnum::Gremlin,
            ],
        }
    }

    /// Calculate the farthest visible point from the target position within the given viewshed.
    pub(crate) fn calculate_farthest_visible_point(
        target_x: &i32,
        target_y: &i32,
        viewshed: &Viewshed,
    ) -> (i32, i32) {
        let (mut new_x, mut new_y) = (-1, -1);
        let mut distance = 0.0;

        for &index in viewshed.visible_tiles.iter() {
            let (x, y) = Zone::get_xy_from_index(index);
            let new_distance = Utils::distance(target_x, &x, target_y, &y);
            if new_distance > distance {
                distance = new_distance;
                new_x = x;
                new_y = y;
            }
        }

        (new_x, new_y)
    }

    /// Check if a shop owner has ownership of an item and returns its Entity when found.
    pub fn get_item_owner(ecs_world: &World, item: Entity) -> Option<Entity> {
        let mut query_result = ecs_world
            .query_one::<&Position>(item)
            .expect("Item must have Position");
        let position = query_result.get().expect("Item must have Position");

        Utils::get_item_owner_by_position(ecs_world, &position.x, &position.y)
    }

    /// Check if a shop owner has ownership of an item and returns its Entity when found.
    pub fn get_item_owner_by_position(
        ecs_world: &World,
        item_x: &i32,
        item_y: &i32,
    ) -> Option<Entity> {
        let mut shop_owner = ecs_world.query::<(&Named, &ShopOwner)>();

        let mut found = None;
        for (owner, (_, shop_owner)) in &mut shop_owner {
            if found.is_none()
                && shop_owner
                    .shop_tiles
                    .iter()
                    .any(|&index| Zone::get_index_from_xy(&item_x, &item_y) == index)
            {
                found = Some(owner);

                break;
            }
        }

        found
    }
}
