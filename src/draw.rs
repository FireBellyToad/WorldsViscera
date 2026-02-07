use std::collections::HashMap;

use hecs::World;
use macroquad::{
    color::{BLACK, Color, DARKGRAY, GRAY, GREEN, ORANGE, RED, WHITE, YELLOW},
    input::mouse_position,
    math::Rect,
    shapes::{draw_circle, draw_rectangle, draw_rectangle_lines},
    text::draw_text,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::{
    components::{
        combat::{CombatStats, IsHidden},
        common::{CanSmell, Experience, GameLog, Position, Renderable, SmellIntensity, Smellable},
        health::{Hunger, Thirst},
        player::{Player, SpecialViewMode},
    },
    constants::*,
    dialog::Dialog,
    engine::state::{EngineState, RunState},
    inventory::Inventory,
    maps::zone::{DecalType, TileType, Zone},
    systems::{hunger_check::HungerStatus, thirst_check::ThirstStatus},
    utils::{
        assets::TextureName,
        common::Utils,
        particle_animation::{ParticleAnimation, ParticleAnimationType},
    },
};

pub struct Draw {}

impl Draw {
    pub fn render_game(game_state: &EngineState, assets: &HashMap<TextureName, Texture2D>) {
        let mut zones = game_state.ecs_world.query::<&Zone>();
        match game_state.run_state {
            RunState::GameOver => Draw::game_over(),
            RunState::TitleScreen => Draw::title_screen(),
            _ => {
                // Zone and renderables
                for (_, zone) in &mut zones {
                    Draw::zone(zone, assets);
                    Draw::renderables(&game_state.ecs_world, assets, zone);
                    Draw::smells(&game_state.ecs_world, assets, zone);

                    #[cfg(not(target_arch = "wasm32"))]
                    Draw::debug_exit(zone);
                }

                //Overlay
                match &game_state.run_state {
                    RunState::ShowInventory(mode) => {
                        Inventory::draw(assets, &game_state.ecs_world, mode)
                    }
                    RunState::ShowDialog(mode) => Dialog::draw(assets, &game_state.ecs_world, mode),
                    RunState::MouseTargeting(special_view_mode) => {
                        Draw::targeting(&game_state.ecs_world, special_view_mode);
                    }
                    RunState::DrawParticles => {
                        let mut animations = game_state.ecs_world.query::<&mut ParticleAnimation>();
                        for a in &mut animations {
                            // For each zone, draw particles. Usually is only one zone!
                            for (_, zone) in &mut zones {
                                Draw::particles(a.1, assets, zone);
                            }
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
        let (_, game_log) = game_log_query
            .iter()
            .last()
            .expect("Game log is not in hecs::World");

        // Going backwards to get last message on top
        for (index, message) in game_log.entries.iter().rev().enumerate() {
            let draw_index = game_log.entries.len() - index;
            draw_text(
                format!("{draw_index} - {message}"),
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
        let (_, zone) = zones.iter().last().expect("Zone is not in hecs::World");

        let mut player_query = ecs_world
            .query::<(&Experience, &CombatStats, &Hunger, &Thirst)>()
            .with::<&Player>();
        let (_, (experience, player_stats, hunger, thirst)) = player_query
            .iter()
            .last()
            .expect("Player is not in hecs::World");

        let level_text = format!("LVL:{}", player_stats.level);
        let level_text_len = level_text.len();

        let exp_text = format!("EXP:{}", experience.value);
        let exp_text_len = exp_text.len();

        let sta_text = format!(
            "STA:{}/{}",
            player_stats.current_stamina, player_stats.max_stamina
        );
        let sta_text_len = sta_text.len();

        let tou_text = format!(
            "TOU:{}/{}",
            player_stats.current_toughness, player_stats.max_toughness
        );
        let tou_text_len = tou_text.len();

        let dex_text = format!(
            "DEX:{}/{}",
            player_stats.current_dexterity, player_stats.max_dexterity
        );
        let hunger_status = &hunger.current_status;
        let hunger_text = format!("HUN:{}", hunger_status.to_string());
        let hunger_text_len = hunger_text.len();

        let thirst_status = &thirst.current_status;
        let thirst_text = format!("THI:{}", thirst_status.to_string());
        let thirst_text_len = thirst_text.len();

        let dex_text_len = dex_text.len();
        let depth_text = format!("Depth:{}", zone.depth);
        let depth_text_len = depth_text.len();

        draw_rectangle(
            (HEADER_LEFT_SPAN + HUD_BORDER) as f32,
            (MAP_HEIGHT * TILE_SIZE) as f32 + UI_BORDER as f32,
            8.0 * LETTER_SIZE - HUD_BORDER as f32 * 2.0
                + (level_text_len as f32 * LETTER_SIZE)
                + (exp_text_len as f32 * LETTER_SIZE)
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

        // Draw Level (LVL)
        Draw::stat_text(level_text, 0.0, text_color);

        // Draw Experience (EXP)
        Draw::stat_text(
            exp_text,
            LETTER_SIZE + (level_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // Draw Stamina (STA)
        if player_stats.current_stamina == 0 {
            text_color = RED;
        } else if player_stats.current_stamina <= player_stats.max_stamina / 2 {
            text_color = YELLOW;
        }

        Draw::stat_text(
            sta_text,
            2.0 * LETTER_SIZE
                + (level_text_len as f32 * LETTER_SIZE)
                + (exp_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // Draw Toughness (TOU)
        text_color = WHITE;

        if player_stats.current_toughness < player_stats.max_toughness {
            text_color = YELLOW;
        }

        Draw::stat_text(
            tou_text,
            3.0 * LETTER_SIZE
                + (level_text_len as f32 * LETTER_SIZE)
                + (exp_text_len as f32 * LETTER_SIZE)
                + (sta_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // Draw Dexterity (DEX)
        text_color = WHITE;

        if player_stats.current_dexterity < player_stats.max_dexterity {
            text_color = YELLOW;
        }

        Draw::stat_text(
            dex_text,
            4.0 * LETTER_SIZE
                + (level_text_len as f32 * LETTER_SIZE)
                + (exp_text_len as f32 * LETTER_SIZE)
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // TODO improve
        match hunger_status {
            HungerStatus::Satiated => text_color = GREEN,
            HungerStatus::Hungry => text_color = YELLOW,
            HungerStatus::Starved => text_color = RED,
            _ => text_color = WHITE,
        }
        Draw::stat_text(
            hunger_text,
            5.0 * LETTER_SIZE
                + (level_text_len as f32 * LETTER_SIZE)
                + (exp_text_len as f32 * LETTER_SIZE)
                + (sta_text_len as f32 * LETTER_SIZE)
                + (tou_text_len as f32 * LETTER_SIZE)
                + (dex_text_len as f32 * LETTER_SIZE),
            text_color,
        );

        // TODO improve
        match thirst_status {
            ThirstStatus::Quenched => text_color = GREEN,
            ThirstStatus::Thirsty => text_color = YELLOW,
            ThirstStatus::Dehydrated => text_color = RED,
            _ => text_color = WHITE,
        }
        Draw::stat_text(
            thirst_text,
            6.0 * LETTER_SIZE
                + (level_text_len as f32 * LETTER_SIZE)
                + (exp_text_len as f32 * LETTER_SIZE)
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
            7.0 * LETTER_SIZE
                + (level_text_len as f32 * LETTER_SIZE)
                + (exp_text_len as f32 * LETTER_SIZE)
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
    fn renderables(ecs_world: &World, assets: &HashMap<TextureName, Texture2D>, zone: &Zone) {
        //Get all entities in readonly
        let mut renderables_with_position = ecs_world
            .query::<(&Renderable, &Position)>()
            .without::<&IsHidden>();

        let mut renderables_vec: Vec<(hecs::Entity, (&Renderable, &Position))> =
            renderables_with_position.iter().collect();
        renderables_vec.sort_by_key(|(_, (renderable, _))| renderable.z_index);

        for (_, (renderable, position)) in renderables_vec {
            let texture_to_render = assets
                .get(&renderable.texture_name)
                .expect("Texture not found");

            if zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)] {
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

    /// Draw title game screen
    fn title_screen() {
        let title = "WORLD'S VISCERA";
        let command = "Press any key to start, Q to exit";
        draw_rectangle(0.0, 0.0, 64.0, 32.0, BLACK);
        draw_text(
            title,
            (WINDOW_WIDTH / 2) as f32 - ((title.len() as f32 / 2.0) * FONT_SIZE),
            64.0,
            FONT_SIZE * 2.0,
            WHITE,
        );
        draw_text(
            command,
            (WINDOW_WIDTH / 2) as f32 - ((command.len() as f32 / 2.0) * FONT_SIZE / 2.0),
            96.0,
            FONT_SIZE,
            WHITE,
        );
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
    fn targeting(ecs_world: &World, special_view_mode: &SpecialViewMode) {
        draw_text(
            "Use mouse to select, ESC to cancel",
            24.0,
            48.0,
            FONT_SIZE,
            WHITE,
        );

        let mut zone_query = ecs_world.query::<&Zone>();
        let (_, zone) = zone_query
            .iter()
            .last()
            .expect("Zone is not in hecs::World");

        let (mouse_x, mouse_y) = mouse_position();

        let rounded_x = (((mouse_x - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;
        let rounded_y = (((mouse_y - UI_BORDER_F32) / TILE_SIZE_F32).ceil() - 1.0) as i32;

        // Draw target if tile is visible
        let index = Zone::get_index_from_xy(&rounded_x, &rounded_y);
        if special_view_mode == &SpecialViewMode::Smell
            || (zone.visible_tiles.len() > index && zone.visible_tiles[index])
        {
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

    /// Draws smells
    fn smells(ecs_world: &World, assets: &HashMap<TextureName, Texture2D>, zone: &Zone) {
        let mut player_query_smell = ecs_world
            .query::<(&Position, &CanSmell)>()
            .with::<&Player>();

        for (_, (player_position, player_smell_ability)) in &mut player_query_smell {
            //Show smellable on not visibile tiles
            let mut smells_with_position = ecs_world.query::<(&Position, &Smellable)>();
            for (_, (smell_position, smell)) in &mut smells_with_position {
                let index = Zone::get_index_from_xy(&smell_position.x, &smell_position.y);

                let distance = Utils::distance(
                    &smell_position.x,
                    &player_position.x,
                    &smell_position.y,
                    &player_position.y,
                );

                let can_smell = player_smell_ability.intensity != SmellIntensity::None // the player cannot smell anything (common cold or other penalities)
                        && !zone.visible_tiles[index]
                        && ((distance < player_smell_ability.radius / 2.0 && smell.intensity == SmellIntensity::Faint) // Faint odors can be smell from half normal distance
                            || (distance < player_smell_ability.radius
                                && (smell.intensity == SmellIntensity::Strong // Strong odors can be smelled at double distance.
                                    || player_smell_ability.intensity == SmellIntensity::Strong))); // Player have improved smell (can smell faint odors from far away)

                //draw not visible smellables within smell radius
                if can_smell {
                    let texture_to_render = assets
                        .get(&TextureName::Particles)
                        .expect("Texture not found");

                    let mut index = 0.0;
                    if smell.intensity == SmellIntensity::Strong {
                        index += 1.0;
                    }

                    draw_texture_ex(
                        texture_to_render,
                        (UI_BORDER + (smell_position.x * TILE_SIZE)) as f32,
                        (UI_BORDER + (smell_position.y * TILE_SIZE)) as f32,
                        WHITE, // Seems like White color is needed to normal render
                        DrawTextureParams {
                            source: Some(Rect {
                                x: (index) * TILE_SIZE_F32,
                                y: SMELL_PARTICLE_TYPE as f32 * TILE_SIZE_F32,
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

    /// Draws zone
    pub fn zone(zone: &Zone, assets: &HashMap<TextureName, Texture2D>) {
        let texture_to_render = assets.get(&TextureName::Tiles).expect("Texture not found");

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile_to_draw = Zone::get_index_from_xy(&x, &y);
                let tile_index = Zone::get_tile_sprite_sheet_index(&zone.tiles[tile_to_draw]);

                if zone.revealed_tiles[tile_to_draw] {
                    let mut alpha = DARKGRAY;

                    if zone.visible_tiles[tile_to_draw] {
                        alpha = WHITE;
                    }

                    // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                    draw_texture_ex(
                        texture_to_render,
                        (UI_BORDER + (x * TILE_SIZE)) as f32,
                        (UI_BORDER + (y * TILE_SIZE)) as f32,
                        alpha,
                        DrawTextureParams {
                            source: Some(Rect {
                                x: tile_index.0 * TILE_SIZE_F32,
                                y: tile_index.1 * TILE_SIZE_F32,
                                w: TILE_SIZE_F32,
                                h: TILE_SIZE_F32,
                            }),
                            ..Default::default()
                        },
                    );

                    // Decals must be drawn on top of NON water tiles
                    if zone.visible_tiles[tile_to_draw]
                        && !zone.water_tiles[tile_to_draw]
                        && zone.decals_tiles.contains_key(&tile_to_draw)
                    {
                        Draw::draw_decals(
                            &x,
                            &y,
                            zone.decals_tiles
                                .get(&tile_to_draw)
                                .expect("decals_tiles not available"),
                        );
                    }
                }
            }
        }
    }

    /// Utility for drawing blood blots
    pub fn draw_decals(x: &i32, y: &i32, particle_type: &DecalType) {
        let color = match particle_type {
            DecalType::Blood => RED,
            DecalType::Vomit => ORANGE,
            DecalType::Slime => WHITE,
            DecalType::Acid => GREEN,
            DecalType::Filth => GRAY,
        };

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
    pub fn particles(
        animation: &mut ParticleAnimation,
        assets: &HashMap<TextureName, Texture2D>,
        zone: &Zone,
    ) {
        if animation.current_frame < animation.frames.len() {
            let texture_to_render = assets
                .get(&TextureName::Particles)
                .expect("Texture not found");
            let frame_to_render = &animation.frames[animation.current_frame];

            //Render different directions for particles
            let mut direction = LEFT_DIR;
            let (mut previous_x, mut previous_y) = (-1, -1);

            match animation.animation_type {
                ParticleAnimationType::Frame => {
                    // If frame, usually show only the "down" sprite
                    direction = DOWN_DIR;
                }
                ParticleAnimationType::Projectile => {
                    // If projectile, get direction from the first two frames
                    // TODO use atan and 30° degrees increments for better graphics
                    let first_frame = &animation.frames[0];
                    let last_frame = &animation.frames[&animation.frames.len() - 1];
                    (previous_x, previous_y) = first_frame[0];
                    let (x, y) = last_frame[0];
                    direction = Draw::get_direction_from_angle(&previous_x, &previous_y, &x, &y);
                }
                ParticleAnimationType::Ray => {
                    let previous_frame = &animation.frames[animation.current_frame - 1];
                    (previous_x, previous_y) = previous_frame[0];
                }
            }

            for (subframe_idx, (x, y)) in frame_to_render.iter().enumerate() {
                //Render different directions for particles of a single frame (rays, explosions...)
                if animation.animation_type == ParticleAnimationType::Ray {
                    direction = Draw::get_direction_from_two_points(&previous_x, &previous_y, x, y);
                }
                // If the previous position is the starting one and the animation should exclude the first subframe, skip drawing
                // This is important for the rays animations, avoiding overlap of the first subframe with the origin
                if zone.visible_tiles[Zone::get_index_from_xy(x, y)]
                    && (animation.animation_type != ParticleAnimationType::Ray || subframe_idx > 0)
                {
                    // Take the texture and draw only the wanted tile ( DrawTextureParams.source )
                    draw_texture_ex(
                        texture_to_render,
                        (UI_BORDER + (x * TILE_SIZE)) as f32,
                        (UI_BORDER + (y * TILE_SIZE)) as f32,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect {
                                x: direction * TILE_SIZE_F32,
                                y: animation.particle_type as f32 * TILE_SIZE_F32,
                                w: TILE_SIZE_F32,
                                h: TILE_SIZE_F32,
                            }),
                            ..Default::default()
                        },
                    );
                }

                // For everything that has a subframe, get the previous position
                previous_x = *x;
                previous_y = *y;
            }
        }
    }

    /// Calculate the direction from two points (used for particle animations)
    /// Simpler and faster than get_direction_from_angle, but much less accurate
    fn get_direction_from_two_points(ax: &i32, ay: &i32, bx: &i32, by: &i32) -> f32 {
        let mut direction = LEFT_DIR;
        if *ay == *by {
            if *ax < *bx {
                direction = RIGHT_DIR;
            } else {
                direction = LEFT_DIR;
            }
        } else if *ax == *bx {
            if *ay < *by {
                direction = DOWN_DIR;
            } else {
                direction = UP_DIR;
            }
        } else if *ay < *by {
            if *ax < *bx {
                direction = DOWN_RIGHT_DIR;
            } else {
                direction = DOWN_LEFT_DIR;
            }
        } else if *ay > *by {
            if *ax > *bx {
                direction = UP_LEFT_DIR;
            } else {
                direction = UP_RIGHT_DIR;
            }
        }

        direction
    }

    /// Calculate the direction using the angle from two points (used for particle animations)
    /// Uses atan2 and radians to degrees coversion, accurate but slower than get_direction_from_two_points
    fn get_direction_from_angle(ax: &i32, ay: &i32, bx: &i32, by: &i32) -> f32 {
        let mut direction = LEFT_DIR;

        let angle_rad = f64::atan2((by - ay) as f64, (bx - ax) as f64);
        let angle_deg = angle_rad.to_degrees() + 180.0;

        if (150.0..=210.0).contains(&angle_deg) {
            direction = RIGHT_DIR;
        } else if ((330.0..=360.0).contains(&angle_deg)) || ((0.0..=30.0).contains(&angle_deg)) {
            direction = LEFT_DIR;
        } else if (60.0..=120.0).contains(&angle_deg) {
            direction = UP_DIR;
        } else if (240.0..=300.0).contains(&angle_deg) {
            direction = DOWN_DIR;
        } else if angle_deg > 30.0 && angle_deg < 60.0 {
            direction = UP_LEFT_DIR;
        } else if angle_deg > 120.0 && angle_deg < 150.0 {
            direction = UP_RIGHT_DIR;
        } else if angle_deg > 300.0 && angle_deg < 330.0 {
            direction = DOWN_LEFT_DIR;
        } else if angle_deg > 210.0 && angle_deg < 240.0 {
            direction = DOWN_RIGHT_DIR;
        }

        direction
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn debug_exit(zone: &Zone) {
        let start_index = zone
            .tiles
            .iter()
            .position(|tile| tile == &TileType::DownPassage)
            .or(None);

        // Draw a red rectangle on the DownPassage tile index, if present
        if let Some(start_index) = start_index {
            let (rounded_x, rounded_y) = Zone::get_xy_from_index(start_index);

            draw_rectangle(
                (UI_BORDER + (rounded_x * TILE_SIZE)) as f32,
                (UI_BORDER + (rounded_y * TILE_SIZE)) as f32,
                TILE_SIZE_F32,
                TILE_SIZE_F32,
                RED,
            );
        }
    }
}
