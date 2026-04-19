use std::collections::HashMap;

use hecs::World;
use macroquad::{
    color::{BLACK, WHITE},
    input::{KeyCode, get_key_pressed},
    shapes::draw_rectangle,
    text::{TextAlignment, TextParams, draw_multiline_text_ex, draw_text},
    texture::Texture2D,
};

use crate::{
    constants::*,
    engine::state::{GameState, RunState},
    utils::{
        assets::TextureName,
        dialog::{Dialog, DialogAction},
    },
};

/// Show a simple dialog that can be closed
pub struct SimpleDialog {}

impl Dialog for SimpleDialog {
    /// Handle dialog input
    fn handle_input(game_state: &mut GameState, action: DialogAction) {
        match get_key_pressed() {
            //Just handle Esc, Enter or Space to close the dialog
            Some(KeyCode::Enter | KeyCode::Escape | KeyCode::Space) => {
                game_state.run_state = RunState::WaitingPlayerInput
            }
            _ => game_state.run_state = RunState::ShowDialog(action),
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

        // ------- Footer -----------
        let footer_text = "ESC, Enter or Space to close";
        draw_rectangle(
            (DIALOG_X + INVENTORY_LEFT_SPAN) as f32,
            (DIALOG_Y + DIALOG_SIZE - UI_BORDER) as f32,
            footer_text.len() as f32 * LETTER_SIZE - HUD_BORDER as f32 * 3.0,
            HEADER_HEIGHT as f32,
            BLACK,
        );
        draw_text(
            footer_text,
            (DIALOG_X + INVENTORY_LEFT_SPAN + HUD_BORDER) as f32,
            (DIALOG_Y + DIALOG_SIZE + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
    }
}
