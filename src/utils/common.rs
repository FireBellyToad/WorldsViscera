use std::cmp::max;

use hecs::{Entity, World};

use crate::{components::{common::{MyTurn, Named, WaitingToAct}, items::{BodyLocation, Eroded, InBackback, Metallic, MustBeFueled, TurnedOn}}, constants::MAX_ACTION_SPEED};

pub type ItemsInBackpack<'a> = (
    &'a Named,
    &'a InBackback,
    Option<&'a TurnedOn>,
    Option<&'a MustBeFueled>,
    Option<&'a Metallic>,
    Option<&'a Eroded>,
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

    pub fn wait_after_action(ecs_world: &mut World, waiter: Entity, speed:i32) {
        
        let count = max(1, MAX_ACTION_SPEED - speed);
        println!("Entity id {} must wait {} ticks",waiter.id(),count);
        // TODO account speed penalties
        let _ = ecs_world.exchange_one::<MyTurn, WaitingToAct>(
            waiter,
            WaitingToAct {
                tick_countdown: count,
            },
        );
        
    }
}
