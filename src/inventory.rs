use std::collections::HashMap;

use hecs::{Component, Entity};
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
            WantsToApply, WantsToDrink, WantsToDrop, WantsToEat, WantsToEquip, WantsToFuel,
            WantsToInvoke, WantsToTrade,
        },
        common::{Named, Wet},
        items::{
            Appliable, Edible, Equippable, Equipped, Eroded, InBackback, Invokable, Item,
            MustBeFueled, Quaffable, RangedWeapon, Refiller,
        },
        player::SpecialViewMode,
    },
    constants::*,
    engine::state::{GameState, RunState},
    utils::assets::TextureName,
};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum InventoryAction {
    Eat,
    Drop,
    Invoke,
    Quaff,
    RefillWhat,
    Equip,
    Apply,
    Trade,
}

/// Inventory Item Data trasfer type: used for rendering and general inventory usage
type InventoryItemData = Vec<(Entity, String, char, (i32, i32), bool, bool, i32, bool)>;
type InventoryItem<'a> = (
    &'a Named,
    &'a Item,
    &'a InBackback,
    Option<&'a Equipped>,
    Option<&'a Eroded>,
    Option<&'a RangedWeapon>,
    Option<&'a Wet>,
);

pub struct Inventory {}

impl Inventory {
    /// Handle inventory input
    pub fn handle_input(game_state: &mut GameState, mode: InventoryAction) {
        if is_key_pressed(KeyCode::Escape) {
            // Exit inventory, clear queue to avoid to reopen on cancel
            // caused by char input queue
            clear_input_queue();

            game_state.run_state = RunState::WaitingPlayerInput;
        } else {
            //Any other key
            let mut selected_item_entity: Option<Entity> = None;
            let mut user_entity: Option<Entity> = None;

            match get_char_pressed() {
                None => {}
                Some(letterkey) => {
                    let player_entity = game_state.current_player_entity.expect("must be Some");

                    //Inventory = Named items in backpack of the Player assigned to the pressed char key
                    let inventory: InventoryItemData = match mode {
                        InventoryAction::Eat => {
                            Inventory::get_all_in_backpack_filtered_by::<Edible>(game_state)
                        }
                        InventoryAction::Invoke => {
                            Inventory::get_all_in_backpack_filtered_by::<Invokable>(game_state)
                        }
                        InventoryAction::Quaff => {
                            Inventory::get_all_in_backpack_filtered_by::<Quaffable>(game_state)
                        }
                        InventoryAction::RefillWhat => {
                            Inventory::get_all_in_backpack_filtered_by::<MustBeFueled>(game_state)
                        }
                        InventoryAction::Equip => {
                            Inventory::get_all_in_backpack_filtered_by::<Equippable>(game_state)
                        }
                        InventoryAction::Apply => {
                            Inventory::get_all_in_backpack_filtered_by::<Appliable>(game_state)
                        }
                        InventoryAction::Trade | InventoryAction::Drop => {
                            Inventory::get_all_in_backpack(game_state)
                        }
                    };

                    // Validating char input
                    let item_selected = inventory
                        .iter()
                        .find(|(_, _, assigned_char, _, _q, _, _, _)| *assigned_char == letterkey);

                    // Check if item exist for letter, then register it and go on
                    if let Some(item_sel_unwrap) = item_selected {
                        selected_item_entity = Some(item_sel_unwrap.0);
                        user_entity = Some(player_entity);
                    } else {
                        game_state
                            .game_log
                            .entries
                            .push(format!("No item available for letter {letterkey}"));
                    }
                }
            }

            // Use selected item

            game_state.run_state = RunState::DoTick;
            if let Some(item) = selected_item_entity {
                let user = user_entity.expect("user_entity is none!");
                match mode {
                    InventoryAction::Eat => {
                        let _ = game_state.ecs_world.insert_one(user, WantsToEat { item });
                    }
                    InventoryAction::Drop => {
                        let _ = game_state.ecs_world.insert_one(user, WantsToDrop { item });
                    }
                    InventoryAction::Quaff => {
                        let _ = game_state.ecs_world.insert_one(user, WantsToDrink { item });
                    }
                    InventoryAction::Invoke => {
                        let _ = game_state
                            .ecs_world
                            .insert_one(user, WantsToInvoke { item });

                        game_state.run_state =
                            RunState::MouseTargeting(SpecialViewMode::ZapTargeting);
                    }
                    InventoryAction::RefillWhat => {
                        // Select what to refill, then which item you are going to refill with
                        let wants_to_fuel = game_state.ecs_world.get::<&mut WantsToFuel>(user);
                        wants_to_fuel.expect("Must have WantsToFuel!").item = Some(item);
                    }
                    InventoryAction::Trade => {
                        // Select what to offer, then which item you are going to offer
                        let wants_to_fuel = game_state.ecs_world.get::<&mut WantsToTrade>(user);
                        wants_to_fuel.expect("Must have WantsToTrade!").item = Some(item);
                    }
                    InventoryAction::Equip => {
                        let body_location;
                        // Scope to keep the borrow check quiet
                        {
                            let equippable = game_state
                                .ecs_world
                                .get::<&Equippable>(item)
                                .expect("Should be Equippable!");
                            body_location = equippable.body_location.clone();
                        }
                        let _ = game_state.ecs_world.insert_one(
                            user,
                            WantsToEquip {
                                item,
                                body_location,
                            },
                        );
                    }
                    InventoryAction::Apply => {
                        // Applied refillers must be handled in a custom way
                        if game_state
                            .ecs_world
                            .satisfies::<&Refiller>(item)
                            .unwrap_or(false)
                        {
                            let _ = game_state.ecs_world.insert_one(
                                user,
                                WantsToFuel {
                                    item: None,
                                    with: item,
                                },
                            );

                            game_state.run_state =
                                RunState::ShowInventory(InventoryAction::RefillWhat);
                        } else {
                            let _ = game_state.ecs_world.insert_one(user, WantsToApply { item });
                            println!("Player wants to apply {:?}", item.id());
                        }
                    }
                };

                //Avoid strange behaviors
                clear_input_queue();
            } else {
                // Keep inventory showing if invalid or no item has been selected
                game_state.run_state = RunState::ShowInventory(mode);
            }
        }
    }

