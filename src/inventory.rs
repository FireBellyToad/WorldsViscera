use std::collections::HashMap;

use hecs::{Entity, World};
use macroquad::{
    color::{BLACK, WHITE},
    input::{KeyCode, clear_input_queue, get_char_pressed, is_key_pressed},
    math::Rect,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    assets::TextureName,
    components::{
        common::{GameLog, Named},
        items::{InBackback, Item, WantsToDrop, WantsToEat},
        player::Player,
    },
    constants::*,
    engine::state::RunState,
};

#[derive(PartialEq, Debug)]
pub enum InventoryAction {
    Eat,
    Drop,
}

pub struct Inventory {}

impl Inventory {
    /// Handle inventory input
    pub fn handle_input(ecs_world: &mut World, mode: InventoryAction) -> RunState {
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
                    let mut player_query = ecs_world.query::<&Player>();
                    let (player_entity, _p) = player_query
                        .iter()
                        .last()
                        .expect("Player is not in hecs::World");

                    //Log
                    let mut game_log_query = ecs_world.query::<&mut GameLog>();
                    let (_e, game_log) = game_log_query
                        .iter()
                        .last()
                        .expect("Game log is not in hecs::World");

                    //Inventory = Named items in backpack of the Player assigned to the pressed char key
                    let mut inventory_query = ecs_world.query::<(&Named, &Item, &InBackback)>();
                    let inventory: Vec<(hecs::Entity, (&Named, &Item, &InBackback))> =
                        inventory_query
                            .iter()
                            .filter(|(_e, (_n, _i, in_backpack))| {
                                in_backpack.owner.id() == player_entity.id()
                                    && in_backpack.assigned_char == letterkey
                            })
                            .collect::<Vec<_>>();

                    // Validating char input
                    if !inventory.is_empty() {
                        let (item_entity, (_n, _i, _b)) = inventory[0];
                        selected_item_entity = Some(item_entity);
                        user_entity = Some(player_entity);
                    } else {
                        game_log
                            .entries
                            .push(format!("No item available for letter {letterkey}"));
                    }
                }
            }

            // Use selected item
            if selected_item_entity.is_some() {
                let item: Entity = selected_item_entity.unwrap();
                match mode {
                    InventoryAction::Eat => {
                        let _ = ecs_world.insert_one(user_entity.unwrap(), WantsToEat { item });
                    }
                    InventoryAction::Drop => {
                        let _ = ecs_world.insert_one(user_entity.unwrap(), WantsToDrop { item });
                    }
                };

                //Avoid strange behaviors 
                clear_input_queue();
                return RunState::PlayerTurn;
            }
        }

        // Keep inventory showing if invalid or no item has been selected
        match mode {
            InventoryAction::Eat => RunState::ShowInventory,
            InventoryAction::Drop => RunState::ShowDropInventory,
        }
    }

    pub fn draw(
        assets: &HashMap<TextureName, Texture2D>,
        ecs_world: &World,
        mode: InventoryAction,
    ) {
        let texture_to_render = assets.get(&TextureName::Items).expect("Texture not found");

        let player_id = Player::get_player_id(ecs_world);

        //Inventory = Named items in backpack of the Player
        let mut inventory_query = ecs_world.query::<(&Named, &Item, &InBackback)>();
        let mut inventory: Vec<(hecs::Entity, (&Named, &Item, &InBackback))> = inventory_query
            .iter()
            .filter(|(_e, (_n, _i, in_backpack))| in_backpack.owner.id() == player_id) //
            .collect::<Vec<_>>();

        //Sort alphabetically by assigned char
        inventory.sort_by_key(|k| k.1.2.assigned_char);

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
        let header_text;
        match mode {
            InventoryAction::Eat => {
                header_text = "Eat what?";
            }
            InventoryAction::Drop => {
                header_text = "Drop what?";
            }
        }

        draw_rectangle(
            (INVENTORY_X + INVENTORY_LEFT_SPAN) as f32,
            (INVENTORY_Y - UI_BORDER) as f32,
            header_text.len() as f32 * 15.0,
            HEADER_HEIGHT as f32,
            BLACK,
        );

        draw_text(
            header_text,
            (INVENTORY_X + INVENTORY_LEFT_SPAN + HUD_BORDER) as f32,
            (INVENTORY_Y + UI_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );

        // ------- Item List -----------
        for (index, (_e, (named, _i, in_backpack))) in inventory.iter().enumerate() {
            let x = (INVENTORY_X + UI_BORDER * 2) as f32;
            let y = (INVENTORY_Y + INVENTORY_TOP_SPAN) as f32 + (FONT_SIZE * index as f32);

            draw_text(
                format!("{} : \t - {}", in_backpack.assigned_char, named.name),
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
