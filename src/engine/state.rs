use hecs::{Entity, World};

use crate::{components::{common::GameLog, items::InBackback, player::Player}, inventory::InventoryAction};

#[derive(PartialEq, Debug)]
pub enum RunState {
    BeforeTick,
    WaitingPlayerInput,
    DoTick,
    GameOver,
    ShowInventory(InventoryAction),
    MouseTargeting,
    DrawParticles,
    GoToNextZone,
}

// Game state struct
pub struct EngineState {
    pub ecs_world: World, // World of ECS, where the framework lives
    pub run_state: RunState,
}

// State implementations
impl EngineState {
    
    /// Retain the player, gamelog and backpack items when changing Zone
    //TODO froze for backtracking
    pub fn get_entities_to_delete_on_zone_change(&mut self) -> Vec<Entity> {
        let mut entities_to_delete: Vec<Entity> = Vec::new();

        let player_id = Player::get_player_id(&self.ecs_world);

        let mut must_delete;
        let all_entities_in_world: Vec<Entity> = self.ecs_world.iter().map(|eref| eref.entity()).collect();
        
        for entity in all_entities_in_world {
            must_delete = true;

            if entity.id() == player_id
                || self.ecs_world.satisfies::<&InBackback>(entity).unwrap()
                || self.ecs_world.satisfies::<&GameLog>(entity).unwrap()
            {
                must_delete = false;
            }
            if must_delete {
                entities_to_delete.push(entity);
            }
        }

        entities_to_delete
    }
}
