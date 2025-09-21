use hecs::World;
use macroquad::math::Rect;

use crate::{
    components::{
        combat::InflictsDamage,
        common::{Named, Position, Renderable, SmellIntensity, Smellable},
        items::{
            Appliable, BodyLocation, Deadly, Edible, Equippable, Invokable, InvokablesEnum, Item, MustBeFueled, ProduceLight, Quaffable, Refiller, ToBeHarvested, TurnedOff, TurnedOn, Unsavoury, Weapon
        },
    },
    constants::*,
    utils::{assets::TextureName, roll::Roll},
};

pub fn mushroom(ecs_world: &mut World, x: i32, y: i32) {
    let index = Roll::dice(1, 10) - 1;
    let mushroom_type = MUSHROOM_SPAWN_MAP[index as usize];

    let common_components = (
        Item {
            item_tile: (mushroom_type, 1),
        },
        Position { x, y },
        Renderable {
            texture_name: TextureName::Items,
            texture_region: Rect {
                x: (mushroom_type * TILE_SIZE) as f32,
                y: TILE_SIZE_F32,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 0,
        },
        Smellable {
            smell_log: "mushrooms".to_string(),
            intensity: SmellIntensity::Faint,
        },
        ToBeHarvested {},
    );

    let mushroom_entity = ecs_world.spawn(common_components);

    match mushroom_type {
        MUSHROOM_EXCELLENT => {
            let _ = ecs_world.insert(
                mushroom_entity,
                (
                    Edible {
                        nutrition_dice_number: 5,
                        nutrition_dice_size: 20,
                    },
                    Named {
                        name: "brown mushroom".to_string(),
                    },
                ),
            );
        }
        MUSHROOM_MEDIOCRE => {
            let _ = ecs_world.insert(
                mushroom_entity,
                (
                    Edible {
                        nutrition_dice_number: 1,
                        nutrition_dice_size: 20,
                    },
                    Named {
                        name: "tuft of tiny mushrooms".to_string(),
                    },
                ),
            );
        }
        MUSHROOM_POISONOUS => {
            let _ = ecs_world.insert(
                mushroom_entity,
                (
                    Edible {
                        nutrition_dice_number: 1,
                        nutrition_dice_size: 1,
                    },
                    Unsavoury {
                        game_log: "poisonous".to_string(),
                    },
                    Named {
                        name: "white-spotted red mushroom".to_string(),
                    },
                ),
            );
        }
        MUSHROOM_DEADLY => {
            let _ = ecs_world.insert(
                mushroom_entity,
                (
                    Edible {
                        nutrition_dice_number: 1,
                        nutrition_dice_size: 1,
                    },
                    Deadly {},
                    Named {
                        name: "white mushroom".to_string(),
                    },
                ),
            );
        }
        MUSHROOM_LUMINESCENT => {
            let _ = ecs_world.insert(
                mushroom_entity,
                (
                    Edible {
                        nutrition_dice_number: 1,
                        nutrition_dice_size: 20,
                    },
                    ProduceLight {
                        radius: MUSHROOM_LIGHT_RADIUS,
                    },
                    TurnedOn {},
                    Named {
                        name: "glowing mushroom".to_string(),
                    },
                ),
            );
        }
        _ => {}
    }
}

// TODO unused... keep in mind
#[allow(dead_code)]
pub fn flask_of_water(ecs_world: &mut World, x: i32, y: i32) {
    let item_tile_index = (2, 0);
    let flask_of_water = (
        Position { x, y },
        Renderable {
            texture_name: TextureName::Items,
            texture_region: Rect {
                x: (item_tile_index.0 * TILE_SIZE) as f32,
                y: (item_tile_index.1 * TILE_SIZE) as f32,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 0,
        },
        Named {
            name: "flask of water".to_string(),
        },
        Item {
            item_tile: item_tile_index,
        },
        Quaffable {
            thirst_dice_number: 4,
            thirst_dice_size: 20,
        },
    );

    ecs_world.spawn(flask_of_water);
}

pub fn lantern(ecs_world: &mut World, x: i32, y: i32) {
    let item_tile_index = (3, 0);
    let lantern = (
        Position { x, y },
        Renderable {
            texture_name: TextureName::Items,
            texture_region: Rect {
                x: (item_tile_index.0 * TILE_SIZE) as f32,
                y: (item_tile_index.1 * TILE_SIZE) as f32,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 0,
        },
        Named {
            name: "lantern".to_string(),
        },
        Item {
            item_tile: item_tile_index,
        },
        ProduceLight {
            radius: LANTERN_RADIUS,
        },
        MustBeFueled {
            fuel_counter: STARTING_FUEL + Roll::d100(),
        },
        Smellable {
            smell_log: "a scent of burning fuel".to_string(),
            intensity: SmellIntensity::Faint,
        },
        TurnedOff {},
        Appliable{}
    );

    ecs_world.spawn(lantern);
}

pub fn wand(ecs_world: &mut World, x: i32, y: i32) {
    let item_tile_index = (1, 0);
    let wand = (
        Position { x, y },
        Renderable {
            texture_name: TextureName::Items,
            texture_region: Rect {
                x: (item_tile_index.0 * TILE_SIZE) as f32,
                y: (item_tile_index.1 * TILE_SIZE) as f32,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 0,
        },
        Named {
            name: "lightning wand".to_string(),
        },
        Item {
            item_tile: item_tile_index,
        },
        Invokable {
            invokable_type: InvokablesEnum::LightningWand,
        },
        InflictsDamage {
            number_of_dices: 2,
            dice_size: 4,
        },
        Smellable {
            smell_log: "ozone".to_string(),
            intensity: SmellIntensity::Faint,
        },
    );

    ecs_world.spawn(wand);
}

pub fn flask_of_oil(ecs_world: &mut World, x: i32, y: i32) {
    let item_tile_index = (4, 0);
    let flask_of_oil = (
        Position { x, y },
        Renderable {
            texture_name: TextureName::Items,
            texture_region: Rect {
                x: (item_tile_index.0 * TILE_SIZE) as f32,
                y: (item_tile_index.1 * TILE_SIZE) as f32,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 0,
        },
        Named {
            name: "flask of oil".to_string(),
        },
        Item {
            item_tile: item_tile_index,
        },
        MustBeFueled {
            fuel_counter: STARTING_FUEL + Roll::d100(),
        },
        Refiller {},
        Smellable {
            smell_log: "a faint scent of fuel".to_string(),
            intensity: SmellIntensity::Faint,
        },
        Appliable{}
    );

    ecs_world.spawn(flask_of_oil);
}

pub fn shiv(ecs_world: &mut World, x: i32, y: i32) {
    let item_tile_index = (0, 2);
    let flask_of_oil = (
        Position { x, y },
        Renderable {
            texture_name: TextureName::Items,
            texture_region: Rect {
                x: (item_tile_index.0 * TILE_SIZE) as f32,
                y: (item_tile_index.1 * TILE_SIZE) as f32,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 0,
        },
        Named {
            name: "shiv".to_string(),
        },
        Item {
            item_tile: item_tile_index,
        },
        Equippable {
            body_location: BodyLocation::RightHand,
        },
        Weapon { attack_dice: 4 },
    );

    ecs_world.spawn(flask_of_oil);
}

pub fn rockpick(ecs_world: &mut World, x: i32, y: i32) {
    let item_tile_index = (1, 2);
    let flask_of_oil = (
        Position { x, y },
        Renderable {
            texture_name: TextureName::Items,
            texture_region: Rect {
                x: (item_tile_index.0 * TILE_SIZE) as f32,
                y: (item_tile_index.1 * TILE_SIZE) as f32,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 0,
        },
        Named {
            name: "rock pick".to_string(),
        },
        Item {
            item_tile: item_tile_index,
        },
        Equippable {
            body_location: BodyLocation::RightHand,
        },
        Weapon { attack_dice: 6 }
    );

    //TODO Bonus to climb while wielded

    ecs_world.spawn(flask_of_oil);
}

pub fn maul(ecs_world: &mut World, x: i32, y: i32) {
    let item_tile_index = (2, 2);
    let flask_of_oil = (
        Position { x, y },
        Renderable {
            texture_name: TextureName::Items,
            texture_region: Rect {
                x: (item_tile_index.0 * TILE_SIZE) as f32,
                y: (item_tile_index.1 * TILE_SIZE) as f32,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 0,
        },
        Named {
            name: "maul".to_string(),
        },
        Item {
            item_tile: item_tile_index,
        },
        Equippable {
            body_location: BodyLocation::Hands,
        },
        Weapon { attack_dice: 8 },
        Appliable{},
    );

    ecs_world.spawn(flask_of_oil);
}
