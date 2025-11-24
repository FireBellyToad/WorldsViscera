use crate::{
    components::common::Experience,
    maps::arena_zone_builder::ArenaZoneBuilder,
    systems::{
        advancement_system::AdvancementSystem, debugger::Debugger, dig_manager::DigManager,
        leave_trail_system::LeaveTrailSystem, ranged_manager::RangedManager,
    },
};
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
use spawning::spawner::Spawn;
use systems::{
    damage_manager::DamageManager, eating_edibles::EatingEdibles, fov::FieldOfView,
    item_collection::ItemCollection, item_dropping::ItemDropping, melee_manager::MeleeManager,
    monster_think::MonsterThink,
};

use crate::{
    components::common::{Position, Viewshed},
    dialog::Dialog,
    maps::{
        ZoneBuilder, drunken_walk_zone_builder::DrunkenWalkZoneBuilder,
        test_zone_builder::TestZoneBuilder, zone::Zone,
    },
    systems::{
        apply_system::ApplySystem, automatic_healing::AutomaticHealing,
        decay_manager::DecayManager, drinking_quaffables::DrinkingQuaffables,
        fuel_manager::FuelManager, hidden_manager::HiddenManager, hunger_check::HungerCheck,
        item_equipping::ItemEquipping, map_indexing::MapIndexing,
        monster_approach::MonsterApproach, particle_manager::ParticleManager,
        smell_manager::SmellManager, sound_system::SoundSystem, thirst_check::ThirstCheck,
        turn_checker::TurnCheck, wet_manager::WetManager, zap_manager::ZapManager,
    },
    utils::assets::Load,
};

mod components;
mod constants;
mod dialog;
mod draw;
mod engine;
mod inventory;
mod maps;
mod spawning;
mod systems;
mod utils;

