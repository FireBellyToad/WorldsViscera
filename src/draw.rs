use std::collections::HashMap;

use hecs::World;
use macroquad::{
    color::{BLACK, Color, DARKGRAY, RED, WHITE, YELLOW},
    input::mouse_position,
    math::Rect,
    shapes::{draw_circle, draw_rectangle, draw_rectangle_lines},
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    components::{
        combat::CombatStats,
        common::{GameLog, Position, Renderable},
        health::Hunger,
        player::Player,
    },
    constants::*,
    engine::state::{EngineState, RunState},
    inventory::{Inventory, InventoryAction},
    maps::game_map::GameMap,
    systems::hunger_check::HungerStatus,
    utils::assets::TextureName,
};

pub struct Draw {}

impl Draw {
    pub fn render_game(game_state: &EngineState, assets: &HashMap<TextureName, Texture2D>) {
        let mut maps = game_state.ecs_world.query::<&GameMap>();
        match game_state.run_state {
            RunState::GameOver => Draw::game_over(),
            _ => {
                for (_e, map) in &mut maps {
                    Draw::map(&map, assets);
                    Draw::renderables(&game_state.ecs_world, &assets, &map);
                }

                //Overlay
                match game_state.run_state {
                    RunState::ShowEatInventory => {
                        Inventory::draw(assets, &game_state.ecs_world, InventoryAction::Eat)
                    }
                    RunState::ShowDropInventory => {
                        Inventory::draw(assets, &game_state.ecs_world, InventoryAction::Drop)
                    }
                    RunState::ShowInvokeInventory => {
                        Inventory::draw(assets, &game_state.ecs_world, InventoryAction::Invoke)
                    }
                    RunState::MouseTargeting => {
                        Draw::targeting(&game_state.ecs_world);
                    }
                    _ => {}
                }
            }
        }
        Draw::game_log(&game_state.ecs_world);
    }

    /// Draw HUD
    fn game_log(ecs_world: &World) {
        // ------- Background Rectangle -----------
        draw_rectangle(
            UI_BORDER_F32,
            (MAP_HEIGHT * TILE_SIZE) as f32 + 2.0 * UI_BORDER as f32,
            HUD_WIDTH as f32,
            HUD_HEIGHT as f32,
            WHITE,
        );
        draw_rectangle(
            (HUD_BORDER + UI_BORDER) as f32,
            (HUD_BORDER + MAP_HEIGHT * TILE_SIZE) as f32 + 2.0 * UI_BORDER as f32,
            (HUD_WIDTH - UI_BORDER) as f32,
            (HUD_HEIGHT - UI_BORDER) as f32,
            BLACK,
        );

        // ------- Stat Text header -----------

        Draw::hud_header(ecs_world);

        // ------- Messages log  -----------

        let mut game_log_query = ecs_world.query::<&GameLog>();
        let (_e, game_log) = game_log_query
            .iter()
            .last()
            .expect("Game log is not in hecs::World");

        // Going backwards to get last message on top
        for (index, message) in game_log.entries.iter().rev().enumerate() {
            draw_text(
                format!("- {message}"),
                (HUD_BORDER + (UI_BORDER * 2)) as f32,
                (HUD_BORDER
                    + 32
                    + (UI_BORDER * 4)
                    + (MAP_HEIGHT * TILE_SIZE)
                    + ((MAX_MESSAGES_IN_LOG - index) as i32 * 32)) as f32,
                FONT_SIZE,
                WHITE,
            );

            // Show only the last 5 messages
            if index == MAX_MESSAGES_IN_LOG {
                break;
            }
        }
    }

    fn hud_header(ecs_world: &World){
        let mut player_query = ecs_world.query::<(&Player, &CombatStats, &Hunger)>();
        let (_e, (_p, player_stats, hunger)) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        let sta_text = format!(
            "STA: {}/{}",
            player_stats.current_stamina, player_stats.max_stamina
        );
        let sta_text_len = sta_text.len();

        let tou_text = format!(
            "TOU {}/{}",
            player_stats.current_toughness, player_stats.max_toughness
        );
        let tou_text_len = tou_text.len();

        let dex_text = format!(
            "DEX {}/{}",
            player_stats.current_dexterity, player_stats.max_dexterity
        );
        let dex_text_len = dex_text.len();

        let hunger_status = &hunger.current_status;
        let hunger_text = format!("Hunger:{:?}", hunger_status);
        let hunger_text_len = hunger_text.len();

        draw_rectangle(
            (HEADER_LEFT_SPAN + HUD_BORDER) as f32,
            (MAP_HEIGHT * TILE_SIZE) as f32 + UI_BORDER as f32,
            4.0 * LETTER_SIZE
                - HUD_BORDER as f32 * 3.0
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE)
                + (dex_text_len as f32 * LETTER_SIZE)
                + (hunger_text_len as f32 * LETTER_SIZE),
            HEADER_HEIGHT as f32,
            BLACK,
        );

        let mut text_color = WHITE;

        // Draw Stamina (STA)
        if player_stats.current_stamina == 0 {
            text_color = RED;
        } else if player_stats.current_stamina <= player_stats.max_stamina / 2 {
            text_color = YELLOW;
        }

        Self::draw_stat_text(sta_text, 0.0, text_color);

        // Draw Toughness (TOU)
        text_color = WHITE;

        if player_stats.current_toughness < player_stats.max_toughness {
            text_color = YELLOW;
        }

