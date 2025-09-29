use hecs::{Entity, World};

use crate::components::{
    common::{MyTurn, WaitingToAct},
    player::Player,
};

pub struct TurnCheck {}

impl TurnCheck {
    pub fn run(ecs_world: &mut World) {
        let mut entities_that_can_act: Vec<Entity> = Vec::new();
        let player_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to act
            let mut actors = ecs_world.query::<&mut WaitingToAct>();

            for (actor, wants_to_act) in &mut actors {
                wants_to_act.tick_countdown -= 1;
                if actor.id() == player_id {
                    // println!("Player's tick_countdown {}", wants_to_act.tick_countdown);
                } else {
                    // println!(
                    //     "Entity {} tick_countdown {}",
                    //     actor.id(),
                    //     wants_to_act.tick_countdown
                    // );
                }
                if wants_to_act.tick_countdown == 0 {
                    entities_that_can_act.push(actor);
                }
            }
        }

        for entity in entities_that_can_act {
            // Stop wait and act!
            // if entity.id() == player_id {
            //     println!("Player's turn soon");
            // } else {
            //         println!(
            //             "Entity {} turn soon",
            //             entity.id()
            //         );
            //     }
            let _ = ecs_world.exchange_one::<WaitingToAct, MyTurn>(entity, MyTurn {});
        }
    }
}
