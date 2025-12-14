use hecs::{Entity, World};

use crate::components::{
    actions::WantsToTrade,
    common::{GameLog, Named},
    items::{Corpse, Perishable, Quaffable, ShopOwner, Tradable},
};

pub struct TradeSystem {}

/// System for handling trading between entities
impl TradeSystem {
    pub fn run(ecs_world: &mut World) {
        let mut traders: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            let mut want_to_trade = ecs_world.query::<&WantsToTrade>();

            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (trader, wants_to_trade) in &mut want_to_trade {
                // Only get traders who has something to trade
                if let Some(item) = wants_to_trade.item {
                    let mut shop_owner_components = ecs_world
                        .query_one::<(&ShopOwner, &Named)>(wants_to_trade.target)
                        .expect("Must be a Named ShopOwner!");

                    let (shop_owner, shop_owner_name) = shop_owner_components
                        .get()
                        .expect("Must be a Named ShopOwner!");

                    // Calculate the selling cost of the item, based on the shop owner's wanted items
                    let mut item_selling_cost = 0;
                    for wanted in shop_owner.wanted_items.iter() {
                        match wanted {
                            Tradable::Corpse => {
                                if ecs_world.satisfies::<&Corpse>(item).unwrap_or(false) {
                                    item_selling_cost += 1;
                                }
                            }
                            Tradable::Quaffable => {
                                if ecs_world.satisfies::<&Quaffable>(item).unwrap_or(false) {
                                    item_selling_cost += 1;
                                }
                            }
                        }
                    }

                    if item_selling_cost > 0 {
                        todo!("Take item from shop with cost of {}", item_selling_cost);
                    } else {
                        game_log
                            .entries
                            .push(format!("{} is not interested", shop_owner_name.name));
                    }
                    traders.push(trader);
                }
            }
        }

        //Remove all WantsToTrade components
        for trader in traders {
            let _ = ecs_world.remove_one::<WantsToTrade>(trader);
        }
    }
}
