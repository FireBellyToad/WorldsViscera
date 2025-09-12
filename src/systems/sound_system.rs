use hecs::World;

use crate::{
    components::{
        common::{CanListen, GameLog, Position, ProduceSound},
        player::Player,
    },
    constants::LISTEN_COOLDOWN_START,
    utils::{common::Utils, roll::Roll},
};

pub struct SoundSystem {}

impl SoundSystem {
    pub fn run(ecs_world: &mut World) {
        //Log all the equipments
        let mut game_log_query = ecs_world.query::<&mut GameLog>();
        let (_e, game_log) = game_log_query
            .iter()
            .last()
            .expect("Game log is not in hecs::World");

        // Scope for keeping borrow checker quiet
        {
            // 1 -  List of entities that can listen and list of entities that produce sounds
            let mut listeners = ecs_world
                .query::<(&mut CanListen, &Position)>()
                .with::<&Player>();
            let mut sound_producers = ecs_world.query::<(&ProduceSound, &Position)>();

            for (_l, (can_listen, listener_pos)) in &mut listeners {
                if can_listen.cooldown == 0 {
                    for (producer, (produce_sound, producer_pos)) in &mut sound_producers {
                        if Utils::distance(
                            producer_pos.x,
                            listener_pos.x,
                            producer_pos.y,
                            listener_pos.y,
                        ) < can_listen.radius
                        {
                            // Listen to all sound producers in a radius and manage listen cache
                            // TODO how about deaf characters?
                            can_listen
                                .listen_cache
                                .entry(producer.id())
                                .or_insert_with(|| {
                                    (producer, produce_sound.sound_log.clone(), false)
                                });
                        } else if can_listen.listen_cache.contains_key(&producer.id()) {
                            let _ = can_listen.listen_cache.remove(&producer.id());
                        }
                    }

                    // Play a random sound that was not already played
                    let random_sound = Roll::dice(1, can_listen.listen_cache.len() as i32) as usize;
                    for (index, (_, (_, listen_log, already_listened))) in
                        can_listen.listen_cache.iter_mut().enumerate()
                    {
                        if !*already_listened && index == random_sound - 1 {
                            game_log.entries.push(format!("You hear {}", listen_log));
                            *already_listened = true;
                            break;
                        }
                    }

                    can_listen.cooldown = Roll::d20() + LISTEN_COOLDOWN_START;
                } else {
                    can_listen.cooldown -= 1;
                }
            }
        }
    }
}
