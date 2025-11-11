use hecs::{Entity, World};

use crate::{
    components::{
        common::{CanListen, GameLog},
        items::InBackback,
        player::{Player, SpecialViewMode},
    },
    dialog::DialogAction,
    inventory::InventoryAction,
};

#[derive(PartialEq, Debug)]
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
pub struct EngineState {
    pub ecs_world: World, // World of ECS, where the framework lives
    pub run_state: RunState,
}

// State implementations
impl EngineState {
    /// Retain the player, gamelog and backpack items when changing Zone
    pub fn get_entities_to_delete_on_zone_change(&mut self) -> Vec<Entity> {
        let mut entities_to_delete: Vec<Entity> = Vec::new();

        let player_id = Player::get_entity_id(&self.ecs_world);

        let mut must_delete;
        let all_entities_in_world: Vec<Entity> =
            self.ecs_world.iter().map(|eref| eref.entity()).collect();

        for entity in all_entities_in_world {
            must_delete = true;

            if entity.id() == player_id
                || self
                    .ecs_world
                    .satisfies::<&InBackback>(entity)
                    .expect("cannot extract satisfies value")
                || self
                    .ecs_world
                    .satisfies::<&GameLog>(entity)
                    .expect("cannot extract satisfies value")
            {
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
