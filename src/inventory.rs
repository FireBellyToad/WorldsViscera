use std::collections::HashMap;

use hecs::{Component, Entity, World};
use macroquad::{
    color::{BLACK, WHITE},
    input::{KeyCode, clear_input_queue, get_char_pressed, is_key_pressed},
    math::Rect,
    shapes::draw_rectangle,
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    components::{
        actions::{
            WantsToApply, WantsToDrink, WantsToDrop, WantsToEat, WantsToEquip, WantsToFuel, WantsToInvoke
        },
        common::{GameLog, Named},
        items::{
            Appliable, Edible, Equippable, Equipped, InBackback, Invokable, Item, ProduceLight,
            Quaffable, Refiller,
        },
        player::{Player, SpecialViewMode},
    },
    constants::*,
    engine::state::RunState,
    utils::assets::TextureName,
};

#[derive(PartialEq, Debug)]
pub enum InventoryAction {
    Eat,
    Drop,
    Invoke,
    Quaff,
    RefillWhat,
    RefillWith,
    Equip,
    Apply,
}

/// Inventory Item Data trasfer type: used for rendering and general inventory usage
type InventoryItemDtt = Vec<(Entity, String, char, (i32, i32), bool)>;

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
                    let (player_entity, _) = player_query
                        .iter()
                        .last()
                        .expect("Player is not in hecs::World");

                    //Log
                    let mut game_log_query = ecs_world.query::<&mut GameLog>();
                    let (_, game_log) = game_log_query
                        .iter()
                        .last()
                        .expect("Game log is not in hecs::World");

                    //Inventory = Named items in backpack of the Player assigned to the pressed char key
                    let inventory: InventoryItemDtt = match mode {
                        InventoryAction::Eat => {
                            Inventory::get_all_in_backpack_filtered_by::<Edible>(ecs_world)
                        }
                        InventoryAction::Invoke => {
                            Inventory::get_all_in_backpack_filtered_by::<Invokable>(ecs_world)
                        }
                        InventoryAction::Quaff => {
                            Inventory::get_all_in_backpack_filtered_by::<Quaffable>(ecs_world)
                        }
                        InventoryAction::RefillWhat => {
                            Inventory::get_all_in_backpack_filtered_by::<ProduceLight>(ecs_world)
                        }
                        InventoryAction::RefillWith => {
                            Inventory::get_all_in_backpack_filtered_by::<Refiller>(ecs_world)
                        }
                        InventoryAction::Equip => {
                            Inventory::get_all_in_backpack_filtered_by::<Equippable>(ecs_world)
                        }
                        InventoryAction::Apply => {
                            Inventory::get_all_in_backpack_filtered_by::<Appliable>(ecs_world)
                        }
                        InventoryAction::Drop => Inventory::get_all_in_backpack(ecs_world),
                    };

                    // Validating char input
                    let item_selected = inventory
                        .iter()
                        .find(|(_, _, assigned_char, _, _q)| *assigned_char == letterkey);

                    // Check if item exist for letter, then register it and go on
                    if let Some(item_sel_unwrap) = item_selected {
                        selected_item_entity = Some(item_sel_unwrap.0);
                        user_entity = Some(player_entity);
                    } else {
                        game_log
                            .entries
                            .push(format!("No item available for letter {letterkey}"));
                    }
                }
            }

            // Use selected item
            let mut new_run_state = RunState::DoTick;
            if let Some(item) = selected_item_entity {
                let user = user_entity.expect("user_entity is none!");
                match mode {
                    InventoryAction::Eat => {
                        let _ = ecs_world.insert_one(user, WantsToEat { item });
                    }
                    InventoryAction::Drop => {
                        let _ = ecs_world.insert_one(user, WantsToDrop { item });
                    }
                    InventoryAction::Quaff => {
                        let _ = ecs_world.insert_one(user, WantsToDrink { item });
                    }
                    InventoryAction::Invoke => {
                        let _ = ecs_world.insert_one(user, WantsToInvoke { item });
                        new_run_state = RunState::MouseTargeting(SpecialViewMode::ZapTargeting);
                    }
                    InventoryAction::RefillWhat => {
                        // Select what to refill, then which item you are going to refill with
                        let _ = ecs_world.insert_one(user, WantsToFuel { item, with: None });
                        new_run_state = RunState::ShowInventory(InventoryAction::RefillWith);
                    }
                    InventoryAction::RefillWith => {
                        let wants_to_fuel = ecs_world.get::<&mut WantsToFuel>(user);
                        wants_to_fuel.expect("Must Want to Fuel!").with = Some(item);
                    }
                    InventoryAction::Equip => {
                        let body_location;
                        // Scope to keep the borrow check quiet
                        {
                            let equippable = ecs_world
                                .get::<&Equippable>(item)
                                .expect("Should be Equippable!");
                            body_location = equippable.body_location.clone();
                        }
                        let _ = ecs_world.insert_one(
                            user,
                            WantsToEquip {
                                item,
                                body_location,
                            },
                        );
                    }
                    InventoryAction::Apply => {
                        let _ = ecs_world.insert_one(user, WantsToApply { item });
                        println!("Player wants to apply {:?}", item.id());
                    }
                };

                //Avoid strange behaviors
                clear_input_queue();
                return new_run_state;
            }
        }

        // Keep inventory showing if invalid or no item has been selected
        RunState::ShowInventory(mode)
    }

    pub fn draw(
        assets: &HashMap<TextureName, Texture2D>,
        ecs_world: &World,
        mode: &InventoryAction,
    ) {
        let texture_to_render = assets.get(&TextureName::Items).expect("Texture not found");

        //Inventory = Named items in backpack of the Player
        let inventory: InventoryItemDtt;
        let header_text;

        match mode {
            InventoryAction::Eat => {
                header_text = "Eat what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Edible>(ecs_world);
            }
            InventoryAction::Invoke => {
                header_text = "Invoke what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Invokable>(ecs_world);
            }
            InventoryAction::Quaff => {
                header_text = "Drink what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Quaffable>(ecs_world);
            }
            InventoryAction::RefillWhat => {
                header_text = "Refill what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<ProduceLight>(ecs_world);
            }
            InventoryAction::RefillWith => {
                header_text = "With what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Refiller>(ecs_world);
            }
            InventoryAction::Equip => {
                header_text = "Equip what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Equippable>(ecs_world);
            }
            InventoryAction::Drop => {
                header_text = "Drop what?";
                inventory = Inventory::get_all_in_backpack(ecs_world);
            }
            InventoryAction::Apply => {
                header_text = "Apply what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Appliable>(ecs_world);
            }
        }

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
            header_text.len() as f32 * LETTER_SIZE,
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
        for (index, (_, item_name, assigned_char, item_tile, equipped)) in
            inventory.iter().enumerate()
        {
            let x = (INVENTORY_X + UI_BORDER * 2) as f32;
            let y = (INVENTORY_Y + INVENTORY_TOP_SPAN) as f32
                + ((FONT_SIZE + LETTER_SIZE) * index as f32);

            let text: String = if *equipped {
                // TODO show body location
                format!("{} : \t - {} - Equipped", assigned_char, item_name)
            } else {
                format!("{} : \t - {}", assigned_char, item_name)
            };

            draw_text(text, x, y, FONT_SIZE, WHITE);

            // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
            draw_texture_ex(
                texture_to_render,
                (UI_BORDER + (x as i32 + TILE_SIZE + ITEM_INVENTORY_LEFT_SPAN + HUD_BORDER)) as f32,
                (UI_BORDER + (y as i32 - TILE_SIZE - ITEM_INVENTORY_TOP_SPAN)) as f32,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect {
                        x: (item_tile.0 * TILE_SIZE) as f32,
                        y: (item_tile.1 * TILE_SIZE) as f32,
                        w: TILE_SIZE_F32,
                        h: TILE_SIZE_F32,
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

    /// Get all items in backpack for UI
    fn get_all_in_backpack(ecs_world: &World) -> InventoryItemDtt {
        let player_id = Player::get_entity_id(ecs_world);
        let mut inventory_query =
            ecs_world.query::<(&Named, &Item, &InBackback, Option<&Equipped>)>();
        let mut inventory = inventory_query
            .iter()
            .filter(|(_, (_, _, in_backpack, _q))| in_backpack.owner.id() == player_id)
            .map(|(entity, (named, item, in_backpack, equipped))| {
                (
                    entity,
                    named.name.clone(),
                    in_backpack.assigned_char,
                    item.item_tile,
                    equipped.is_some(),
                )
            })
            .collect::<InventoryItemDtt>();

        //Sort alphabetically by assigned char
        inventory.sort_by_key(|k| k.2);
        inventory
    }

    /// Get all items in backpack with a certain component T for UI
    fn get_all_in_backpack_filtered_by<T: Component>(ecs_world: &World) -> InventoryItemDtt {
        let player_id = Player::get_entity_id(ecs_world);

        let mut inventory_query = ecs_world
            .query::<(&Named, &Item, &InBackback, Option<&Equipped>)>()
            .with::<&T>();

        let mut inventory = inventory_query
            .iter()
            .filter(|(_, (_, _, in_backpack, _q))| in_backpack.owner.id() == player_id)
            .map(|(entity, (named, item, in_backpack, equipped))| {
                (
                    entity,
                    named.name.clone(),
                    in_backpack.assigned_char,
                    item.item_tile,
                    equipped.is_some(),
                )
            }) //
            .collect::<InventoryItemDtt>();

        //Sort alphabetically by assigned char
        inventory.sort_by_key(|k| k.2);
        inventory
    }
}
