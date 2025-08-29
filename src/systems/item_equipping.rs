use hecs::{Entity, World};

use crate::{
    components::{
        actions::WantsToEquip,
        common::{GameLog, Named},
        items::{BodyLocation, Equipped},
        player::Player,
    },
    utils::common::Utils,
};

pub struct ItemEquipping {}

impl ItemEquipping {
    pub fn run(ecs_world: &mut World) {
        let mut item_to_equip_list: Vec<(Entity, BodyLocation, Entity)> = Vec::new();
        let mut item_to_unequip_list: Vec<(Entity, Entity)> = Vec::new();

        let player_id = Player::get_entity_id(ecs_world);

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to equip items
            let mut items_to_equip = ecs_world.query::<&WantsToEquip>();
            let mut equipped_items = ecs_world.query::<&Equipped>();

            //Log all the drop downs
            let mut game_log_query = ecs_world.query::<&mut GameLog>();
            let (_e, game_log) = game_log_query
                .iter()
                .last()
                .expect("Game log is not in hecs::World");

            for (equipper, wants_to_equip) in &mut items_to_equip {
                // Show appropriate log messages
                let named_item: hecs::Ref<'_, Named> =
                    ecs_world.get::<&Named>(wants_to_equip.item).unwrap();
                let is_already_equipped = ecs_world.get::<&Equipped>(wants_to_equip.item).is_ok();

                if is_already_equipped {
                    // Unequip item
                    item_to_unequip_list.push((equipper, wants_to_equip.item));

                    if player_id == equipper.id() {
                        game_log
                            .entries
                            .push(format!("You unequip the {}", named_item.name));
                    }
                } else {
                    let named_dropper = ecs_world.get::<&Named>(equipper).unwrap();
                    //TODO check if wants_item.body_location is already taken?

                    // Drop item and keep track of the drop Position
                    item_to_equip_list.push((
                        wants_to_equip.item,
                        wants_to_equip.body_location.clone(),
                        equipper,
                    ));

                    let item_in_same_location: Option<(Entity, &Equipped)> =
                        equipped_items.iter().find(|(_e, equipped)| {
                            equipped.owner.id() == equipper.id()
                                && Utils::has_same_location(
                                    &equipped.body_location,
                                    &wants_to_equip.body_location,
                                )
                        });
                    if item_in_same_location.is_some() {
                        let (item_to_remove, _) = item_in_same_location.unwrap();
                        // Unequip item in same location
                        item_to_unequip_list.push((equipper, item_to_remove));

                        if player_id == equipper.id() {
                            game_log
                                .entries
                                .push(format!("You unequip the {}", named_item.name));
                        }
                    }

                    if player_id == equipper.id() {
                        game_log
                            .entries
                            .push(format!("You equip the {}", named_item.name));
                    } else {
                        game_log.entries.push(format!(
                            "{} equips the {}",
                            named_dropper.name, named_item.name
                        ));
                    }
                }
            }
        }

        for (item, body_location, equipper) in item_to_equip_list {
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
        }

        for (unequipper, item) in item_to_unequip_list {
            // Unequip
            let _ = ecs_world.remove_one::<WantsToEquip>(unequipper);
            let _ = ecs_world.remove_one::<Equipped>(item);
        }
    }
}
