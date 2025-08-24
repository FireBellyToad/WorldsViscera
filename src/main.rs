use components::{common::GameLog, player::Player};
use constants::*;
use draw::Draw;
use engine::{
    gameengine::GameEngine,
    state::{EngineState, RunState},
};
use hecs::World;
use inventory::Inventory;
use macroquad::prelude::*;
use spawner::Spawn;
use systems::{
    damage_manager::DamageManager, eating_edibles::EatingEdibles, fov::FieldOfView,
    item_collection::ItemCollection, item_dropping::ItemDropping, melee_manager::MeleeManager,
    monster_ai::MonsterAI,
};

use crate::{
    components::common::{Position, Viewshed},
    maps::{
        ZoneBuilder, arena_zone_builder::ArenaZoneBuilder,
        drunken_walk_zone_builder::DrunkenWalkZoneBuilder, zone::Zone,
    },
    systems::{
        automatic_healing::AutomaticHealing, decay_manager::DecayManager,
        drinking_quaffables::DrinkingQuaffables, fuel_manager::FuelManager,
        hunger_check::HungerCheck, map_indexing::MapIndexing, particle_manager::ParticleManager,
        smell_manager::SmellManager, status_manager::StatusManager, thirst_check::ThirstCheck,
        turn_checker::TurnCheck, wet_manager::WetManager, zap_manager::ZapManager,
    },
    utils::assets::Load,
};

