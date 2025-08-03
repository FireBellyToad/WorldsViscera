use std::collections::HashMap;

use hecs::World;
use macroquad::{
    color::{BLACK, Color, DARKGRAY, ORANGE, RED, WHITE, YELLOW},
    input::mouse_position,
    math::Rect,
    shapes::{draw_circle, draw_rectangle, draw_rectangle_lines},
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    components::{
        combat::CombatStats,
        common::{CanSmell, GameLog, Position, Renderable, SmellIntensity, Smellable},
        health::{Hunger, Thirst},
        player::{Player, SpecialViewMode},
    },
    constants::*,
    engine::state::{EngineState, RunState},
    inventory::Inventory,
    maps::zone::{ParticleType, Zone},
    systems::{hunger_check::HungerStatus, thirst_check::ThirstStatus},
    utils::{assets::TextureName, common::Utils, particle_animation::ParticleAnimation},
};

pub struct Draw {}

impl Draw {
    pub fn render_game(game_state: &EngineState, assets: &HashMap<TextureName, Texture2D>) {
        let mut zones = game_state.ecs_world.query::<&Zone>();
        match game_state.run_state {
            RunState::GameOver => Draw::game_over(),
            _ => {
                // Zone and renderables
                for (_e, zone) in &mut zones {
                    Draw::zone(&zone, assets);
                    Draw::renderables(&game_state.ecs_world, &assets, &zone);
                }

                //Overlay
                match &game_state.run_state {
                    RunState::ShowInventory(mode) => {
                        Inventory::draw(assets, &game_state.ecs_world, mode)
                    }
                    RunState::MouseTargeting(special_view_mode) => {
                        Draw::targeting(&game_state.ecs_world, &assets, special_view_mode);
                    }
                    RunState::DrawParticles => {
                        let mut animations = game_state.ecs_world.query::<&mut ParticleAnimation>();
                        for a in &mut animations {
                            Draw::particles(a.1, assets);
                        }
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

    fn hud_header(ecs_world: &World) {
        let mut zones = ecs_world.query::<&Zone>();
        let (_e, zone) = zones.iter().last().expect("Zone is not in hecs::World");

        let mut player_query = ecs_world.query::<(&Player, &CombatStats, &Hunger, &Thirst)>();
        let (_e, (_p, player_stats, hunger, thirst)) = player_query
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
        let hunger_status = &hunger.current_status;
        let hunger_text = format!("Hunger:{:?}", hunger_status);
        let hunger_text_len = hunger_text.len();

        let thirst_status = &thirst.current_status;
        let thirst_text = format!("Thirst:{:?}", thirst_status);
        let thirst_text_len = thirst_text.len();

        let dex_text_len = dex_text.len();
        let depth_text = format!("Depth: {}", zone.depth);
        let depth_text_len = depth_text.len();

        draw_rectangle(
            (HEADER_LEFT_SPAN + HUD_BORDER) as f32,
            (MAP_HEIGHT * TILE_SIZE) as f32 + UI_BORDER as f32,
            6.0 * LETTER_SIZE - HUD_BORDER as f32 * 3.0
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE)
                + (dex_text_len as f32 * LETTER_SIZE)
                + (hunger_text_len as f32 * LETTER_SIZE)
                + (thirst_text_len as f32 * LETTER_SIZE)
                + (depth_text_len as f32 * LETTER_SIZE),
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

        Draw::stat_text(sta_text, 0.0, text_color);

        // Draw Toughness (TOU)
        text_color = WHITE;

        if player_stats.current_toughness < player_stats.max_toughness {
            text_color = YELLOW;
        }

        Draw::stat_text(
            tou_text,
            LETTER_SIZE + (sta_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // Draw Dexterity (DEX)
        text_color = WHITE;

        if player_stats.current_dexterity < player_stats.max_dexterity {
            text_color = YELLOW;
        }

        Draw::stat_text(
            dex_text,
            2.0 * LETTER_SIZE
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // TODO improve
        match hunger_status {
            HungerStatus::Hungry => text_color = YELLOW,
            HungerStatus::Starved => text_color = RED,
            _ => text_color = WHITE,
        }
        Draw::stat_text(
            hunger_text,
            3.0 * LETTER_SIZE
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE)
                + (dex_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // TODO improve
        match thirst_status {
            ThirstStatus::Thirsty => text_color = YELLOW,
            ThirstStatus::Dehydrated => text_color = RED,
            _ => text_color = WHITE,
        }
        Draw::stat_text(
            thirst_text,
            4.0 * LETTER_SIZE
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE)
                + (dex_text_len as f32 * LETTER_SIZE)
                + (hunger_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        text_color = WHITE;
        // TODO improve
        Draw::stat_text(
            depth_text,
            5.0 * LETTER_SIZE
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE)
                + (dex_text_len as f32 * LETTER_SIZE)
                + (hunger_text_len as f32 * LETTER_SIZE)
                + (thirst_text_len as f32 * LETTER_SIZE),
            text_color,
        );
    }

    fn stat_text(text: String, left_pad: f32, text_color: Color) {
        draw_text(
            text,
            (HUD_BORDER + HEADER_LEFT_SPAN + UI_BORDER) as f32 + left_pad,
            (HUD_BORDER + UI_BORDER * 3 + MAP_HEIGHT * TILE_SIZE) as f32,
            FONT_SIZE,
            text_color,
        );
    }

    /// Draw all Renderable entities in World
    fn renderables(world: &World, assets: &HashMap<TextureName, Texture2D>, zone: &Zone) {
        //Get all entities in readonly
        let mut renderables_with_position = world.query::<(&Renderable, &Position)>();

        let mut renderables_vec: Vec<(hecs::Entity, (&Renderable, &Position))> =
            renderables_with_position.iter().collect();
        renderables_vec.sort_by_key(|(_e, (renderable, _p))| renderable.z_index);

        for (_e, (renderable, position)) in renderables_vec {
            let texture_to_render = assets
                .get(&renderable.texture_name)
                .expect("Texture not found");

            if zone.visible_tiles[Zone::get_index_from_xy(position.x, position.y)] {
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
    fn targeting(
        ecs_world: &World,
        assets: &HashMap<TextureName, Texture2D>,
        special_view_mode: &SpecialViewMode,
    ) {
        draw_text(
            "Use mouse to select, ESC to cancel",
            24.0,
            48.0,
            FONT_SIZE,
            WHITE,
        );

        let mut zone_query = ecs_world.query::<&Zone>();
        let (_e, zone) = zone_query
            .iter()
            .last()
            .expect("Zone is not in hecs::World");

        let only_on_visible_tiles = *special_view_mode == SpecialViewMode::ZapTargeting;
        Draw::special_targets(ecs_world, assets, special_view_mode, zone);

        let (mouse_x, mouse_y) = mouse_position();

        let rounded_x = (((mouse_x - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;
        let rounded_y = (((mouse_y - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;

        // Draw target if tile is visible
        let index = Zone::get_index_from_xy(rounded_x, rounded_y);
        if !only_on_visible_tiles || zone.visible_tiles.len() > index && zone.visible_tiles[index] {
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

    /// Draws special targets when in targeting mode
    fn special_targets(
        ecs_world: &World,
        assets: &HashMap<TextureName, Texture2D>,
        special_view_mode: &SpecialViewMode,
        zone: &Zone,
    ) {
        let player_entity = Player::get_entity(ecs_world);
        let player_position = ecs_world.get::<&Position>(player_entity).unwrap();
        let player_smell_ability = ecs_world.get::<&CanSmell>(player_entity).unwrap();

        // Draw targets if special
        match special_view_mode {
            SpecialViewMode::Smell => {
                //Show smellable on not visibile tiles
                let mut smells_with_position = ecs_world.query::<(&Position, &Smellable)>();
                for (_e, (smell_position, smell)) in &mut smells_with_position {
                    let index = Zone::get_index_from_xy(smell_position.x, smell_position.y);

                    let distance = Utils::distance(
                        smell_position.x,
                        player_position.x,
                        smell_position.y,
                        player_position.y,
                    );

                    let can_smell = player_smell_ability.intensity != SmellIntensity::None // the player cannot smell anything (common cold or other penalities)
                        && !zone.visible_tiles[index]
                        && ((distance < PLAYER_SMELL_RADIUS / 2.0 && smell.intensity == SmellIntensity::Faint) // Faint odors can be smell from half normal distance
                            || (distance < PLAYER_SMELL_RADIUS
                                && (smell.intensity == SmellIntensity::Strong // Strong odors can be smelled at double distance. 
                                    || player_smell_ability.intensity == SmellIntensity::Strong))); // Player have improved smell (can smell faint odors from far away)

                    //draw not visible smellables within smell radius
                    if can_smell {
                        let texture_to_render = assets
                            .get(&TextureName::Particles)
                            .expect("Texture not found");

                        draw_texture_ex(
                            texture_to_render,
                            (UI_BORDER + (smell_position.x * TILE_SIZE)) as f32,
                            (UI_BORDER + (smell_position.y * TILE_SIZE)) as f32,
                            WHITE, // Seems like White color is needed to normal render
                            DrawTextureParams {
                                source: Some(Rect {
                                    x: 4.0 * TILE_SIZE_F32,
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
            _ => {}
        }
    }

    /// Draws zone
    pub fn zone(zone: &Zone, assets: &HashMap<TextureName, Texture2D>) {
        let texture_to_render = assets.get(&TextureName::Tiles).expect("Texture not found");

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile_to_draw = Zone::get_index_from_xy(x, y);
                let tile_index =
                    Zone::get_tile_sprite_sheet_index(&zone.tiles[tile_to_draw]) * TILE_SIZE_F32;

                if zone.revealed_tiles[tile_to_draw] {
                    let mut alpha = DARKGRAY;

                    if zone.visible_tiles[tile_to_draw] {
                        alpha = WHITE;
                        if zone.particle_tiles.contains_key(&tile_to_draw) {
                            Draw::draw_particles(
                                x,
                                y,
                                zone.particle_tiles.get(&tile_to_draw).unwrap(),
                            );
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
    pub fn draw_particles(x: i32, y: i32, particle_type: &ParticleType) {
        let color;

        match particle_type {
            ParticleType::Blood => color = Color::from_rgba(255, 10, 10, 32),
            ParticleType::Vomit => color = ORANGE,
        }

        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 - 6.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 - 7.0,
            2.0,
            color,
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 + 4.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 - 4.0,
            2.0,
            color,
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 - 5.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 + 4.0,
            1.0,
            color,
        );
        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32 + 3.0,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32 + 5.0,
            2.0,
            color,
        );

        draw_circle(
            (UI_BORDER + (x * TILE_SIZE) + TILE_SIZE / 2) as f32,
            (UI_BORDER + (y * TILE_SIZE) + TILE_SIZE / 2) as f32,
            4.0,
            color,
        );
    }

    /// Draw particles
    pub fn particles(animation: &mut ParticleAnimation, assets: &HashMap<TextureName, Texture2D>) {
        if animation.current_frame < animation.frames.len() {
            let texture_to_render = assets
                .get(&TextureName::Particles)
                .expect("Texture not found");
            let frame_to_render = &animation.frames[animation.current_frame];

            //Render different directions for particles
            let mut direction = 0.0;
            let (mut previous_x, mut previous_y) = (-1, -1);
            for (x, y) in frame_to_render {
                // First particle must not be rendered
                if previous_x != -1 && previous_y != 1 {
                    if previous_y == *y {
                        direction = 0.0;
                    } else if previous_x == *x {
                        direction = 1.0;
                    } else if previous_y > *y {
                        if previous_x < *x {
                            direction = 2.0;
                        } else {
                            direction = 3.0;
                        }
                    } else if previous_y < *y {
                        if previous_x > *x {
                            direction = 2.0;
                        } else {
                            direction = 3.0;
                        }
                    }

                    // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                    draw_texture_ex(
                        texture_to_render,
                        (UI_BORDER + (x * TILE_SIZE)) as f32,
                        (UI_BORDER + (y * TILE_SIZE)) as f32,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect {
                                x: direction * TILE_SIZE_F32,
                                y: 0.0,
                                w: TILE_SIZE_F32,
                                h: TILE_SIZE_F32,
                            }),
                            ..Default::default()
                        },
                    );
                }

                previous_x = *x;
                previous_y = *y;
            }
        }
    }
}
