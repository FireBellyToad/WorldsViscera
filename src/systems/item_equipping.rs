use std::borrow::Cow;

use hecs::Entity;

use crate::{
    components::{
        actions::WantsToEquip,
        combat::CombatStats,
        common::{Immunity, Named, Position},
        items::{BodyLocation, Equipped, GivesImmunity},
    },
    engine::state::GameState,
    maps::zone::Zone,
    utils::common::Utils,
};

pub struct ItemEquipping {}

impl ItemEquipping {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let player_id = game_state
            .current_player_entity
            .expect("Player id should be set")
            .id();
        let mut item_to_equip_list: Vec<(Entity, BodyLocation, Entity, i32)> = Vec::new();
        let mut item_to_unequip_list: Vec<(Entity, Entity, i32)> = Vec::new();
        let mut cleanup_equip: Vec<Entity> = Vec::new();

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to equip items
            let mut items_to_equip = ecs_world.query::<(
                &WantsToEquip,
                &Position,
                &CombatStats,
                &mut Immunity,
                &Named,
            )>();
            let mut equipped_items = ecs_world.query::<(&Equipped, Option<&GivesImmunity>)>();

            //Log all the equipments

            let zone = game_state
                .current_zone
                .as_ref()
                .expect("must have Some Zone");

            for (equipper, (wants_to_equip, position, stats, current_immunities, named_equipper)) in
                &mut items_to_equip
            {
                // Show appropriate log messages
                let mut item_query = ecs_world
                    .query_one::<(&Named, Option<&Equipped>, Option<&GivesImmunity>)>(
                        wants_to_equip.item,
                    )
                    .expect("item_query failed");
                let (named_item, equipped, gives_immunity_opt) =
                    item_query.get().expect("item_query must have some");

                if equipped.is_some() {
                    // Unequip item
                    item_to_unequip_list.push((equipper, wants_to_equip.item, stats.speed));

                    // Remove any immunities given by the item
                    if let Some(given_immunities) = gives_immunity_opt {
                        for &immunity_type in given_immunities.to.iter() {
                            current_immunities
                                .to
                                .entry(immunity_type)
                                .and_modify(|v| *v -= 1);

                            if current_immunities.to[&immunity_type] == 0 {
                                current_immunities.to.remove(&immunity_type);
                            }
                        }
                        println!("Unequipped: New Immunities: {:?}", current_immunities.to);
                    }

                    if player_id == equipper.id() {
                        game_state
                            .game_log
                            .entries
                            .push(Cow::Owned(format!("You unequip the {}", named_item.name)));
                    }
                } else {
                    //Check if wants_item.body_location is already taken
                    let item_in_same_location: Option<(
                        Entity,
                        (&Equipped, Option<&GivesImmunity>),
                    )> = equipped_items.iter().find(|(_, (equipped, _))| {
                        equipped.owner.id() == equipper.id()
                            && Utils::occupies_same_location(
                                &equipped.body_location,
                                &wants_to_equip.body_location,
                            )
                    });

                    match item_in_same_location {
                        // Old item in same body location as new one
                        Some((item_to_remove, _)) => {
                            let named_item_to_remove = ecs_world
                                .get::<&Named>(item_to_remove)
                                .expect("Entity is not Named");
                            // Log to warning to Unequip item in same location and cleanup
                            cleanup_equip.push(equipper);

                            if player_id == equipper.id() {
                                game_state.game_log.add_entry(Cow::Owned(format!(
                                    "You must unequip the {} before equipping the {}",
                                    named_item_to_remove.name, named_item.name
                                )));
                            }
                        }
                        None => {
                            // Equip item
                            item_to_equip_list.push((
                                wants_to_equip.item,
                                wants_to_equip.body_location.clone(),
                                equipper,
                                stats.speed,
                            ));
                            // Add any immunities given by the item
                            if let Some(given_immunities) = gives_immunity_opt {
                                for &immunity_type in given_immunities.to.iter() {
                                    current_immunities
                                        .to
                                        .entry(immunity_type)
                                        .and_modify(|v| *v += 1)
                                        .or_insert(1);
                                }
                                println!("Equipped: New Immunities: {:?}", current_immunities.to);
                            }

                            if player_id == equipper.id() {
                                game_state
                                    .game_log
                                    .entries
                                    .push(Cow::Owned(format!("You equip the {}", named_item.name)));
                            } else if zone.visible_tiles
                                [Zone::get_index_from_xy(&position.x, &position.y)]
                            {
                                game_state.game_log.add_entry(Cow::Owned(format!(
                                    "{} equips the {}",
                                    named_equipper.name, named_item.name
                                )));
                            }
                        }
                    }
                }
            }
        }

        for (item, body_location, equipper, speed) in item_to_equip_list {
            // Remove owner's will to equip
            let _ = ecs_world.remove_one::<WantsToEquip>(equipper);

            // Equip at specified location
            let _ = ecs_world.insert_one(
                item,
                Equipped {
                    owner: equipper,
                    body_location,
                },
            );

            Utils::wait_after_action(ecs_world, equipper, speed);
        }

        for (unequipper, item, speed) in item_to_unequip_list {
            // Unequip and remove owner's will to equip
            let _ = ecs_world.remove_one::<WantsToEquip>(unequipper);
            let _ = ecs_world.remove_one::<Equipped>(item);

            Utils::wait_after_action(ecs_world, unequipper, speed);
        }

        for to_clean in cleanup_equip {
            // Remove owner's will to equip
            let _ = ecs_world.remove_one::<WantsToEquip>(to_clean);
        }
    }
}