    pub fn draw(
        assets: &HashMap<TextureName, Texture2D>,
        game_state: &mut GameState,
        mode: &InventoryAction,
    ) {
        let texture_to_render = assets.get(&TextureName::Items).expect("Texture not found");

        //Inventory = Named items in backpack of the Player
        let inventory: InventoryItemData;
        let header_text;

        match mode {
            InventoryAction::Eat => {
                header_text = "Eat what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Edible>(game_state);
            }
            InventoryAction::Invoke => {
                header_text = "Invoke what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Invokable>(game_state);
            }
            InventoryAction::Quaff => {
                header_text = "Drink what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Quaffable>(game_state);
            }
            InventoryAction::RefillWhat => {
                header_text = "Refill what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<MustBeFueled>(game_state);
            }
            InventoryAction::Equip => {
                header_text = "Equip what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Equippable>(game_state);
            }
            InventoryAction::Trade => {
                header_text = "Trade what?";
                inventory = Inventory::get_all_in_backpack(game_state);
            }
            InventoryAction::Drop => {
                header_text = "Drop what?";
                inventory = Inventory::get_all_in_backpack(game_state);
            }
            InventoryAction::Apply => {
                header_text = "Apply what?";
                inventory = Inventory::get_all_in_backpack_filtered_by::<Appliable>(game_state);
            }
        }

        // ------- Background Rectangle -----------
        draw_rectangle(
            INVENTORY_X as f32,
            INVENTORY_Y as f32,
            INVENTORY_WIDTH as f32,
            INVENTORY_HEIGHT as f32,
            WHITE,
        );
        draw_rectangle(
            (INVENTORY_X + HUD_BORDER) as f32,
            (INVENTORY_Y + HUD_BORDER) as f32,
            (INVENTORY_WIDTH - UI_BORDER) as f32,
            (INVENTORY_HEIGHT - UI_BORDER) as f32,
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
        for (index, (_, item_name, assigned_char, item_tile, equipped, eroded, ammo, wet)) in
            inventory.iter().enumerate()
        {
            let x = (INVENTORY_X + UI_BORDER * 2) as f32;
            let y = (INVENTORY_Y + INVENTORY_TOP_SPAN) as f32
                + ((FONT_SIZE + LETTER_SIZE) * index as f32);

            // TODO show body location
            let text: String =
                build_item_string(item_name, assigned_char, equipped, eroded, ammo, wet);

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
            (INVENTORY_Y + INVENTORY_HEIGHT - UI_BORDER) as f32,
            INVENTORY_FOOTER_WIDTH as f32,
            HEADER_HEIGHT as f32,
            BLACK,
        );
        draw_text(
            "ESC to cancel",
            (INVENTORY_X + INVENTORY_LEFT_SPAN + HUD_BORDER) as f32,
            (INVENTORY_Y + INVENTORY_HEIGHT + HUD_BORDER) as f32,
            FONT_SIZE,
            WHITE,
        );
    }

    /// Get all items in backpack for UI
    fn get_all_in_backpack(game_state: &GameState) -> InventoryItemData {
        let ecs_world = &game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let mut inventory_query = ecs_world.query::<InventoryItem>();
        let mut inventory = inventory_query
            .iter()
            .filter(|(_, (_, _, in_backpack, _q, _, _, _))| in_backpack.owner.id() == player_id)
            .map(
                |(entity, (named, item, in_backpack, equipped, eroded, ranged_opt, wet_opt))| {
                    let mut ammo = -1;
                    if let Some(ranged) = ranged_opt {
                        ammo = ranged.ammo_count_total as i32;
                    }

                    (
                        entity,
                        named.name.clone(),
                        in_backpack.assigned_char,
                        item.item_tile,
                        equipped.is_some(),
                        eroded.is_some(),
                        ammo,
                        wet_opt.is_some(),
                    )
                },
            )
            .collect::<InventoryItemData>();

        //Sort alphabetically by assigned char
        inventory.sort_by_key(|k| k.2);
        inventory
    }

    /// Get all items in backpack with a certain component T for UI
    fn get_all_in_backpack_filtered_by<T: Component>(game_state: &GameState) -> InventoryItemData {
        let ecs_world = &game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let mut inventory_query = ecs_world.query::<InventoryItem>().with::<&T>();

        let mut inventory = inventory_query
            .iter()
            .filter(|(_, (_, _, in_backpack, _q, _, _, _))| in_backpack.owner.id() == player_id)
            .map(
                |(entity, (named, item, in_backpack, equipped, eroded, ranged_opt, wet_opt))| {
                    let mut ammo = -1;
                    if let Some(ranged) = ranged_opt {
                        ammo = ranged.ammo_count_total as i32;
                    }

                    (
                        entity,
                        named.name.clone(),
                        in_backpack.assigned_char,
                        item.item_tile,
                        equipped.is_some(),
                        eroded.is_some(),
                        ammo,
                        wet_opt.is_some(),
                    )
                },
            ) //
            .collect::<InventoryItemData>();

        //Sort alphabetically by assigned char
        inventory.sort_by_key(|k| k.2);
        inventory
    }
}

/// Builds a string representation of an item based on its properties.
fn build_item_string(
    item_name: &String,
    assigned_char: &char,
    equipped: &bool,
    eroded: &bool,
    ammo: &i32,
    wet: &bool,
) -> String {
    // \t is need to place item icon within its space
    let mut item_string = format!("{} : \t - ", assigned_char).to_owned();

    if *eroded {
        item_string.push_str("rusty ");
    }

    if *wet {
        item_string.push_str("wet ");
    }

    item_string.push_str(&format!("{} ", item_name));

    if *ammo >= 0 {
        item_string.push_str(&format!("({}) ", ammo));
    }

    if *equipped {
        item_string.push_str("- Equipped ");
    }

    item_string
}
