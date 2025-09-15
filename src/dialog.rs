use std::collections::HashMap;

use hecs::{Entity, World};
use macroquad::{
    color::{BLACK, WHITE},
    input::{KeyCode, clear_input_queue, get_char_pressed, is_key_pressed},
    shapes::draw_rectangle,
    text::draw_text,
    texture::Texture2D,
};

use crate::{constants::*, engine::state::RunState, utils::assets::TextureName};

pub struct Dialog {}

impl Dialog {
    /// Handle dialog input
    pub fn handle_input(ecs_world: &mut World, mode: UIAction) -> RunState {
        if is_key_pressed(KeyCode::Escape) {
            // Exit dialog, clear queue to avoid to reopen on cancel
            // caused by char input queue
            clear_input_queue();
            return RunState::WaitingPlayerInput;
        } else {
            //Any other key
            let mut selected_item_entity: Option<Entity> = None;
            let mut user_entity: Option<Entity> = None;

            match get_char_pressed() {
                None => {}
                Some(letterkey) => {}
            }
            let mut new_run_state = RunState::ShowDialog(mode);

            //Avoid strange behaviors
            clear_input_queue();
            return new_run_state;
        }
    }

    pub fn draw(assets: &HashMap<TextureName, Texture2D>, ecs_world: &World, mode: &UIAction) {
        let body_text = "This is a dialog";

        // ------- Background Rectangle -----------
        draw_rectangle(DIALOG_X as f32, DIALOG_Y as f32, 512.0, 512.0, WHITE);
        draw_rectangle(
            (DIALOG_X + HUD_BORDER) as f32,
            (DIALOG_Y + HUD_BORDER) as f32,
            (DIALOG_SIZE - UI_BORDER) as f32,
            (DIALOG_SIZE - UI_BORDER) as f32,
            BLACK,
        );

        // ------- Header -----------

        draw_text(
            body_text,
            (DIALOG_X + DIALOG_LEFT_SPAN + HUD_BORDER) as f32,
            (DIALOG_Y + DIALOG_TOP_SPAN + UI_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );

        // ------- Footer -----------
        draw_text(
            "Yes",
            (DIALOG_X + DIALOG_SIZE - DIALOG_LEFT_SPAN + HUD_BORDER) as f32 - (3.0 * LETTER_SIZE) ,
            (DIALOG_Y + DIALOG_SIZE - DIALOG_TOP_SPAN + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            "No",
            (DIALOG_X + DIALOG_LEFT_SPAN + HUD_BORDER) as f32,
            (DIALOG_Y + DIALOG_SIZE - DIALOG_TOP_SPAN + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
    }
}
