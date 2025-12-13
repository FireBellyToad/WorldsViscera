use hecs::{Entity, World};

use crate::components::actions::WantsToTrade;

pub struct TradeSystem {}

/// System for handling trading between entities
impl TradeSystem {
    pub fn run(ecs_world: &mut World) {
        let mut traders: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut want_to_trade = ecs_world.query::<&WantsToTrade>();

            for (trader, wants_to_trade) in &mut want_to_trade {
                // Only get traders who has something to trade
                if let Some(item) = wants_to_trade.item {
                    // TODO Implement trading logic here
                    println!(
                        "Entity {:?} will trade {:?} with {:?}",
                        trader, item, wants_to_trade.target
                    );
                    traders.push(trader);
                }
            }
        }

        //Remove all WantsToTrade components
        for trader in traders {
            ecs_world.remove_one::<WantsToTrade>(trader);
        }
    }
}