        Self::draw_stat_text(
            tou_text,
            LETTER_SIZE + (sta_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // Draw Dexterity (DEX)
        text_color = WHITE;

        if player_stats.current_dexterity < player_stats.max_dexterity {
            text_color = YELLOW;
        }

        Self::draw_stat_text(
            dex_text,
            2.0 * LETTER_SIZE
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        match hunger_status {
            HungerStatus::Hungry => text_color = YELLOW,
            HungerStatus::Starved => text_color = RED,
            _ => text_color = WHITE,
        }

        // TODO improve
        Self::draw_stat_text(
            hunger_text,
            3.0 * LETTER_SIZE
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE)
                + (dex_text_len as f32 * LETTER_SIZE),
            text_color,
        );
    }

    fn draw_stat_text(text: String, left_pad: f32, text_color: Color) {
        draw_text(
            text,
            (HUD_BORDER + HEADER_LEFT_SPAN + UI_BORDER) as f32 + left_pad,
            (HUD_BORDER + UI_BORDER * 3 + MAP_HEIGHT * TILE_SIZE) as f32,
            FONT_SIZE,
            text_color,
        );
    }

    /// Draw all Renderable entities in World
    fn renderables(world: &World, assets: &HashMap<TextureName, Texture2D>, map: &GameMap) {
        //Get all entities in readonly
        let mut renderables_with_position = world.query::<(&Renderable, &Position)>();

        let mut renderables_vec: Vec<(hecs::Entity, (&Renderable, &Position))> =
            renderables_with_position.iter().collect();
        renderables_vec.sort_by_key(|(_e, (renderable, _p))| renderable.z_index);

        for (_e, (renderable, position)) in renderables_vec {
            let texture_to_render = assets
                .get(&renderable.texture_name)
                .expect("Texture not found");

            if map.visible_tiles[GameMap::get_index_from_xy(position.x, position.y)] {
                // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                draw_texture_ex(
                    texture_to_render,
                    (UI_BORDER + (position.x * TILE_SIZE)) as f32,
                    (UI_BORDER + (position.y * TILE_SIZE)) as f32,
                    WHITE, // Seems like White color is needed to normal render
                    DrawTextureParams {
                        source: Some(renderable.texture_region),
                        ..Default::default()
                    },
                );
            }
        }
    }

    /// Draw game over screen
    fn game_over() {
        draw_rectangle(0.0, 0.0, 64.0, 32.0, BLACK);
        draw_text("YOU ARE DEAD", 32.0, 64.0, FONT_SIZE * 2.0, WHITE);
        draw_text(
            "Press R to restart, Q to exit",
            32.0,
            96.0,
            FONT_SIZE,
            WHITE,
        );
    }

    /// Draw target on tile where mouse is poiting
    fn targeting(ecs_world: &World) {
        draw_text(
            "Use mouse to aim, ESC to cancel",
            24.0,
            48.0,
            FONT_SIZE,
            WHITE,
        );
        let (mouse_x, mouse_y) = mouse_position();

        let mut map_query = ecs_world.query::<&GameMap>();
        let (_e, map) = map_query
            .iter()
            .last()
            .expect("GameMap is not in hecs::World");

        let rounded_x = (((mouse_x - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;
        let rounded_y = (((mouse_y - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;

        // Draw target if tile is visible
        let index = GameMap::get_index_from_xy(rounded_x, rounded_y);
        if map.visible_tiles.len() > index && map.visible_tiles[index] {
            draw_rectangle_lines(
                (UI_BORDER + (rounded_x * TILE_SIZE)) as f32,
                (UI_BORDER + (rounded_y * TILE_SIZE)) as f32,
                TILE_SIZE_F32,
                TILE_SIZE_F32,
                3.0,
                RED,
            );
        }
    }

    /// Draws map
    pub fn map(game_map: &GameMap, assets: &HashMap<TextureName, Texture2D>) {
        let texture_to_render = assets.get(&TextureName::Tiles).expect("Texture not found");

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile_to_draw = GameMap::get_index_from_xy(x, y);
                let tile_index =
                    GameMap::get_tile_sprite_sheet_index(&game_map.tiles[tile_to_draw])
                        * TILE_SIZE_F32;

                if game_map.revealed_tiles[tile_to_draw] {
                    let mut alpha = DARKGRAY;

                    if game_map.visible_tiles[tile_to_draw] {
                        alpha = WHITE;
                        if game_map.bloodied_tiles.contains(&tile_to_draw) {
                            Draw::draw_blood_blots(x, y);
                        }
                    }

                    // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                    draw_texture_ex(
                        texture_to_render,
                        (UI_BORDER + (x * TILE_SIZE)) as f32,
                        (UI_BORDER + (y * TILE_SIZE)) as f32,
                        alpha,
                        DrawTextureParams {
                            source: Some(Rect {
                                x: tile_index,
                                y: 0.0,
                                w: TILE_SIZE_F32,
                                h: TILE_SIZE_F32,
                            }),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    /// Utility for drawing blood blots
    pub fn draw_blood_blots(x: i32, y: i32) {
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 - 6.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 - 7.0,
            2.0,
            Color::from_rgba(255, 10, 10, 32),
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 + 4.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 - 4.0,
            2.0,
            Color::from_rgba(255, 10, 10, 32),
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 - 5.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 + 4.0,
            1.0,
            Color::from_rgba(255, 10, 10, 32),
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 + 3.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 + 5.0,
            2.0,
            Color::from_rgba(255, 10, 10, 32),
        );

        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32,
            4.0,
            Color::from_rgba(255, 10, 10, 32),
        );
    }
}
