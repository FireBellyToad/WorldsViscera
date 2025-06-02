use std::collections::HashMap;

use hecs::World;
use macroquad::{
    color::{BLACK, WHITE},
    input::{KeyCode, clear_input_queue, get_key_pressed, is_key_pressed},
    math::Rect,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    assets::TextureName,
    components::{
        common::Named,
        items::{InBackback, Item},
        player::Player,
    },
    constants::*,
    engine::state::RunState,
};

pub struct Inventory {}

impl Inventory {
    pub fn handle_input() -> RunState {
        match get_key_pressed() {
            None => {}
            Some(key) => match key {
                KeyCode::Escape => {
                    return RunState::WaitingPlayerInput;
                }
                _ => {}
            },
        }

        RunState::ShowInventory
    }

    pub fn draw(assets: &HashMap<TextureName, Texture2D>, ecs_world: &World) {
        let texture_to_render = assets.get(&TextureName::Items).expect("Texture not found");

        let mut player_query = ecs_world.query::<&Player>();
        let (player_entity, _p) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        //Inventory = Named items in backpack of the Player
        let mut inventory_query = ecs_world.query::<(&Named, &Item, &InBackback)>();
        let inventory: Vec<(hecs::Entity, (&Named, &Item, &InBackback))> = inventory_query
            .iter()
            .filter(|(_e, (_n, _i, in_backpack))| in_backpack.owner.id() == player_entity.id()) //
            .collect::<Vec<_>>();

        // ------- Background Rectangle -----------
        draw_rectangle(INVENTORY_X as f32, INVENTORY_Y as f32, 512.0, 512.0, WHITE);
        draw_rectangle(
            (INVENTORY_X + HUD_BORDER) as f32,
            (INVENTORY_Y + HUD_BORDER) as f32,
            (INVENTORY_SIZE - UI_BORDER) as f32,
            (INVENTORY_SIZE - UI_BORDER) as f32,
            BLACK,
        );

        // ------- Header -----------
        draw_rectangle(
            (INVENTORY_X + INVENTORY_LEFT_SPAN) as f32,
            (INVENTORY_Y - UI_BORDER) as f32,
            INVENTORY_HEADER_WIDTH as f32,
            HEADER_HEIGHT as f32,
            BLACK,
        );
        draw_text(
            "Inventory",
            (INVENTORY_X + INVENTORY_LEFT_SPAN + HUD_BORDER) as f32,
            (INVENTORY_Y + UI_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );

        // ------- Item List -----------
        for (index, (_e, (named, _i, _b))) in inventory.iter().enumerate() {
            let x = (INVENTORY_X + UI_BORDER * 2) as f32;
            let y = (INVENTORY_Y + INVENTORY_TOP_SPAN) as f32 + (FONT_SIZE * index as f32);

            draw_text(
                format!("{} - \t - {}", index + 1, named.name),
                x,
                y,
                FONT_SIZE,
                WHITE,
            );
            // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
            draw_texture_ex(
                texture_to_render,
                (UI_BORDER + (x as i32 + TILE_SIZE + HUD_BORDER)) as f32,
                (UI_BORDER + (y as i32 - TILE_SIZE - UI_BORDER)) as f32,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect {
                        x: 0.0, // TODO refactor
                        y: 0.0,
                        w: TILE_SIZE as f32,
                        h: TILE_SIZE as f32,
                    }),
                    ..Default::default()
                },
            );
        }

        // ------- Footer -----------
        draw_rectangle(
            (INVENTORY_X + INVENTORY_LEFT_SPAN) as f32,
            (INVENTORY_Y + INVENTORY_SIZE - UI_BORDER) as f32,
            INVENTORY_FOOTER_WIDTH as f32,
            HEADER_HEIGHT as f32,
            BLACK,
        );
        draw_text(
            "ESC to cancel",
            (INVENTORY_X + INVENTORY_LEFT_SPAN + HUD_BORDER) as f32,
            (INVENTORY_Y + INVENTORY_SIZE + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
    }
}
