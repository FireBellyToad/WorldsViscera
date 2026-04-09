use hecs::{Entity, World};

use crate::{
    components::{
        common::{CanListen, GameLog},
        items::InBackback,
        player::SpecialViewMode,
    },
    dialog::DialogAction,
    inventory::InventoryAction,
    maps::zone::Zone,
};

#[derive(PartialEq, Debug, Clone)]
pub enum RunState {
    TitleScreen,
    BeforeTick,
    WaitingPlayerInput,
    DoTick,
    GameOver,
    ShowInventory(InventoryAction),
    ShowDialog(DialogAction),
    MouseTargeting(SpecialViewMode),
    DrawParticles,
    GoToNextZone,
}

// Game state struct
pub struct GameState {
    pub ecs_world: World, // World of ECS, where the framework lives
    pub run_state: RunState,
    pub current_player_entity: Option<Entity>,
    pub current_zone: Option<Zone>,
    pub game_log: GameLog,
    pub debug_mode: bool,
    pub debug_monster_vision: bool,
    pub current_tick: u32,
}

// State implementations
impl GameState {
    /// Retain the player, gamelog and backpack items when changing Zone
    pub fn get_entities_to_delete_on_zone_change(&mut self) -> Vec<Entity> {
        let mut entities_to_delete: Vec<Entity> = Vec::new();
        let player_id = self
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let mut must_delete;

        for entity_ref in self.ecs_world.iter() {
            let entity = entity_ref.entity();

            must_delete = true;

            // Do not despawn objects in player's backpack
            // All the others must be deleted or else could be casually reassigned to NPCs
            if let Ok(in_backpack) = self.ecs_world.get::<&InBackback>(entity) {
                if in_backpack.owner.id() == player_id {
                    must_delete = false;
                }
            } else if entity.id() == player_id {
                must_delete = false;
                // Clear listen cache
                if let Ok(mut can_listen) = self.ecs_world.get::<&mut CanListen>(entity) {
                    can_listen.listen_cache.clear();
                }
            }
            if must_delete {
                entities_to_delete.push(entity);
            }
        }

        entities_to_delete
    }
}
