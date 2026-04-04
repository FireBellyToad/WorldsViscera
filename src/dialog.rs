use std::{collections::HashMap, slice::Iter};

use hecs::{Entity, World};
use macroquad::{
    color::{BLACK, WHITE},
    input::{clear_input_queue, get_char_pressed},
    shapes::draw_rectangle,
    text::draw_text,
    texture::Texture2D,
};

use crate::{
    components::{
        actions::{WantsItem, WantsToDrink, WantsToEat},
        common::Named,
        items::Corpse,
    },
    constants::*,
    engine::state::{GameState, RunState},
    inventory::InventoryAction,
    systems::trade_system::{TradeDtt, TradeSystem},
    utils::assets::TextureName,
};

#[derive(PartialEq, Debug, Clone)]
pub enum DialogAction {
    Eat(Entity),
    Quaff(Entity),
    Trade(TradeDtt),
    StealPick(Entity),
    StealEat(Entity),
}

pub struct Dialog {}

impl Dialog {
    /// Handle dialog input
    pub fn handle_input(game_state: &mut GameState, action: DialogAction) {
        let ecs_world = &mut game_state.ecs_world;
        match get_char_pressed() {
            Some(letterkey) => match letterkey {
                'n' => {
                    // Exit dialog, clear queue to avoid to reopen on cancel
                    // caused by char input queue
                    clear_input_queue();

                    // show equivalent inventory action on exit
                    // Usually is used when we do an action while there is an appropriate object on the ground
                    // so that the player can choose if use that item or one that is already in inventory
                    game_state.run_state = match action {
                        DialogAction::Eat(_) => RunState::ShowInventory(InventoryAction::Eat),
                        DialogAction::Quaff(_) => RunState::ShowInventory(InventoryAction::Quaff),
                        _ => RunState::WaitingPlayerInput,
                    }
                }
                'y' => {
                    // Confirm action and execute its game logic
                    let player_entity = game_state
                        .current_player_entity
                        .expect("Player is not in hecs::World");
                    match action {
                        DialogAction::Eat(item) => {
                            let _ = ecs_world.insert_one(player_entity, WantsToEat { item });
                        }
                        DialogAction::Quaff(item) => {
                            let _ = ecs_world.insert_one(player_entity, WantsToDrink { item });
                        }
                        DialogAction::StealPick(item) => {
                            let _ = ecs_world.insert_one(
                                player_entity,
                                WantsItem {
                                    items: vec![item],
                                    was_bought: false,
                                },
                            );
                        }
                        DialogAction::StealEat(item) => {
                            game_state.run_state = RunState::ShowDialog(DialogAction::Eat(item));
                            return; // abort all other actions
                        }
                        DialogAction::Trade(trade_info) => {
                            TradeSystem::end_trade(ecs_world, trade_info);
                        }
                    }

                    //Avoid strange behaviors
                    clear_input_queue();
                    game_state.run_state = RunState::DoTick;
                }
                _ => game_state.run_state = RunState::ShowDialog(action),
            },
            None => game_state.run_state = RunState::ShowDialog(action),
        }
    }

