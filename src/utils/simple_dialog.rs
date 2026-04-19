use std::{collections::HashMap, slice::Iter};

use hecs::{Entity, World};
use macroquad::{
    color::{BLACK, WHITE},
    input::{KeyCode, clear_input_queue, get_char_pressed, get_key_pressed},
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

/// Show a simple dialog that can be closed
pub struct SimpleDialog {}

impl Dialog<DialogAction> for SimpleDialog {
    /// Handle dialog input
    fn handle_input(game_state: &mut GameState, action: DialogAction) {
        //Just handle Esc, Enter or Space to close the dialog
        match get_key_pressed() {
            Some(key) => match key {
                KeyCode::Enter | KeyCode::Escape | KeyCode::Space => {
                    game_state.run_state = RunState::WaitingPlayerInput
                }
                _ => game_state.run_state = RunState::ShowDialog(action),
            },
            None => game_state.run_state = RunState::ShowDialog(action),
        }
    }

    fn draw(_: &HashMap<TextureName, Texture2D>, _: &World, action: &DialogAction) {
        let body_text: String = match action {
            DialogAction::ShowMessage(message) => message.to_string(),
            _ => panic!("Cannot handle DialogAction {:?} in a SimpleDialog", action),
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

        draw_multiline_text_ex(
            "(Esc)ape, Space or Enter to close",
            DIALOG_X as f32 + DIALOG_SIZE as f32 / 2.0 + HUD_BORDER as f32,
            (DIALOG_Y + DIALOG_SIZE - DIALOG_TOP_SPAN + HUD_BORDER) as f32,
            Some(1.5),
            TextParams {
                font_size: FONT_SIZE as u16,
                font_scale: 1.0,
                color: WHITE,
                alignment: TextAlignment::Center,
                ..Default::default()
            },
        );
    }
}
