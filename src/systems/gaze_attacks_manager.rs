use hecs::Entity;
use macroquad::ui::Vertex;

use crate::{
    components::{
        combat::{CombatStats, GazeAttack, GazeEffectEnum, WantsToGaze},
        common::*,
        health::Blind,
    },
    engine::state::GameState,
    maps::zone::Zone,
    utils::roll::Roll,
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

        let mut end_gazing: Vec<Entity> = Vec::new();
        let mut gazed_targets: Vec<(Entity, GazeEffectEnum)> = Vec::new();

        // Scope to keep the borrow checker quiet
        {
            let mut gazers = ecs_world
                .query::<(&WantsToGaze, &GazeAttack, &Position, &Named)>()
                .with::<&MyTurn>();

            //For each Entity with Components Viewshed and Position
            for (gazer, (wants_to_gaze, gaze_attack, position, named)) in &mut gazers {
                end_gazing.push(gazer);
                let index = Zone::get_index_from_xy(&position.x, &position.y);
                // if target can see the gazer
                if let Ok(target_view) = ecs_world.get::<&Viewshed>(wants_to_gaze.target)
                    && target_view.visible_tiles.contains(&index)
                {
                    // Get all target info
                    let mut target_query = ecs_world
                        .query_one::<(&CombatStats, &Position, &Named)>(wants_to_gaze.target)
                        .expect("target of gaze must have components");
                    let (target_stats, t_pos, target_name) =
                        target_query.get().expect("Query should get something!");

                    // One save to avoid the gaze, the other to resist once the target has been gazed upon
                    if Roll::d20() <= target_stats.current_dexterity {
                        if player_entity_id == wants_to_gaze.target.id() {
                            game_state
                                .game_log
                                .entries
                                .push(format!("You avoid the {}'s gaze!", named.name));
                        }
                    } else if Roll::d20() <= target_stats.current_toughness {
                        if player_entity_id == wants_to_gaze.target.id() {
                            game_state
                                .game_log
                                .entries
                                .push(format!("You resist the {}'s gaze!", named.name));
                        }
                    } else {
                        gazed_targets.push((wants_to_gaze.target, gaze_attack.effect.clone()));

                        let effect = match gaze_attack.effect {
                            GazeEffectEnum::Blindness => "blinds",
                        };

                        //Log attack
                        if player_entity_id == wants_to_gaze.target.id() {
                            game_state
                                .game_log
                                .entries
                                .push(format!("The {} {} you with its gaze!", named.name, effect));
                        } else if zone.visible_tiles[Zone::get_index_from_xy(&t_pos.x, &t_pos.y)]
                            && zone.visible_tiles[Zone::get_index_from_xy(&position.x, &position.y)]
                        {
                            game_state.game_log.entries.push(format!(
                                "The {} {} the {} with its gaze!",
                                named.name, effect, target_name.name
                            ));
                        }
                    }
                }
            }
        }

        for gazer in end_gazing {
            let _ = ecs_world.remove_one::<WantsToGaze>(gazer);
        }

        for (gazer, effect) in gazed_targets {
            let _ = match effect {
                GazeEffectEnum::Blindness => ecs_world.insert_one(
                    gazer,
                    Blind {
                        tick_counter: Roll::d20() + 6,
                    },
                ),
            };
        }
    }
}
