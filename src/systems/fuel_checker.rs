
use hecs::World;

use crate::components::{
    common::{GameLog, Named, Viewshed},
    items::{InBackback, Fuel},
    player::Player,
};

pub struct FuelCheck {}

impl FuelCheck {
    pub fn run(ecs_world: &mut World) {
        // List of light producers with fuel
        let mut lighters = ecs_world.query::<&mut Fuel>();

        let player_entity = Player::get_player_entity(ecs_world);

        let mut game_log_query = ecs_world.query::<&mut GameLog>();
        let (_e, game_log) = game_log_query
            .iter()
            .last()
            .expect("Game log is not in hecs::World");

        for (lighter, fuel) in &mut lighters {
            // Log fuel change for lantern used by player
            let entity_in_backpack = ecs_world.get::<&InBackback>(lighter);

            if entity_in_backpack.is_ok() {
                let in_backback = entity_in_backpack.unwrap();
                let named = ecs_world.get::<&Named>(lighter).unwrap();
                // Log messages for fuel status
                if player_entity.id() == in_backback.owner.id(){
                    match fuel.counter {
                        25 => {
                            game_log
                                .entries
                                .push(format!("Your {} is flickering", named.name));
                        }
                        1 => {
                            game_log
                                .entries
                                .push(format!("Your {} goes out", named.name));
                        }
                        _ => {}
                    }

                    //show immediately new vision
                    let mut player_viewshed =
                        ecs_world.get::<&mut Viewshed>(player_entity).unwrap();
                    player_viewshed.must_recalculate = true;
                }
            }

            //If fuel is less then 1, the lighter will not produce light
            if fuel.counter > 0 {
                fuel.counter -= 1;
            }
        }
    }
}
