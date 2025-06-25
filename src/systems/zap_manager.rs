use hecs::{Entity, World};

use crate::{
    components::{
        combat::{CombatStats, InflictsDamage, SufferingDamage, WantsToZap},
        common::{GameLog, Named, Position},
        items::WantsToInvoke,
    }, maps::zone::Zone, utils::{particle_animation::ParticleAnimation, roll::Roll}
};

pub struct ZapManager {}

impl ZapManager {
    pub fn run(ecs_world: &mut World) {
        let mut wants_to_zap_list: Vec<Entity> = Vec::new();
        let mut invokable_list: Vec<Entity> = Vec::new();
        let mut particle_animations: Vec<ParticleAnimation> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to collect items
            let mut zappers = ecs_world.query::<(&WantsToZap, &WantsToInvoke, &Position)>();

            //Log all the pick ups
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            let mut zone_query = ecs_world.query::<&Zone>();
            let (_e, zone) = zone_query.iter().last().expect("Zone is not in hecs::World");

            for (zapper, (wants_zap, wants_invoke, zapper_position)) in &mut zappers {
                let index = Zone::get_index_from_xy(wants_zap.target.0, wants_zap.target.1);
                let target_list = &zone.tile_content[index];

                // Do not draw if zapping himself
                if zapper_position.x != wants_zap.target.0
                    || zapper_position.y != wants_zap.target.1
                {
                    // TODO why here we do not use the line effect in gameplay?
                    particle_animations.push(ParticleAnimation::new_line(
                        (zapper_position.x, zapper_position.y),
                        (wants_zap.target.0, wants_zap.target.1),
                    ));
                }

                for &target in target_list {
                    let target_stats = ecs_world.get::<&CombatStats>(target).unwrap();
                    let item_damage = ecs_world.get::<&InflictsDamage>(wants_invoke.item).unwrap();
                    let target_damage = ecs_world.get::<&mut SufferingDamage>(target);

                    //Sum damage, keeping in mind that could not have SufferingDamage component
                    if target_damage.is_ok() {
                        let damage_roll =
                            Roll::dice(item_damage.number_of_dices, item_damage.dice_size);
                        let saving_throw_roll = Roll::d20();

                        // Show appropriate log messages
                        let named_attacker = ecs_world.get::<&Named>(zapper).unwrap();
                        let named_target = ecs_world.get::<&Named>(target).unwrap();
                        game_log.entries.push(format!(
                            "{} saving with {} against dexterity {} ",
                            named_target.name, saving_throw_roll, target_stats.current_dexterity
                        ));

                        // Dextery Save made halves damage
                        if saving_throw_roll > target_stats.current_dexterity {
                            target_damage.unwrap().damage_received += damage_roll;
                        } else {
                            target_damage.unwrap().damage_received += damage_roll / 2;
                            game_log
                                .entries
                                .push(format!("{} ducks some of the blow!", named_target.name));
                        }

                        game_log.entries.push(format!(
                            "{} zaps the {} for {} damage",
                            named_attacker.name, named_target.name, damage_roll
                        ));
                    };
                }

                // prepare lists for removal
                wants_to_zap_list.push(zapper);
                invokable_list.push(wants_invoke.item)
            }
        }

        // Remove owner's will to invoke and zap
        for zapper in wants_to_zap_list {
            let _ = ecs_world.remove_one::<WantsToInvoke>(zapper);
            let _ = ecs_world.remove_one::<WantsToZap>(zapper);
        }

        for particle in particle_animations {
            let _ = ecs_world.spawn((true, particle));
        }

        // Remove invokable item: is consumed!
        for invokable in invokable_list {
            let _ = ecs_world.despawn(invokable);
        }
    }
}
