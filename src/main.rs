use components::{common::GameLog, player::Player};
use constants::*;
use draw::Draw;
use engine::{
    gameengine::GameEngine,
    state::{EngineState, RunState},
};
use hecs::World;
use loader::Load;
use macroquad::prelude::*;
use map::Map;
use spawner::Spawn;
use systems::{
    damage_manager::DamageManager, fov::FovSystem, item_collection::ItemCollection,
    map_indexing::MapIndexing, monster_ai::MonsterAI,
};

mod assets;
mod components;
mod constants;
mod draw;
mod engine;
mod loader;
mod map;
mod spawner;
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
    let assets = Load::assets().await;

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
                    game_state.run_state = Player::player_input(&mut game_state.ecs_world)
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
                    } else if is_key_pressed(KeyCode::R) {
                        game_state.ecs_world.clear();
                        game_state.run_state = RunState::SystemsRunning;
                        populate_world(&mut game_state.ecs_world)
                    }
                }
                RunState::ShowInventory => {
                    clear_input_queue();
                    //TODO refactor
                    if is_key_pressed(KeyCode::Escape) {
                        game_state.run_state = RunState::WaitingPlayerInput;
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

    populate_world(&mut world);

    world
}

fn populate_world(ecs_world: &mut World) {
    // Generate new seed, or else it will always generate the same things
    rand::srand(macroquad::miniquad::date::now() as _);

    //Add Game log to world
    ecs_world.spawn((
        true,
        GameLog {
            entries: vec!["Welcome to World's Viscera".to_string()],
        },
    ));

    let map: Map = Map::new_dungeon_map();

    Spawn::player(ecs_world, &map);

    for room in map.rooms.iter().skip(1) {
        Spawn::in_room(ecs_world, room);
    }

    // Add map
    ecs_world.spawn((true, map));
}

fn do_game_logic(game_state: &mut EngineState, next_state: RunState) -> RunState {
    let game_over;
    DamageManager::manage_damage(&game_state.ecs_world);
    game_over = DamageManager::remove_dead(&mut game_state.ecs_world);
    //Proceed on game logic ifis not Game Over
    if !game_over {
        FovSystem::calculate_fov(&game_state.ecs_world);
        MapIndexing::index_map(&game_state.ecs_world);
        ItemCollection::run(&mut game_state.ecs_world);
        return next_state;
    } else {
        return RunState::GameOver;
    }
}
