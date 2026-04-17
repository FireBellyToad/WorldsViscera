use crate::{
    components::{combat::Grappled, common::Experience},
    maps::{
        arena_zone_builder::ArenaZoneBuilder, crystal_cave_builder::CrystalCaveBuilder,
        test_zone_builder::TestZoneBuilder,
    },
    systems::{
        advancement_system::AdvancementSystem, dig_manager::DigManager,
        gaze_attacks_manager::GazeAttacksManager, health_manager::HealthManager,
        leave_trail_system::LeaveTrailSystem, ranged_manager::RangedManager,
        special_tiles_system::SpecialTilesSystem, spell_manager::SpellManager,
        trade_system::TradeSystem,
    },
};
use components::{common::GameLog, player::Player};
use constants::*;
use draw::Draw;
use engine::{
    gameengine::GameEngine,
    state::{GameState, RunState},
};
use hecs::World;
use inventory::Inventory;
use macroquad::prelude::*;
use spawning::spawner::Spawn;
use systems::{
    damage_manager::DamageManager, eating_edibles::EatingEdibles, fov_manager::FieldOfViewManager,
    item_collection::ItemCollection, item_dropping::ItemDropping, melee_manager::MeleeManager,
    monster_think::MonsterThink,
};

use crate::{
    components::common::{Position, Viewshed},
    dialog::Dialog,
    maps::{ZoneBuilder, drunken_walk_zone_builder::DrunkenWalkZoneBuilder, zone::Zone},
    systems::{
        apply_system::ApplySystem, automatic_healing::AutomaticHealing,
        decay_manager::DecayManager, drinking_quaffables::DrinkingQuaffables,
        fuel_manager::FuelManager, hidden_manager::HiddenManager, hunger_check::HungerCheck,
        invoke_manager::InvokeManager, item_equipping::ItemEquipping, map_indexing::MapIndexing,
        monster_approach::MonsterApproach, particle_manager::ParticleManager,
        smell_manager::SmellManager, sound_system::SoundSystem, thirst_check::ThirstCheck,
        turn_checker::TurnCheck, wet_manager::WetManager,
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
    Conf {
        window_title: "World's Viscera".to_owned(),
        fullscreen: false,
        window_height: WINDOW_HEIGHT,
        window_width: WINDOW_WIDTH,
        window_resizable: true,
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
    let mut game_state = GameState {
        ecs_world: World::new(),
        run_state: RunState::TitleScreen,
        current_player_entity: None,
        current_zone: None,
        game_log: GameLog::new(),
        debug_mode: false,
        debug_monster_vision: false,
        current_tick: 0,
    };
    game_state.game_log.add_entry("Welcome to World's Viscera!");
    populate_world(&mut game_state);

    loop {
        //If there are particles, skip everything and draw
        if game_state.run_state != RunState::GameOver {
            let _ =
                ParticleManager::check_if_animations_are_present(&mut game_engine, &mut game_state);
        }

        if game_engine.next_tick() {
            // Run system only while not paused, or else wait for player input.
            // Make the whole game turn based

            #[cfg(not(target_arch = "wasm32"))]
            do_debug_logic(&mut game_state);

            match game_state.run_state.clone() {
                RunState::TitleScreen => {
                    // Quit game on Q\
                    if is_key_pressed(KeyCode::Q) {
                        break;
                    } else if get_last_key_pressed().is_some() {
                        game_state.ecs_world.clear();
                        populate_world(&mut game_state);
                        clear_input_queue();
                        game_state.run_state = RunState::BeforeTick;
                        game_state.current_tick = 0;
                    }
                }
                RunState::BeforeTick => {
                    game_state.current_tick += 1;
                    println!(
                        "BeforeTick ---------------------------- tick {}",
                        game_state.current_tick
                    );
                    do_before_tick_logic(&mut game_state);

                    if game_state.run_state != RunState::GameOver
                        && game_state.run_state != RunState::DrawParticles
                    {
                        game_state.run_state = RunState::DoTick;
                    }
                }
                RunState::WaitingPlayerInput => {
                    SmellManager::run(&mut game_state);
                    Player::checks_keyboard_input(&mut game_state);
                }
                RunState::DoTick => {
                    println!(
                        "DoTick ---------------------------- tick {}",
                        game_state.current_tick
                    );
                    do_in_tick_game_logic(&mut game_engine, &mut game_state);

                    match game_state.run_state {
                        RunState::GameOver | RunState::ShowDialog(_) | RunState::DrawParticles => {}
                        _ => {
                            if Player::can_act(
                                &game_state.ecs_world,
                                game_state
                                    .current_player_entity
                                    .expect("No current player entity"),
                            ) {
                                println!("Player's turn");
                                game_state.run_state = RunState::WaitingPlayerInput;
                            } else {
                                game_state.run_state = RunState::BeforeTick;
                            }
                        }
                    }
                }
                RunState::GameOver => {
                    // Quit game on Q
                    if is_key_pressed(KeyCode::Q) {
                        break;
                    } else if is_key_pressed(KeyCode::R) {
                        game_state.ecs_world.clear();
                        populate_world(&mut game_state);
                        clear_input_queue();
                        game_state.run_state = RunState::BeforeTick;
                        game_state.current_tick = 0;
                    }
                }
                RunState::ShowInventory(mode) => {
                    Inventory::handle_input(&mut game_state, mode);
                }
                RunState::ShowDialog(mode) => {
                    Dialog::handle_input(&mut game_state, mode.clone());
                }
                RunState::MouseTargeting(special_view_mode) => {
                    Player::checks_input_for_targeting(&mut game_state, special_view_mode);
                }
                RunState::GoToNextZone => {
                    Player::wait_after_action(&mut game_state, STANDARD_ACTION_MULTIPLIER);
                    change_zone(&mut game_state);
                    clear_input_queue();
                    game_state.run_state = RunState::BeforeTick;
                }
                RunState::DrawParticles => {
                    ParticleManager::run(&mut game_state);
                }
            }

            // Keep this here, is needed to render correctly the particles!
            Draw::render_game(&mut game_state, &assets);
            next_frame().await;
        }
    }
}

fn populate_world(game_state: &mut GameState) {
    // Generate new seed, or else it will always generate the same things
    rand::srand(macroquad::miniquad::date::now() as _);

    let zone = TestZoneBuilder::build(1, &mut game_state.ecs_world);

    game_state.current_player_entity = Some(Spawn::player(&mut game_state.ecs_world, &zone));
    Spawn::everyhing_in_map(&mut game_state.ecs_world, &zone);

    // Add zone
    game_state.current_zone = Some(zone);
}

fn change_zone(game_state: &mut GameState) {
    // Generate new seed, or else it will always generate the same things
    rand::srand(macroquad::miniquad::date::now() as _);

    let current_depth = game_state
        .current_zone
        .as_ref()
        .expect("must have Some Zone")
        .depth;

    let entities_to_delete = game_state.get_entities_to_delete_on_zone_change();

    let player = game_state
        .current_player_entity
        .expect("Player id should be set");

    // Remove any existing Grappled component from the player
    let _ = game_state.ecs_world.remove_one::<&Grappled>(player);

    for e in entities_to_delete {
        let _ = game_state.ecs_world.despawn(e);
    }

    // Build new zone based on depth.
    // -1 because current depth is then incremented to get the next Zone
    let zone = match current_depth + 1 {
        CRYSTAL_CAVE_DEPTH => {
            CrystalCaveBuilder::build(current_depth + 1, &mut game_state.ecs_world)
        }
        _ => DrunkenWalkZoneBuilder::build(current_depth + 1, &mut game_state.ecs_world),
    };

    // Scope for keeping borrow checker quiet
    {
        //Set player position in new zone and force a FOV recalculation. Also, award experience
        let mut player_query_viewshed = game_state
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

    Spawn::everyhing_in_map(&mut game_state.ecs_world, &zone);

    // Add zone (previous shuold be removed)
    game_state.current_zone = Some(zone);
}

fn do_before_tick_logic(game_state: &mut GameState) {
    TurnCheck::run(game_state);
    RangedManager::check_ammo_counts(game_state);
    AutomaticHealing::run(game_state);
    DecayManager::run(game_state);
    HungerCheck::run(game_state);
    ThirstCheck::run(game_state);
    HealthManager::run(game_state);
    FuelManager::check_fuel(game_state);
    WetManager::run(game_state);
    HiddenManager::run(game_state);
    MonsterThink::run(game_state);
    LeaveTrailSystem::handle_spawned_trail(game_state);
    AdvancementSystem::run(game_state);
    SpellManager::decrease_cooldowns(game_state);
    // These Systems must always be run last
    MapIndexing::run(game_state);
    FieldOfViewManager::calculate(game_state);
}

fn do_in_tick_game_logic(game_engine: &mut GameEngine, game_state: &mut GameState) {
    // Every System that could produce particle animations should be run before the particle manager check
    // This makes sure that the particle animations will not be executed after the Entity has been killed
    SpellManager::run(game_state);
    RangedManager::run(game_state);
    FuelManager::do_refills(game_state);
    InvokeManager::run(game_state);
    //If there are particles, skip everything and draw
    if !ParticleManager::check_if_animations_are_present(game_engine, game_state) {
        GazeAttacksManager::run(game_state);
        MeleeManager::run(game_state);
        DamageManager::run(game_state);
        DamageManager::remove_dead_and_check_gameover(game_state);
        //Proceed on game logic if is not Game Over
        if game_state.run_state != RunState::GameOver {
            ApplySystem::check(game_state);
            ApplySystem::do_applications(game_state);
            ItemCollection::run(game_state);
            ItemEquipping::run(game_state);
            ItemDropping::run(game_state);
            DigManager::run(game_state);
            // EatingEdibles must run after DigManager because monsters digging also eat stone
            EatingEdibles::run(game_state);
            DrinkingQuaffables::run(game_state);
            SoundSystem::run(game_state);
            LeaveTrailSystem::run(game_state);
            MonsterApproach::run(game_state);
            TradeSystem::run(game_state);
            // These Systems must always be run last
            MapIndexing::run(game_state);
            SpecialTilesSystem::grow_on_step_tiles(game_state);
            FieldOfViewManager::calculate(game_state);
            TurnCheck::check_for_turn_reset(game_state);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn do_debug_logic(game_state: &mut GameState) {
    {
        if is_key_pressed(KeyCode::F12) {
            game_state.debug_mode = !game_state.debug_mode;
            println!("Debug mode {}", game_state.debug_mode);
        }

        if game_state.debug_mode {
            use crate::systems::debugger::Debugger;
            Debugger::run(game_state);
            // TODO spawn what prompt
            if is_key_pressed(KeyCode::F11) {
                use crate::components::health::Stunned;

                let _ = game_state.ecs_world.insert_one(
                    game_state.current_player_entity.expect("must be some"),
                    Stunned { tick_counter: 3 },
                );
            } else if is_key_pressed(KeyCode::F10) {
                Spawn::slingshot(&mut game_state.ecs_world, MAP_WIDTH / 2, MAP_HEIGHT / 2);
            } else if is_key_pressed(KeyCode::F9) {
                Spawn::refugee(&mut game_state.ecs_world, MAP_WIDTH / 2, MAP_HEIGHT / 2);
            } else if is_key_pressed(KeyCode::F8) {
                use crate::components::combat::CombatStats;

                let mut stats = game_state
                    .ecs_world
                    .get::<&mut CombatStats>(
                        game_state.current_player_entity.expect("must be some"),
                    )
                    .expect("must have stats");
                stats.current_dexterity = 1;
            } else if is_key_pressed(KeyCode::F7) {
                use std::collections::HashMap;

                use crate::{
                    components::health::{DiseaseType, Diseased},
                    utils::roll::Roll,
                };

                let mut tick_counters = HashMap::new();
                tick_counters.insert(
                    DiseaseType::Fever,
                    (MAX_DISEASE_TICK_COUNTER + Roll::d20(), false),
                );
                tick_counters.insert(
                    DiseaseType::FleshRot,
                    (MAX_DISEASE_TICK_COUNTER + Roll::d20(), false),
                );
                tick_counters.insert(
                    DiseaseType::Calcification,
                    (MAX_DISEASE_TICK_COUNTER + Roll::d20(), false),
                );
                let _ = game_state.ecs_world.insert_one(
                    game_state.current_player_entity.expect("must be some"),
                    Diseased { tick_counters },
                );
            } else if is_key_pressed(KeyCode::F6) {
                game_state.debug_monster_vision = !game_state.debug_monster_vision;
            } else if is_key_pressed(KeyCode::F5) {
                Spawn::colossal_worm(
                    &mut game_state.ecs_world,
                    38,
                    MAP_HEIGHT / 2,
                    game_state.current_zone.as_ref().unwrap(),
                );
            } else if is_key_pressed(KeyCode::F4) {
                use crate::components::combat::Grappled;

                if let Ok(grappled) = game_state
                    .ecs_world
                    .get::<&Grappled>(game_state.current_player_entity.expect("must be some"))
                {
                    use crate::components::combat::SufferingDamage;

                    if let Ok(mut damage) = game_state
                        .ecs_world
                        .get::<&mut SufferingDamage>(grappled.by)
                    {
                        damage.damage_received += 10000;
                    }
                }
            } else if is_key_pressed(KeyCode::F3) {
                game_state.run_state = RunState::GoToNextZone;
            }
        }
    }
}
