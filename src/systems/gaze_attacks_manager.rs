use crate::{
    components::{combat::WantsToGaze, common::*},
    engine::state::GameState,
    maps::zone::Zone,
};

pub struct GazeAttacksManager {}

impl GazeAttacksManager {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_entity_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();

        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");

        // Scope to keep the borrow checker quiet
        {
            let mut gazers = ecs_world
                .query::<(&WantsToGaze, &Position, &Named)>()
                .with::<&MyTurn>();

            //For each Entity with Components Viewshed and Position
            for (_, (wants_to_gaze, position, named)) in &mut gazers {
                let index = Zone::get_index_from_xy(&position.x, &position.y);
                // if target can see the gazer
                if let Ok(target_view) = ecs_world.get::<&Viewshed>(wants_to_gaze.target)
                    && target_view.visible_tiles.contains(&index)
                {
                    // Perform gaze attack logic here TODO

                    //Log attack
                    if player_entity_id == wants_to_gaze.target.id() {
                        game_state
                            .game_log
                            .entries
                            .push(format!("{} gazes at you", named.name));
                    } else if let Ok(t_pos) = ecs_world.get::<&Position>(wants_to_gaze.target)
                        && zone.visible_tiles[Zone::get_index_from_xy(&t_pos.x, &t_pos.y)]
                        && zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)]
                    {
                        let target_name = ecs_world
                            .get::<&Named>(wants_to_gaze.target)
                            .expect("target of gaze must be named");

                        game_state.game_log.entries.push(format!(
                            "The {} gazes at the {}",
                            named.name, target_name.name
                        ));
                    }
                }
            }
        }
    }
}