    pub fn draw(_: &HashMap<TextureName, Texture2D>, ecs_world: &World, action: &DialogAction) {
        // Build the body text based on the dialog action
        // The body text is a vector of strings that will be displayed in the dialog box
        // each string will be displayed on a new line
        let body_text: Vec<String> = match action {
            DialogAction::Eat(item) => {
                let mut q = ecs_world
                    .query_one::<(&Named, Option<&Corpse>)>(*item)
                    .unwrap_or_else(|_| panic!("Item with entity {:?} is not named", item));
                let (named, corpse_opt) = q.get().expect("Item is not named!");

                // Hack to determine if the collected item is a corpse (for logging purposes)
                let corpse_text = if corpse_opt.is_some() {
                    format!("{}{}", named.name, " corpse")
                } else {
                    named.name.to_owned()
                };
                vec![
                    "There is a".to_owned(),
                    corpse_text,
                    "on the ground.".to_owned(),
                    "Eat it?".to_owned(),
                ]
            }
            DialogAction::Quaff(item) => {
                let named = ecs_world.get::<&Named>(*item).expect("Item is not named");
                vec!["There is a", named.name, "on the ground.", "Drink it?"]
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect()
            }
            DialogAction::StealPick(item) => {
                let named = ecs_world.get::<&Named>(*item).expect("Item is not named");
                vec![
                    "Picking this",
                    named.name,
                    "will anger its owner.",
                    "Steal it?",
                ]
                .into_iter()
                .map(|s| s.to_owned())
                .collect()
            }
            DialogAction::StealEat(item) => {
                let named = ecs_world.get::<&Named>(*item).expect("Item is not named");
                vec![
                    "Eating this",
                    named.name,
                    "will anger its owner.",
                    "Steal it?",
                ]
                .into_iter()
                .map(|s| s.to_owned())
                .collect()
            }
            DialogAction::Trade(trade_info) => {
                let (_, traded_item, shop_owner, items_to_be_received) = trade_info;
                let traded_named = ecs_world
                    .get::<&Named>(*traded_item)
                    .expect("traded_item is not named");
                let shop_owner_named = ecs_world
                    .get::<&Named>(*shop_owner)
                    .expect("shop_owner is not named");
                // Build items string with "and" and "carriage return"
                let mut offer_string =
                    Dialog::build_offer_string(items_to_be_received.iter(), ecs_world);

                let mut final_string_vec: Vec<&str> = vec![shop_owner_named.name, "offers you"];
                final_string_vec.append(&mut offer_string);
                final_string_vec.append(&mut vec!["for your", traded_named.name]);
                final_string_vec.into_iter().map(|s| s.to_owned()).collect()
            }
        };

        // ------- Background Rectangle -----------
        draw_rectangle(
            DIALOG_X as f32,
            DIALOG_Y as f32,
            DIALOG_SIZE as f32,
            DIALOG_SIZE as f32,
            WHITE,
        );
        draw_rectangle(
            (DIALOG_X + HUD_BORDER) as f32,
            (DIALOG_Y + HUD_BORDER) as f32,
            (DIALOG_SIZE - UI_BORDER) as f32,
            (DIALOG_SIZE - UI_BORDER) as f32,
            BLACK,
        );

        // ------- Text, Aligned to center -----------
        for (index, text) in body_text.iter().enumerate() {
            draw_text(
                text,
                (DIALOG_X + DIALOG_SIZE / 2 + HUD_BORDER) as f32
                    - (text.len() as f32 * LETTER_SIZE) / 2.0,
                (DIALOG_Y + DIALOG_TOP_SPAN + UI_BORDER) as f32
                    + (index as f32 * LETTER_SIZE * 2.5),
                FONT_SIZE,
                WHITE,
            );
        }

        // ------- Choices -----------
        draw_text(
            "(N)o",
            (DIALOG_X + DIALOG_SIZE - DIALOG_LEFT_SPAN + HUD_BORDER) as f32 - (5.0 * LETTER_SIZE),
            (DIALOG_Y + DIALOG_SIZE - DIALOG_TOP_SPAN + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            "(Y)es",
            (DIALOG_X + DIALOG_LEFT_SPAN + HUD_BORDER) as f32,
            (DIALOG_Y + DIALOG_SIZE - DIALOG_TOP_SPAN + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
    }

    /// Builds a string representation of the items to be received in a shop offer.
    fn build_offer_string(items: Iter<'_, Entity>, ecs_world: &World) -> Vec<&'static str> {
        let mut offer_string_arr: Vec<&str> = Vec::new();
        let items_length = items.len();
        for (index, item) in items.enumerate() {
            let mut offer_string = Vec::new();
            let named = ecs_world
                .get::<&Named>(*item)
                .expect("offered item is not named");
            offer_string.push("a ");
            offer_string.push(&named.name);
            if items_length >= 2 {
                if index < items_length - 2 {
                    offer_string.push(", ");
                } else if index == items_length - 2 {
                    offer_string.push(" and ");
                }
            }
            offer_string_arr.append(&mut offer_string);
        }
        offer_string_arr
    }
}
