use std::collections::HashMap;

use hecs::World;
use macroquad::{
    color::{BLACK, Color, RED, WHITE, YELLOW},
    input::mouse_position,
    shapes::{draw_rectangle, draw_rectangle_lines},
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    components::{
        combat::CombatStats,
        common::{GameLog, Position, Renderable},
        map::{Map, get_index_from_xy},
        player::Player,
    },
    constants::*,
    engine::state::{EngineState, RunState},
    inventory::{Inventory, InventoryAction},
    utils::assets::TextureName,
};

pub struct Draw {}

impl Draw {
    pub fn render_game(game_state: &EngineState, assets: &HashMap<TextureName, Texture2D>) {
        let mut maps = game_state.ecs_world.query::<&Map>();
        match game_state.run_state {
            RunState::GameOver => Draw::game_over(),
            _ => {
                for (_e, map) in &mut maps {
                    map.draw(assets);
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
            (MAP_HEIGHT * TILE_SIZE) as f32,
            HUD_WIDTH as f32,
            HUD_HEIGHT as f32,
            WHITE,
        );
        draw_rectangle(
            (HUD_BORDER + UI_BORDER) as f32,
            (HUD_BORDER + MAP_HEIGHT * TILE_SIZE) as f32,
            (HUD_WIDTH - UI_BORDER) as f32,
            (HUD_HEIGHT - UI_BORDER) as f32,
            BLACK,
        );

        // ------- Stat Text header -----------
        draw_rectangle(
            (HEADER_LEFT_SPAN + HUD_BORDER) as f32,
            (MAP_HEIGHT * TILE_SIZE - UI_BORDER) as f32,
            HEADER_WIDTH as f32,
            HEADER_HEIGHT as f32,
            BLACK,
        );

        // ------- Stat Text  -----------

        let mut player_query = ecs_world.query::<(&Player, &CombatStats)>();
        let (_e, (_p, player_stats)) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");
        let mut text_color = WHITE;

        // Draw Stamina (STA)
        if player_stats.current_stamina == 0 {
            text_color = RED;
        } else if player_stats.current_stamina <= player_stats.max_stamina / 2 {
            text_color = YELLOW;
        }

        Self::draw_stat_text(
            format!(
                "STA: {} / {}",
                player_stats.current_stamina, player_stats.max_stamina
            ),
            0,
            text_color,
        );

        // Draw Toughness (TOU)
        text_color = WHITE;

        if player_stats.current_toughness < player_stats.max_toughness {
            text_color = YELLOW;
        }

        Self::draw_stat_text(
            format!(
                "TOU {} / {}",
                player_stats.current_toughness, player_stats.max_toughness
            ),
            160,
            text_color,
        );

        // Draw Dexterity (DEX)
        text_color = WHITE;

        if player_stats.current_dexterity < player_stats.max_dexterity {
            text_color = YELLOW;
        }

        Self::draw_stat_text(
            format!(
                "DEX {} / {}",
                player_stats.current_dexterity, player_stats.max_dexterity
            ),
            330,
            text_color,
        );

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
                    + (UI_BORDER * 2)
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

    fn draw_stat_text(text: String, left_pad: i32, text_color: Color) {
        draw_text(
            text,
            (HUD_BORDER + HEADER_LEFT_SPAN + UI_BORDER + left_pad) as f32,
            (HUD_BORDER + UI_BORDER + MAP_HEIGHT * TILE_SIZE) as f32,
            FONT_SIZE,
            text_color,
        );
    }

    /// Draw all Renderable entities in World
    fn renderables(world: &World, assets: &HashMap<TextureName, Texture2D>, map: &Map) {
        //Get all entities in readonly
        let mut renderables_with_position = world.query::<(&Renderable, &Position)>();

        for (_entity, (renderable, position)) in &mut renderables_with_position {
            let texture_to_render = assets
                .get(&renderable.texture_name)
                .expect("Texture not found");

            if map.visible_tiles[get_index_from_xy(position.x, position.y)] {
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
        let (mouse_x, mouse_y) = mouse_position();

        let mut map_query = ecs_world.query::<&Map>();
        let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

        let rounded_x = (((mouse_x - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;
        let rounded_y = (((mouse_y - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;

        // TODO show something indicating mouse usability
        // Draw target if tile is visible
        let index = get_index_from_xy(rounded_x, rounded_y);
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
}
