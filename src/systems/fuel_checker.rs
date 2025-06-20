use std::fmt::format;

use hecs::World;

use crate::components::{common::{GameLog, Viewshed}, items::ProduceLight, player::{self, Player}};

pub struct FuelCheck {}

impl FuelCheck {
    pub fn run(ecs_world: &mut World) {
        // List of light producers with fuel
        let mut lighters = ecs_world.query::<&mut ProduceLight>();

        let player_entity = Player::get_player_entity(ecs_world);
        
            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");


        for (entity, produce_light) in &mut lighters {
            //If fuel is exactly 0, the lighter will not produce light
            // -1 is for infinite fuel
            if player_entity.id() == entity.id() && produce_light.fuel_counter == 1 {
                // TODO get real item name
                game_log.entries.push(format!("Your Lantern goes out"));
                //show immediately new vision
                let mut player_viewshed= ecs_world.get::<&mut Viewshed>(player_entity).unwrap();
                player_viewshed.must_recalculate = true;
            }


            if produce_light.fuel_counter > 0 {
                produce_light.fuel_counter -= 1;
            }
        }
    }
}
