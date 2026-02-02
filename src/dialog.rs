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
        player::Player,
    },
    constants::*,
    engine::state::RunState,
    inventory::InventoryAction,
    systems::trade_system::{TradeDtt, TradeSystem},
    utils::assets::TextureName,
};

#[derive(PartialEq, Debug, Clone)]
pub enum DialogAction {
    Eat(Entity),
    Quaff(Entity),
    Trade(TradeDtt),
    Steal(Entity),
}

pub struct Dialog {}

impl Dialog {
    /// Handle dialog input
    pub fn handle_input(ecs_world: &mut World, action: DialogAction) -> RunState {
        match get_char_pressed() {
            Some(letterkey) => match letterkey {
                'n' => {
                    // Exit dialog, clear queue to avoid to reopen on cancel
                    // caused by char input queue
                    clear_input_queue();

                    // show equivalent inventory action on exit
                    // Usually is used when we do an action while there is an appropriate object on the ground
                    // so that the player can choose if use that item or one that is already in inventory
                    match action {
                        DialogAction::Eat(_) => RunState::ShowInventory(InventoryAction::Eat),
                        DialogAction::Quaff(_) => RunState::ShowInventory(InventoryAction::Quaff),
                        _ => RunState::WaitingPlayerInput,
                    }
                }
                'y' => {
                    // Confirm action and execute its game logic
                    let player_entity: Entity;
                    {
                        let mut player_query = ecs_world.query::<&Player>();
                        (player_entity, _) = player_query
                            .iter()
                            .last()
                            .expect("Player is not in hecs::World");
                    }

                    match action {
                        DialogAction::Eat(item) => {
                            let _ = ecs_world.insert_one(player_entity, WantsToEat { item });
                        }
                        DialogAction::Quaff(item) => {
                            let _ = ecs_world.insert_one(player_entity, WantsToDrink { item });
                        }
                        DialogAction::Steal(item) => {
                            let _ = ecs_world
                                .insert_one(player_entity, WantsItem { items: vec![item] });
                        }
                        DialogAction::Trade(trade_info) => {
                            TradeSystem::end_trade(ecs_world, trade_info);
                        }
                    }

                    //Avoid strange behaviors
                    clear_input_queue();
                    RunState::DoTick
                }
                _ => RunState::ShowDialog(action),
            },
            None => RunState::ShowDialog(action),
        }
    }

    pub fn draw(_: &HashMap<TextureName, Texture2D>, ecs_world: &World, action: &DialogAction) {
        // Build the body text based on the dialog action
        // The body text is a vector of strings that will be displayed in the dialog box
        // each string will be displayed on a new line
        let body_text = match action {
            DialogAction::Eat(item) => {
                let named = ecs_world.get::<&Named>(*item).expect("Item is not named");
                vec![
                    "There is a".to_string(),
                    named.name.clone(),
                    "on the ground.".to_string(),
                    "Eat it?".to_string(),
                ]
            }
            DialogAction::Quaff(item) => {
                let named = ecs_world.get::<&Named>(*item).expect("Item is not named");
                vec![
                    "There is a".to_string(),
                    named.name.clone(),
                    "on the ground.".to_string(),
                    "Drink it?".to_string(),
                ]
            }
            DialogAction::Steal(item) => {
                let named = ecs_world.get::<&Named>(*item).expect("Item is not named");
                vec![
                    "Picking this".to_string(),
                    named.name.clone(),
                    "will anger its owner.".to_string(),
                    "Steal it?".to_string(),
                ]
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
                let mut final_string_vec =
                    vec![shop_owner_named.name.clone(), "offers you".to_string()];
                final_string_vec.append(&mut offer_string);
                final_string_vec
                    .append(&mut vec!["for your".to_string(), traded_named.name.clone()]);
                final_string_vec
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
    fn build_offer_string(items: Iter<'_, Entity>, ecs_world: &World) -> Vec<String> {
        let mut offer_string_arr: Vec<String> = Vec::new();
        let items_length = items.len();
        for (index, item) in items.enumerate() {
            let mut offer_string = String::new();
            let named = ecs_world
                .get::<&Named>(*item)
                .expect("offered item is not named");
            offer_string.push_str("a ");
            offer_string.push_str(&named.name);
            if items_length >= 2 {
                if index < items_length - 2 {
                    offer_string.push_str(", ");
                } else if index == items_length - 2 {
                    offer_string.push_str(" and ");
                }
            }
            offer_string_arr.push(offer_string);
        }
        offer_string_arr
    }
}
