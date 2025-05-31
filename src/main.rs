use std::collections::HashMap;

use assets::TextureName;
use components::{
    combat::{CombatStats, Damageable},
    common::{BlocksTile, GameLog, Named, Position, Renderable, Viewshed},
    monster::Monster,
    player::{Player, VIEW_RADIUS, player_input},
};
use constants::*;
use draw::Draw;
use engine::{
    gameengine::GameEngine,
    state::{EngineState, RunState},
};
use hecs::{EntityBuilder, World};
use macroquad::prelude::*;
use map::Map;
use systems::{
    damage_manager::DamageManager, fov::FovSystem, map_indexing::MapIndexing, monster_ai::MonsterAI,
};

mod assets;
mod components;
mod constants;
mod draw;
mod engine;
mod map;
mod systems;
mod utils;

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
    let mut game_engine = GameEngine::new();
    let mut game_state = EngineState {
        ecs_world: create_ecs_world(),
        run_state: RunState::SystemsRunning,
    };

    loop {
        if game_engine.next_tick() {
            // Run system only while not paused, or else wait for player input.
            // Make the whole game turn based

            match game_state.run_state {
                RunState::SystemsRunning => {
                    game_state.run_state =
                        do_game_logic(&mut game_state, RunState::WaitingPlayerInput);
                }
                RunState::WaitingPlayerInput => {
                    game_state.run_state = player_input(&game_state.ecs_world)
                }
                RunState::PlayerTurn => {
                    game_state.run_state = do_game_logic(&mut game_state, RunState::MonsterTurn);
                }
                RunState::MonsterTurn => {
                    MonsterAI::act(&game_state.ecs_world);
                    game_state.run_state = do_game_logic(&mut game_state, RunState::SystemsRunning);
                }
                RunState::GameOver => {
                    // Quit game on Q
                    if is_key_pressed(KeyCode::Q) {
                        break;
                    }
                }
            }

            next_frame().await;
        }

        Draw::render_game(&game_state, &assets);
    }
}

fn create_ecs_world() -> World {
    let mut world = World::new();
    let mut builder = EntityBuilder::new();

    //Add Game log to world
    world.spawn((
        0u8,
        GameLog {
            entries: vec!["Welcome to World's Viscera".to_string()],
        },
    ));

    let map: Map = Map::new_dungeon_map();

    let player_entity = (
        Player {},
        Position {
            x: map.rooms[0].center()[0] as i32,
            y: map.rooms[0].center()[1] as i32,
        },
        Renderable {
            texture_name: TextureName::Creatures,
            texture_region: Rect {
                x: 0.0,
                y: 0.0,
                w: TILE_SIZE as f32,
                h: TILE_SIZE as f32,
            },
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: VIEW_RADIUS,
            must_recalculate: true,
        },
        Named {
            name: String::from("Player"),
        },
        CombatStats {
            //TOdO Random
            current_stamina: 6,
            max_stamina: 6,
            armor: 2,
            attack_dice: 6,
            current_toughness: 10,
            max_toughness: 10,
        },
        Damageable { damage_received: 0 },
    );

    world.spawn(player_entity);

    let mut monsters = Vec::new();
    for (index, room) in map.rooms.iter().skip(1).enumerate() {
        let monster_entity = (
            Monster {},
            Position {
                x: room.center()[0] as i32,
                y: room.center()[1] as i32,
            },
            Renderable {
                texture_name: TextureName::Creatures,
                texture_region: Rect {
                    x: 1.0 * TILE_SIZE as f32, //TODO fix
                    y: 0.0,
                    w: TILE_SIZE as f32,
                    h: TILE_SIZE as f32,
                },
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: VIEW_RADIUS,
                must_recalculate: true,
            },
            Named {
                name: String::from(format!("Deep one #{index}")),
            },
            BlocksTile {},
            CombatStats {
                //TOdO Random
                current_stamina: 3,
                max_stamina: 3,
                armor: 1,
                attack_dice: 4,
                current_toughness: 8,
                max_toughness: 8,
            },
            Damageable { damage_received: 0 },
        );
        monsters.push(monster_entity);
    }

    world.spawn_batch(monsters);

    let map_entity = builder.add(map).build();
    world.spawn(map_entity);

    world
}

fn do_game_logic(game_state: &mut EngineState, next_state: RunState) -> RunState {
    let game_over;
    DamageManager::manage_damage(&game_state.ecs_world);
    game_over = DamageManager::remove_dead(&mut game_state.ecs_world);
    //Proceed on game logic ifis not Game Over
    if !game_over {
        FovSystem::calculate_fov(&game_state.ecs_world);
        MapIndexing::index_map(&game_state.ecs_world);
        return next_state;
    } else {
        return RunState::GameOver;
    }
}
