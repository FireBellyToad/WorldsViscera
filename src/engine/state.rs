use hecs::World;

#[derive(PartialEq)]
pub enum RunState {
    SystemsRunning,
    WaitingPlayerInput,
    PlayerTurn,
    MonsterTurn,
    GameOver,
    ShowInventory,
    ShowDropInventory
}

// Game state struct
pub struct EngineState {
    pub ecs_world: World, // World of ECS, where the framework lives
    pub run_state: RunState,
}

// State implementations
impl EngineState {}