//Game configuration
fn get_game_configuration() -> Conf {
    // Since when I switched to linux mint 22.22 with Nvidia gtx 4070 driver 580,
    // I had to duplicate the windows size to fit the game content... why?
    Conf {
        window_title: "World's Viscera".to_string(),
        fullscreen: false,
        window_height: WINDOW_HEIGHT * 2,
        window_width: WINDOW_WIDTH * 2,
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
        run_state: RunState::TitleScreen,
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
                RunState::TitleScreen => {
                    // Quit game on Q
                    if is_key_pressed(KeyCode::Q) {
                        break;
                    } else if get_last_key_pressed().is_some() {
                        game_state.ecs_world.clear();
                        populate_world(&mut game_state.ecs_world);
                        clear_input_queue();
                        game_state.run_state = RunState::BeforeTick;
                        tick = 0;
                    }
                }
                RunState::BeforeTick => {
                    tick += 1;
                    println!("BeforeTick ---------------------------- tick {}", tick);
                    do_before_tick_logic(&mut game_state);
                    game_state.run_state = RunState::DoTick;
                }
                RunState::DoTick => {
                    println!("DoTick ---------------------------- tick {}", tick);
                    do_in_tick_game_logic(&mut game_engine, &mut game_state);

                    if game_state.run_state != RunState::GameOver
                        && game_state.run_state != RunState::DrawParticles
                    {
                        if Player::can_act(&game_state.ecs_world) {
                            println!("Player's turn");
                            game_state.run_state = RunState::WaitingPlayerInput;
                        } else {
                            game_state.run_state = RunState::BeforeTick;
                        }
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
                RunState::ShowDialog(mode) => {
                    game_state.run_state = Dialog::handle_input(&mut game_state.ecs_world, mode);
                }
                RunState::MouseTargeting(special_view_mode) => {
                    game_state.run_state = Player::checks_input_for_targeting(
                        &mut game_state.ecs_world,
                        special_view_mode,
                    );
                }
                RunState::GoToNextZone => {
                    // Reset heal counter if the player did not wait
                    Player::reset_heal_counter(&game_state.ecs_world);
                    Player::wait_after_action(&mut game_state.ecs_world);
                    change_zone(&mut game_state);
                    clear_input_queue();
                    game_state.run_state = RunState::BeforeTick;
                }
                RunState::DrawParticles => {
                    ParticleManager::run(&mut game_state);
                }
            }

            // Keep this here, is needed to render correctly the particles!
            Draw::render_game(&game_state, &assets);
            next_frame().await;
        }
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
    let _ = ecs_world.spawn((true, zone));
}

fn change_zone(engine: &mut EngineState) {
    // Generate new seed, or else it will always generate the same things
    rand::srand(macroquad::miniquad::date::now() as _);

    let current_depth;
    // Scope for keeping borrow checker quiet
    {
        let mut zone_query = engine.ecs_world.query::<&Zone>();
        let (_, zone) = zone_query
            .iter()
            .last()
            .expect("Zone is not in hecs::World");
        current_depth = zone.depth;
    }

    let entities_to_delete = engine.get_entities_to_delete_on_zone_change();

    for e in entities_to_delete {
        let _ = engine.ecs_world.despawn(e);
    }

    let zone = DrunkenWalkZoneBuilder::build(current_depth + 1);

    // Scope for keeping borrow checker quiet
    {
        //Set player position in new zone and force a FOV recalculation. Also, award experience
        let mut player_query_viewshed = engine
            .ecs_world
            .query::<(&mut Position, &mut Viewshed, &mut Experience)>()
            .with::<&Player>();

        for (_, (player_position, player_viewshed, player_experience)) in &mut player_query_viewshed
        {
            let (x, y) = Zone::get_xy_from_index(zone.player_spawn_point);
            player_position.x = x;
            player_position.y = y;

            player_viewshed.must_recalculate = true;

            // Award experience based on depth reached
            player_experience.value += zone.depth.pow(2);
            player_experience.auto_advance_counter = AUTO_ADVANCE_EXP_COUNTER_START;
        }
    }

    Spawn::everyhing_in_map(&mut engine.ecs_world, &zone);

    // Add zone (previous shuold be removed)
    engine.ecs_world.spawn((true, zone));
}

fn do_before_tick_logic(game_state: &mut EngineState) {
    TurnCheck::run(&mut game_state.ecs_world);
    RangedManager::check_ammo_counts(&mut game_state.ecs_world);
    AutomaticHealing::run(&mut game_state.ecs_world);
    DecayManager::run(&mut game_state.ecs_world);
    HungerCheck::run(&mut game_state.ecs_world);
    ThirstCheck::run(&mut game_state.ecs_world);
    FuelManager::check_fuel(&mut game_state.ecs_world);
    WetManager::run(&mut game_state.ecs_world);
    HiddenManager::run(&mut game_state.ecs_world);
    MonsterThink::run(&mut game_state.ecs_world);
    LeaveTrailSystem::handle_spawned_trail(&mut game_state.ecs_world);
    AdvancementSystem::run(&mut game_state.ecs_world);
}

fn do_in_tick_game_logic(game_engine: &mut GameEngine, game_state: &mut EngineState) {
    // Every System that could produce particle animations should be run before the particle manager check
    // This makes sure that the particle animations will not be executed after the Entity has been killed
    ZapManager::run(&mut game_state.ecs_world);
    RangedManager::run(&mut game_state.ecs_world);
    FuelManager::do_refills(&mut game_state.ecs_world);
    //If there are particles, skip everything and draw
    if !ParticleManager::check_if_animations_are_present(game_engine, game_state) {
        MeleeManager::run(&mut game_state.ecs_world);
        DamageManager::run(&game_state.ecs_world);
        DamageManager::remove_dead_and_check_gameover(game_state);
        //Proceed on game logic if is not Game Over
        if game_state.run_state != RunState::GameOver {
            ApplySystem::check(&mut game_state.ecs_world);
            ApplySystem::do_applications(game_state);
            ItemCollection::run(&mut game_state.ecs_world);
            ItemEquipping::run(&mut game_state.ecs_world);
            ItemDropping::run(&mut game_state.ecs_world);
            EatingEdibles::run(&mut game_state.ecs_world);
            DigManager::run(&mut game_state.ecs_world);
            DrinkingQuaffables::run(&mut game_state.ecs_world);
            SoundSystem::run(&mut game_state.ecs_world);
            LeaveTrailSystem::run(&mut game_state.ecs_world);
            MonsterApproach::run(&mut game_state.ecs_world);
            // These Systems must always be run last
            MapIndexing::run(&game_state.ecs_world);
            FieldOfView::calculate(&game_state.ecs_world);
        }
    }
}

fn do_tickless_logic(game_state: &mut EngineState) {
    SmellManager::run(&mut game_state.ecs_world);

    #[cfg(not(target_arch = "wasm32"))]
    Debugger::run(&mut game_state.ecs_world);
}
