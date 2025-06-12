use hecs::World;

#[derive(PartialEq, Debug)]
pub enum RunState {
    RoundStart,
    WaitingPlayerInput,
    PlayerTurn,
    MonsterTurn,
    GameOver,
    ShowEatInventory,
    ShowDropInventory,
    ShowInvokeInventory,
    MouseTargeting,
}

// Game state struct
pub struct EngineState {
    pub ecs_world: World, // World of ECS, where the framework lives
    pub run_state: RunState,
}

// State implementations
impl EngineState {}
