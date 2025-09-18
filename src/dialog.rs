use std::collections::HashMap;

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
        actions::{WantsToDrink, WantsToEat},
        common::Named,
        player::Player,
    },
    constants::*,
    engine::state::RunState,
    utils::assets::TextureName,
};

#[derive(PartialEq, Debug)]
pub enum DialogAction {
    Eat(Entity),
    Quaff(Entity),
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
                    // TODO show equivalent inventory action
                    clear_input_queue();
                    RunState::WaitingPlayerInput
                }
                'y' => {
                    // Confirm action
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
}
