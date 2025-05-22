use std::{collections::HashMap, thread::sleep, time::Duration};

use assets::TextureName;
use components::{
    common::{Position, Renderable},
    player::{Player, player_input},
};
use constants::*;
use engine::state::EngineState;
use hecs::{EntityBuilder, World};
use macroquad::prelude::*;
use map::Map;

mod assets;
mod components;
mod constants;
mod engine;
mod map;

//Game configuration
fn get_game_configuration() -> Conf {
    Conf {
        window_title: String::from("World's Viscera"),
        fullscreen: false,
        window_height: WINDOW_HEIGHT,
        window_width: WINDOW_WIDTH,
        window_resizable: false,
        //use the default options:
        ..Default::default()
    }
}

#[macroquad::main(get_game_configuration)]
async fn main() {
    //Load resources inside map
    let mut assets = HashMap::new();
    assets.insert(
        TextureName::Creatures,
        load_texture("assets/creatures.png").await.unwrap(),
    );
    assets.insert(
        TextureName::Tiles,
        load_texture("assets/tiles.png").await.unwrap(),
    );

    //Init ECS
    let game_state = EngineState {
        ecs_world: create_ecs_world(),
    };

    let mut maps = game_state.ecs_world.query::<&Map>();
    let mut time: f32 = 0.0;
    loop {
        time = do_game_tick(time, &game_state);

        if is_tick_done(time){
            next_frame().await;
        }

        for (_entity, map) in &mut maps {
            map.draw_map(&assets);
        }

        draw_renderables(&game_state.ecs_world, &assets);
    }
}

fn do_game_tick(current_time: f32, game_state: &EngineState) -> f32 {
    let time_passed = current_time + get_frame_time();

    // needed for the engine
    if time_passed > SECONDS_TO_WAIT {
        do_game_logic(game_state);
        return 0.0;
    }

    time_passed
}

fn is_tick_done(current_time: f32) -> bool {
  current_time == 0.0
}

fn do_game_logic(game_state: &EngineState) {
    player_input(&game_state.ecs_world);
}

fn create_ecs_world() -> World {
    let mut world = World::new();
    let mut builder = EntityBuilder::new();
    let player_entity = builder
        .add(Player {})
        .add(Position {
            x: MAP_WIDTH / 2,
            y: MAP_HEIGHT / 2,
        })
        .add(Renderable {
            texture_name: TextureName::Creatures,
            texture_region: Rect {
                x: 0.0,
                y: 0.0,
                w: TILE_SIZE as f32,
                h: TILE_SIZE as f32,
            },
        })
        .build();

    world.spawn(player_entity);

    let map_entity = builder.add(Map::new()).build();
    world.spawn(map_entity);

    world
}

fn draw_renderables(world: &World, assets: &HashMap<TextureName, Texture2D>) {
    //Get all entities in readonly
    let mut renderables_with_position = world.query::<(&Renderable, &Position)>();

    for (_entity, (renderable, position)) in &mut renderables_with_position {
        let texture_to_render = assets
            .get(&renderable.texture_name)
            .expect("Texture not found");

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