mod components;
mod constants;
mod draw;
mod engine;
mod inventory;
mod maps;
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
    //Load resources inside zone
    let assets = Load::assets().await;

    //Init ECS
    let mut game_engine = GameEngine::new();
    let mut game_state = EngineState {
        ecs_world: create_ecs_world(),
        run_state: RunState::BeforeTick,
    };

    let mut tick = 0;
    loop {
        //If there are particles, skip everything and draw
        if game_state.run_state != RunState::GameOver {
            let _ =
                ParticleManager::check_if_animations_are_present(&mut game_engine, &mut game_state);
        }

        if game_engine.next_tick() {
            // Run system only while not paused, or else wait for player input.
            // Make the whole game turn based

            match game_state.run_state {
                RunState::BeforeTick => {
                    tick += 1;
                    println!("BeforeTick ---------------------------- tick {}", tick);
                    do_before_tick_logic(&mut game_state);
                    game_state.run_state = RunState::DoTick;
                }
                RunState::DoTick => {
                    println!("DoTick ---------------------------- tick {}", tick);
                    let is_game_over = do_in_tick_game_logic(&mut game_state);
                    //If there are particles, skip everything and draw
                    let must_run_particles = ParticleManager::check_if_animations_are_present(
                        &mut game_engine,
                        &mut game_state,
                    );

                    // TODO refactor
                    if !is_game_over {
                        if !must_run_particles {
                            if Player::can_act(&game_state.ecs_world) {
                                println!("Player's turn");
                                game_state.run_state = RunState::WaitingPlayerInput;
                            } else {
                                MonsterAI::act(&mut game_state.ecs_world);
                                game_state.run_state = RunState::BeforeTick;
                            }
                        }
                    } else {
                        game_state.run_state = RunState::GameOver;
                    }
                }
                RunState::WaitingPlayerInput => {
                    do_tickless_logic(&mut game_state);
                    game_state.run_state = Player::checks_keyboard_input(&mut game_state.ecs_world);
                }
                RunState::GameOver => {
                    // Quit game on Q
                    if is_key_pressed(KeyCode::Q) {
                        break;
                    } else if is_key_pressed(KeyCode::R) {
                        game_state.ecs_world.clear();
                        populate_world(&mut game_state.ecs_world);
                        clear_input_queue();
                        game_state.run_state = RunState::BeforeTick;
                        tick = 0;
                    }
                }
                RunState::ShowInventory(mode) => {
                    game_state.run_state = Inventory::handle_input(&mut game_state.ecs_world, mode);
                }
                RunState::MouseTargeting(special_view_mode) => {
                    game_state.run_state = Player::checks_input_for_targeting(
                        &mut game_state.ecs_world,
                        special_view_mode,
                    );
                }
                RunState::GoToNextZone => {
                    // Reset heal counter if the player did not wait
                    Player::reset_heal_counter(&mut game_state.ecs_world);
                    Player::wait_after_action(&mut game_state.ecs_world);
                    change_zone(&mut game_state);
                    clear_input_queue();
                    game_state.run_state = RunState::BeforeTick;
                }
                RunState::DrawParticles => {
                    ParticleManager::run(&mut game_state);
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

    let zone = ArenaZoneBuilder::build(1);

    Spawn::player(ecs_world, &zone);
    Spawn::everyhing_in_map(ecs_world, &zone);

    // Add zone
    let me = ecs_world.spawn((true, zone));

    println!("spawn entity {}", me.id());
}

fn change_zone(engine: &mut EngineState) {
    // Generate new seed, or else it will always generate the same things
    rand::srand(macroquad::miniquad::date::now() as _);

    let current_depth;
    // Scope for keeping borrow checker quiet
    {
        let mut zone_query = engine.ecs_world.query::<&Zone>();
        let (_e, zone) = zone_query
            .iter()
            .last()
            .expect("Zone is not in hecs::World");
        current_depth = zone.depth;
    }

    let entities_to_delete = engine.get_entities_to_delete_on_zone_change();

    //TODO froze for backtracking
    for e in entities_to_delete {
        let _ = engine.ecs_world.despawn(e);
    }

    let zone = DrunkenWalkZoneBuilder::build(current_depth + 1);

    //Set player position in new zone and force a FOV recalculation
    let player_entity = Player::get_entity(&engine.ecs_world);

    // Scope for keeping borrow checker quiet
    {
        let mut player_position = engine
            .ecs_world
            .get::<&mut Position>(player_entity)
            .unwrap();

        let (x, y) = Zone::get_xy_from_index(zone.player_spawn_point);
        player_position.x = x;
        player_position.y = y;

        let mut player_viewshed: hecs::RefMut<'_, Viewshed> = engine
            .ecs_world
            .get::<&mut Viewshed>(player_entity)
            .unwrap();
        player_viewshed.must_recalculate = true;
    }

    Spawn::everyhing_in_map(&mut engine.ecs_world, &zone);

    // Add zone (previous shuold be removed)
    //TODO froze for backtracking
    engine.ecs_world.spawn((true, zone));
}

fn do_before_tick_logic(game_state: &mut EngineState) {
    TurnCheck::run(&mut game_state.ecs_world);
    AutomaticHealing::run(&mut game_state.ecs_world);
    DecayManager::run(&mut game_state.ecs_world);
    HungerCheck::run(&mut game_state.ecs_world);
    ThirstCheck::run(&mut game_state.ecs_world);
    FuelManager::check_fuel(&mut game_state.ecs_world);
    WetManager::run(&mut game_state.ecs_world);
    StatusManager::run(&mut game_state.ecs_world);
}

fn do_in_tick_game_logic(game_state: &mut EngineState) -> bool {
    let game_over;
    ZapManager::run(&mut game_state.ecs_world);
    MeleeManager::run(&mut game_state.ecs_world);
    DamageManager::run(&game_state.ecs_world);
    game_over = DamageManager::remove_dead_and_check_gameover(&mut game_state.ecs_world);
    //Proceed on game logic if is not Game Over
    if game_over {
        return true;
    } else {
        MapIndexing::run(&game_state.ecs_world);
        FieldOfView::calculate(&game_state.ecs_world);
        ItemCollection::run(&mut game_state.ecs_world);
        ItemDropping::run(&mut game_state.ecs_world);
        EatingEdibles::run(&mut game_state.ecs_world);
        DrinkingQuaffables::run(&mut game_state.ecs_world);
        FuelManager::do_refills(&mut game_state.ecs_world);
    }
    false
}

fn do_tickless_logic(game_state: &mut EngineState) {
    SmellManager::run(&mut game_state.ecs_world);
}
