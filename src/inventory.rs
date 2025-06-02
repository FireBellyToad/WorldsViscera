use std::collections::HashMap;

use hecs::{Entity, World};
use macroquad::{
    color::{BLACK, WHITE},
    input::{clear_input_queue, get_char_pressed, is_key_pressed, KeyCode},
    math::Rect,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

use crate::{
    assets::TextureName,
    components::{
        common::{Named, WantsToEat},
        items::{InBackback, Item},
        player::Player,
    },
    constants::*,
    engine::state::RunState,
};

pub struct Inventory {}

impl Inventory {
    /// Handle inventory input
    pub fn handle_input(ecs_world: &mut World) -> RunState {
        if is_key_pressed(KeyCode::Escape) {
            // Exit inventory, clear queue to avoid to reopen on cancel 
            // caused by char input queue
            clear_input_queue();
            return RunState::WaitingPlayerInput;
        } else {

            //Any other key
            let mut selected_item_entity: Option<Entity> = None;
            let mut user_entity: Option<Entity> = None;

            match get_char_pressed() {
                None => {}
                Some(letterkey) => {
                    // Get the selection and use the item
                    let search_result = OPTION_TO_CHAR_MAP.iter().position(|l| *l == letterkey);

                    // is the selection valid?
                    if search_result.is_some()
                    {
                        let selected_item_index = search_result.unwrap();

                        let mut player_query = ecs_world.query::<&Player>();
                        let (player_entity, _p) = player_query
                            .iter()
                            .last()
                            .expect("Player is not in hecs::World");

                        //Inventory = Named items in backpack of the Player
                        let mut inventory_query = ecs_world.query::<(&Named, &Item, &InBackback)>();
                        let inventory: Vec<(hecs::Entity, (&Named, &Item, &InBackback))> =
                            inventory_query
                                .iter()
                                .filter(|(_e, (_n, _i, in_backpack))| {
                                    in_backpack.owner.id() == player_entity.id()
                                }) //
                                .collect::<Vec<_>>();

                        // Validating char input
                        if selected_item_index < inventory.len() {
                            let (item_entity, (_n, _i, _b)) = inventory[selected_item_index];
                            selected_item_entity = Some(item_entity);
                            user_entity = Some(player_entity);
                        }
                    }
                    println!("Player does not have item for selection \"{}\"", letterkey);
                }
            }

            // Use selected item
            if selected_item_entity.is_some() {
                
                let item_entity = selected_item_entity.unwrap();
                let _ = ecs_world.insert_one(
                    user_entity.unwrap(),
                    WantsToEat {
                        edible: item_entity,
                    },
                );
                return RunState::PlayerTurn;
            }
        }

        // Keep inventory showing if invalid or no item has been selected
        RunState::ShowInventory
    }

    pub fn draw(assets: &HashMap<TextureName, Texture2D>, ecs_world: &World) {
        let texture_to_render = assets.get(&TextureName::Items).expect("Texture not found");

        let player_id = Player::get_player_id(ecs_world);

        //Inventory = Named items in backpack of the Player
        let mut inventory_query = ecs_world.query::<(&Named, &Item, &InBackback)>();
        let inventory: Vec<(hecs::Entity, (&Named, &Item, &InBackback))> = inventory_query
            .iter()
            .filter(|(_e, (_n, _i, in_backpack))| in_backpack.owner.id() == player_id) //
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
                format!("{} : \t - {}", OPTION_TO_CHAR_MAP[index], named.name),
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
