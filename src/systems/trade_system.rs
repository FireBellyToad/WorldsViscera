use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToTrade,
        common::{GameLog, Named},
        items::{Corpse, Item, Quaffable, ShopOwner, Tradable},
    },
    dialog::DialogAction,
    engine::state::RunState,
    maps::zone::Zone,
};

pub type TradeDtt = (Entity, Entity, Entity, Vec<Entity>);

pub struct TradeSystem {}

/// System for handling trading between entities
impl TradeSystem {
    pub fn run(ecs_world: &mut World, run_state: RunState) -> RunState {
        let mut traders: Vec<Entity> = Vec::new();
        let mut new_run_state = None;

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
                if let Some(traded_item) = wants_to_trade.item {
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
                                if ecs_world.satisfies::<&Corpse>(traded_item).unwrap_or(false) {
                                    item_selling_cost += 1;
                                }
                            }
                            Tradable::Quaffable => {
                                if ecs_world
                                    .satisfies::<&Quaffable>(traded_item)
                                    .unwrap_or(false)
                                {
                                    item_selling_cost += 1;
                                }
                            }
                        }
                    }

                    if item_selling_cost > 0 {
                        let mut zone_query = ecs_world.query::<&mut Zone>();
                        let (_, zone) = zone_query
                            .iter()
                            .last()
                            .expect("Zone is not in hecs::World");

                        let mut items_to_be_received: Vec<Entity> = Vec::new();
                        for &index in &shop_owner.shop_tiles {
                            for &content in &zone.tile_content[index] {
                                if ecs_world.satisfies::<&Item>(content).unwrap_or(false) {
                                    if items_to_be_received.len() < item_selling_cost {
                                        items_to_be_received.push(content);
                                    } else {
                                        break;
                                    }
                                }
                            }
                            // Avoid unnecessary iterations
                            if items_to_be_received.len() >= item_selling_cost {
                                break;
                            }
                        }

                        if items_to_be_received.is_empty() {
                            game_log
                                .entries
                                .push(format!("{} has no items to trade", shop_owner_name.name));
                        } else {
                            // Open trade dialog
                            new_run_state = Some(RunState::ShowDialog(DialogAction::Trade((
                                trader,
                                traded_item,
                                wants_to_trade.target,
                                items_to_be_received,
                            ))));
                        }
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

        // If a dialog must be opened, new_run_state is Some
        // If not, the game continues with the current run state
        if let Some(new_run_state) = new_run_state {
            new_run_state
        } else {
            run_state
        }
    }

    pub fn end_trade(_ecs_world: &mut World, _trade_info: TradeDtt) {
        // Place trade_info.1 into backpack
        todo!()
    }
}
