use crate::components::items::Equippable;
use std::cmp::max;

use hecs::{Entity, World};

use crate::{
    components::{
        common::{MyTurn, Named, SpeciesEnum, WaitingToAct},
        items::{
            BodyLocation, Equipped, Eroded, InBackback, Invokable, Metallic, MustBeFueled, TurnedOn,
        },
    },
    constants::MAX_ACTION_SPEED,
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
);

pub struct Utils {}

impl Utils {
    /// Pythagorean distance
    pub fn distance(x1: i32, x2: i32, y1: i32, y2: i32) -> f32 {
        ((x1.abs_diff(x2).pow(2) + y1.abs_diff(y2).pow(2)) as f32).sqrt()
    }

    pub fn occupies_same_location(b1: &BodyLocation, b2: &BodyLocation) -> bool {
        if b1 == b2 {
            return true;
        }

        match b1 {
            BodyLocation::Hands => {
                return b2 == &BodyLocation::LeftHand || b2 == &BodyLocation::RightHand;
            }
            BodyLocation::LeftHand | BodyLocation::RightHand => return b2 == &BodyLocation::Hands,
            _ => {}
        }

        false
    }

    pub fn wait_after_action(ecs_world: &mut World, waiter: Entity, speed: i32) {
        let count = max(1, MAX_ACTION_SPEED / speed);
        println!("Entity id {} must wait {} ticks", waiter.id(), count);
        // TODO account speed penalties
        let _ = ecs_world.exchange_one::<MyTurn, WaitingToAct>(
            waiter,
            WaitingToAct {
                tick_countdown: count,
            },
        );
    }

    pub fn what_hates(hater: &SpeciesEnum) -> Vec<SpeciesEnum> {
        match hater {
            SpeciesEnum::Human => vec![
                SpeciesEnum::Fish,
                SpeciesEnum::Gastropod,
                SpeciesEnum::Gremlin,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::Dvergar => vec![
                SpeciesEnum::Bug,
                SpeciesEnum::Gremlin,
                SpeciesEnum::DeepSpawn,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::Fish => vec![SpeciesEnum::Human, SpeciesEnum::Bug, SpeciesEnum::Gastropod],
            SpeciesEnum::Slime => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Dvergar,
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
                SpeciesEnum::Dvergar,
                SpeciesEnum::DeepSpawn,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::DeepSpawn => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Fish,
                SpeciesEnum::Dvergar,
                SpeciesEnum::Undead,
            ],
            SpeciesEnum::Undead => vec![
                SpeciesEnum::Human,
                SpeciesEnum::Dvergar,
                SpeciesEnum::DeepSpawn,
                SpeciesEnum::Gremlin,
            ],
        }
    }
}
