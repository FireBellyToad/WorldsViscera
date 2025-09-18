use hecs::World;
use macroquad::math::Rect;

use crate::{
    components::{
        combat::InflictsDamage,
        common::{Named, Position, Renderable, SmellIntensity, Smellable},
        items::{
            BodyLocation, Deadly, Edible, Equippable, Invokable, InvokablesEnum, Item,
            MustBeFueled, ProduceLight, Quaffable, Refiller, ToBeHarvested, Unsavoury, Weapon,
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
            smell_log: String::from("mushrooms"),
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
                        name: String::from("brown mushroom"),
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
                        name: String::from("tuft of tiny mushrooms"),
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
                        game_log: String::from("poisonous"),
                    },
                    Named {
                        name: String::from("white-spotted red mushroom"),
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
                        name: String::from("white mushroom"),
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
                    Named {
                        name: String::from("glowing mushroom"),
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
            name: String::from("flask of water"),
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
            name: String::from("lantern"),
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
            smell_log: String::from("a scent of burning fuel"),
            intensity: SmellIntensity::Faint,
        },
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
            name: String::from("lightning wand"),
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
            smell_log: String::from("ozone"),
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
            name: String::from("flask of oil"),
        },
        Item {
            item_tile: item_tile_index,
        },
        MustBeFueled {
            fuel_counter: STARTING_FUEL + Roll::d100(),
        },
        Refiller {},
        Smellable {
            smell_log: String::from("a faint scent of fuel"),
            intensity: SmellIntensity::Faint,
        },
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
            name: String::from("shiv"),
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
            name: String::from("rock pick"),
        },
        Item {
            item_tile: item_tile_index,
        },
        Equippable {
            body_location: BodyLocation::RightHand,
        },
        Weapon { attack_dice: 6 },
    );

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
            name: String::from("maul"),
        },
        Item {
            item_tile: item_tile_index,
        },
        Equippable {
            body_location: BodyLocation::Hands,
        },
        Weapon { attack_dice: 8 },
    );

    ecs_world.spawn(flask_of_oil);
}
