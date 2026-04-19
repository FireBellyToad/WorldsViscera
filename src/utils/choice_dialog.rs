use std::{collections::HashMap, slice::Iter};

use hecs::{Entity, World};
use macroquad::{
    color::{BLACK, WHITE},
    input::{clear_input_queue, get_char_pressed},
    shapes::draw_rectangle,
    text::{TextAlignment, TextParams, draw_multiline_text_ex, draw_text},
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
    systems::trade_system::TradeSystem,
    utils::{
        assets::TextureName,
        common::Utils,
        dialog::{Dialog, DialogAction},
    },
};

pub struct ChoiceDialog {}

impl Dialog<DialogAction> for ChoiceDialog {
    /// Handle dialog input
    fn handle_input(game_state: &mut GameState, action: DialogAction) {
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
                'y' | 'o' => {
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
                        _ => {}
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

    fn draw(_: &HashMap<TextureName, Texture2D>, ecs_world: &World, action: &DialogAction) {
        // Build the body text based on the dialog action
        // The body text is a vector of strings that will be displayed in the dialog box
        // each string will be displayed on a new line
        // These string must be owned (String) rather than borrowed (&str),
        // so we use into_iter().map(|s| s.to_owned()).collect()
        let body_text: String = match action {
            DialogAction::Eat(item) => {
                let mut q = ecs_world
                    .query_one::<(&Named, Option<&Corpse>)>(*item)
                    .unwrap_or_else(|_| panic!("Item with entity {:?} is not named", item));
                let (named, corpse_opt) = q.get().expect("Item is not named!");

                // Hack to determine if the collected item is a corpse (for logging purposes)
                format!(
                    "There is a\n{}{}\non the ground.\nEat it?",
                    named.name,
                    Utils::get_corpse_string(corpse_opt.is_some())
                )
            }
            DialogAction::Quaff(item) => {
                let named = ecs_world.get::<&Named>(*item).expect("Item is not named");
                format!("There is a\n{}\non the ground.\nDrink it?", named.name)
            }
            DialogAction::StealPick(item) => {
                let mut q = ecs_world
                    .query_one::<(&Named, Option<&Corpse>)>(*item)
                    .unwrap_or_else(|_| panic!("Item with entity {:?} is not named", item));
                let (named, corpse_opt) = q.get().expect("Item is not named!");
                // Hack to determine if the collected item is a corpse (for logging purposes)
                format!(
                    "Picking this\n{}{}\nwill anger its owner.\nSteal it?",
                    named.name,
                    Utils::get_corpse_string(corpse_opt.is_some())
                )
            }
            DialogAction::StealEat(item) => {
                let mut q = ecs_world
                    .query_one::<(&Named, Option<&Corpse>)>(*item)
                    .unwrap_or_else(|_| panic!("Item with entity {:?} is not named", item));
                let (named, corpse_opt) = q.get().expect("Item is not named!");
                // Hack to determine if the collected item is a corpse (for logging purposes)
                format!(
                    "Eating this\n{}{}\nwill anger its owner.\nSteal it?",
                    named.name,
                    Utils::get_corpse_string(corpse_opt.is_some())
                )
            }
            DialogAction::Trade(trade_info) => {
                let (_, traded_item, shop_owner, items_to_be_received) = trade_info;

                let mut q = ecs_world
                    .query_one::<(&Named, Option<&Corpse>)>(*traded_item)
                    .unwrap_or_else(|_| panic!("Item with entity {:?} is not named", traded_item));
                let (traded_named, corpse_opt) = q.get().expect("Item is not named!");
                let shop_owner_named = ecs_world
                    .get::<&Named>(*shop_owner)
                    .expect("shop_owner is not named");
                // Build items string with "and" and "carriage return"
                // Hack to determine if the collected item is a corpse (for logging purposes)
                format!(
                    "{}\noffers you\n{}\nfor your\n{}{}.\nAccept the offer?",
                    shop_owner_named.name,
                    ChoiceDialog::build_offer_string(items_to_be_received.iter(), ecs_world),
                    traded_named.name,
                    Utils::get_corpse_string(corpse_opt.is_some()),
                )
            }
            _ => panic!("Cannot handle DialogAction {:?} in a ChoiceDialog", action),
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
        draw_multiline_text_ex(
            &body_text,
            DIALOG_X as f32 + DIALOG_SIZE as f32 / 2.0 + HUD_BORDER as f32,
            DIALOG_Y as f32 + DIALOG_TOP_SPAN as f32 + UI_BORDER as f32,
            Some(1.5),
            TextParams {
                font_size: FONT_SIZE as u16,
                font_scale: 1.0,
                color: WHITE,
                alignment: TextAlignment::Center,
                ..Default::default()
            },
        );

        // ------- Choices -----------

        draw_text(
            "(Y)es",
            (DIALOG_X + DIALOG_LEFT_SPAN + HUD_BORDER) as f32,
            (DIALOG_Y + DIALOG_SIZE - DIALOG_TOP_SPAN + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            "(N)o",
            (DIALOG_X + DIALOG_SIZE - DIALOG_LEFT_SPAN + HUD_BORDER) as f32 - (5.0 * LETTER_SIZE),
            (DIALOG_Y + DIALOG_SIZE - DIALOG_TOP_SPAN + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
    }
}

// Implementations for ChoiceDialog
impl ChoiceDialog {
    /// Builds a string representation of the items to be received in a shop offer.
    /// result example with 2 items: "a sword and a potion"
    fn build_offer_string(items: Iter<'_, Entity>, ecs_world: &World) -> String {
        let mut offer_string_arr = String::new();
        let items_length = items.len();
        for (index, item) in items.enumerate() {
            let named = ecs_world
                .get::<&Named>(*item)
                .expect("offered item is not named");
            offer_string_arr.push_str("a ");
            offer_string_arr.push_str(named.name);
            if items_length >= 2 {
                if index < items_length - 2 {
                    offer_string_arr.push_str(",\n");
                } else if index == items_length - 2 {
                    offer_string_arr.push_str("\nand a ");
                }
            }
        }
        offer_string_arr
    }
}
