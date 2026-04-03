use crate::{
    components::{
        common::DigProductEnum,
        health::DiseaseType,
        items::{Ammo, AmmoType, Cure, DiggingTool, RangedWeapon},
    },
    spawning::spawner::Spawn,
};
use hecs::{Entity, World};
use macroquad::math::Rect;

use crate::{
    components::{
        combat::InflictsDamage,
        common::{Named, Position, Renderable, SmellIntensity, Smellable},
        items::{
            Appliable, Armor, BodyLocation, Bulky, Deadly, Edible, Equippable, Equipped,
            InBackback, Invokable, InvokablesEnum, Item, MeleeWeapon, Metallic, MustBeFueled,
            Poisonous, ProduceLight, Quaffable, Refiller, ToBeHarvested, TurnedOff, TurnedOn,
        },
    },
    constants::*,
    utils::{assets::TextureName, roll::Roll},
};

impl Spawn {
    pub fn mushroom(ecs_world: &mut World, x: i32, y: i32, mushroom_type: i32) {
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
                smell_log: Some("mushrooms"),
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
                            name: "brown mushroom",
                            attack_verb: None,
                        },
                    ),
                );
            }
            MUSHROOM_MEDIOCRE => {
                let _ = ecs_world.insert(
                    mushroom_entity,
                    (
                        Edible {
                            nutrition_dice_number: 4,
                            nutrition_dice_size: 6,
                        },
                        Named {
                            name: "tuft of tiny mushrooms",
                            attack_verb: None,
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
                        Poisonous {},
                        Named {
                            name: "white-spotted red mushroom",
                            attack_verb: None,
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
                            name: "white mushroom",
                            attack_verb: None,
                        },
                    ),
                );
            }
            MUSHROOM_LUMINESCENT => {
                let _ = ecs_world.insert(
                    mushroom_entity,
                    (
                        Edible {
                            nutrition_dice_number: 2,
                            nutrition_dice_size: 20,
                        },
                        ProduceLight {
                            radius: MUSHROOM_LIGHT_RADIUS,
                        },
                        TurnedOn {},
                        Named {
                            name: "glowing mushroom",
                            attack_verb: None,
                        },
                    ),
                );
            }
            MUSHROOM_LICHEN => {
                let _ = ecs_world.insert(
                    mushroom_entity,
                    (
                        Edible {
                            nutrition_dice_number: 3,
                            nutrition_dice_size: 10,
                        },
                        Named {
                            name: "lichen",
                            attack_verb: None,
                        },
                    ),
                );
                //Lichen does not rot after being harvested
                let _ = ecs_world.remove_one::<ToBeHarvested>(mushroom_entity);
            }
            _ => {}
        }
    }

    pub fn random_mushroom(ecs_world: &mut World, x: i32, y: i32) {
        let index = Roll::dice(1, MUSHROOM_SPAWN_MAP.len() as i32) - 1;
        let mushroom_type = MUSHROOM_SPAWN_MAP[index as usize];

        Spawn::mushroom(ecs_world, x, y, mushroom_type);
    }

    pub fn flask_of_water(ecs_world: &mut World, x: i32, y: i32) -> Entity {
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
                name: "flask of water",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Quaffable {
                thirst_dice_number: 4,
                thirst_dice_size: 20,
            },
        );

        ecs_world.spawn(flask_of_water)
    }

    pub fn curing_paste(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = (8, 0);
        let curing_paste = (
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
                name: "curing paste",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Appliable {
                application_time: VERY_LONG_ACTION_MULTIPLIER,
            },
            Cure {
                diseases: vec![DiseaseType::Calcification, DiseaseType::FleshRot],
            },
        );

        ecs_world.spawn(curing_paste);
    }

    pub fn ration(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let item_tile_index = (7, 0);
        let ration = (
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
                name: "ration",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Smellable {
                smell_log: Some("dry meat"),
                intensity: SmellIntensity::Faint,
            },
            Edible {
                nutrition_dice_number: 5,
                nutrition_dice_size: 20,
            },
        );

        ecs_world.spawn(ration)
    }

    pub fn lantern(ecs_world: &mut World, x: i32, y: i32) -> Entity {
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
                name: "lantern",
                attack_verb: None,
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
                smell_log: Some("a scent of burning fuel"),
                intensity: SmellIntensity::Faint,
            },
            TurnedOff {},
            Appliable {
                application_time: STANDARD_ACTION_MULTIPLIER,
            },
        );

        ecs_world.spawn(lantern)
    }

    pub fn wand(ecs_world: &mut World, x: i32, y: i32) -> Entity {
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
                name: "lightning wand",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Invokable {
                invokable_type: InvokablesEnum::LightningWand,
            },
            InflictsDamage {
                number_of_dices: 2,
                dice_size: 8,
            },
            Smellable {
                smell_log: Some("ozone"),
                intensity: SmellIntensity::Faint,
            },
        );

        ecs_world.spawn(wand)
    }

    pub fn crowssbow(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let item_tile_index = (3, 2);
        let crowssbow = (
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
                name: "crossbow",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::BothHands,
            },
            RangedWeapon {
                ammo_type: AmmoType::Crossbow,
                ammo_count_total: 0,
            },
            InflictsDamage {
                number_of_dices: 1,
                dice_size: 8,
            },
            Metallic {},
        );

        ecs_world.spawn(crowssbow)
    }

    pub fn slingshot(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let item_tile_index = (4, 2);
        let slingshot = (
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
                name: "slingshot",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::BothHands,
            },
            RangedWeapon {
                ammo_type: AmmoType::Slingshot,
                ammo_count_total: 0,
            },
            InflictsDamage {
                number_of_dices: 1,
                dice_size: 4,
            },
        );

        ecs_world.spawn(slingshot)
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
                name: "flask of oil",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Refiller {
                fuel_counter: STARTING_FUEL + Roll::d100(),
            },
            Smellable {
                smell_log: Some("a faint scent of fuel"),
                intensity: SmellIntensity::Faint,
            },
            Appliable {
                application_time: STANDARD_ACTION_MULTIPLIER,
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
                name: "shiv",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::RightHand,
            },
            MeleeWeapon {},
            InflictsDamage {
                number_of_dices: 1,
                dice_size: 4,
            },
        );

        ecs_world.spawn(flask_of_oil);
    }

    pub fn rockpick(ecs_world: &mut World, x: i32, y: i32) -> Entity {
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
                name: "rock pick",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::RightHand,
            },
            MeleeWeapon {},
            InflictsDamage {
                number_of_dices: 1,
                dice_size: 6,
            },
            Metallic {},
            DiggingTool {},
        );

        //TODO Bonus to climb while wielded

        ecs_world.spawn(flask_of_oil)
    }

    pub fn pickaxe(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let item_tile_index = (2, 2);
        let pickaxe = (
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
                name: "pickaxe",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::BothHands,
            },
            MeleeWeapon {},
            InflictsDamage {
                number_of_dices: 1,
                dice_size: 10,
            },
            Bulky {},
            Metallic {},
            DiggingTool {},
        );

        ecs_world.spawn(pickaxe)
    }

    pub fn leather_armor(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let item_tile_index = (0, 3);
        let leather_armor = (
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
                name: "leather armor",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::Torso,
            },
            Armor { value: 1 },
        );

        ecs_world.spawn(leather_armor)
    }

    pub fn breastplate(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = (1, 3);
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
                name: "breastplate",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::Torso,
            },
            Armor { value: 3 },
            Bulky {},
            Metallic {},
        );

        ecs_world.spawn(flask_of_oil);
    }

    pub fn moleman_chain(ecs_world: &mut World, owner: Entity) {
        let item_tile_index = (2, 3);
        let dvergar_chain = (
            InBackback {
                owner,
                assigned_char: 'a',
            },
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
                name: "Mole-man chainmail",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::Torso,
            },
            Equipped {
                owner,
                body_location: BodyLocation::Torso,
            },
            Armor { value: 2 },
            Bulky {},
            Metallic {},
        );

        ecs_world.spawn(dvergar_chain);
    }

    pub fn crossbow_ammo(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let item_tile_index = (5, 0);
        let crossbow_ammo = (
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
                name: "bag of bolts",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Ammo {
                ammo_type: AmmoType::Crossbow,
                ammo_count: Roll::dice(2, 6) as u32,
            },
        );

        ecs_world.spawn(crossbow_ammo)
    }

    pub fn slingshot_ammo(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let item_tile_index = (6, 0);
        let slingshot_ammo = (
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
                name: "pile of stones",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Ammo {
                ammo_type: AmmoType::Slingshot,
                ammo_count: Roll::dice(2, 8) as u32,
            },
        );

        ecs_world.spawn(slingshot_ammo)
    }

    pub fn leather_cap(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = (0, 4);
        let leather_cap = (
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
                name: "leather cap",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::Head,
            },
            Armor { value: 1 },
        );

        ecs_world.spawn(leather_cap);
    }

    pub fn helmet(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = (1, 4);
        let helmet = (
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
                name: "helmet",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            Equippable {
                body_location: BodyLocation::Head,
            },
            Armor { value: 2 },
            Metallic {},
        );

        ecs_world.spawn(helmet);
    }

    //TODO improve avoiding preassigned characters
    pub fn give_crossbow_and_ammo(ecs_world: &mut World, entity: Entity) {
        let crosswbow = Spawn::crowssbow(ecs_world, 0, 0);
        let _ = ecs_world.remove_one::<Position>(crosswbow);
        let _ = ecs_world.insert(
            crosswbow,
            (
                InBackback {
                    owner: entity,
                    assigned_char: 'b',
                },
                Equipped {
                    owner: entity,
                    body_location: BodyLocation::BothHands,
                },
            ),
        );

        // Give the farmer some ammo
        for _ in 0..3 {
            let crosswbow_ammo = Spawn::crossbow_ammo(ecs_world, 0, 0);
            let _ = ecs_world.remove_one::<Position>(crosswbow_ammo);
            let _ = ecs_world.insert(
                crosswbow_ammo,
                (InBackback {
                    owner: entity,
                    assigned_char: 'c',
                },),
            );
        }
    }

    //TODO improve avoiding preassigned characters
    pub fn give_slingshot_and_ammo(ecs_world: &mut World, entity: Entity) {
        let slingshot = Spawn::slingshot(ecs_world, 0, 0);
        let _ = ecs_world.remove_one::<Position>(slingshot);
        let _ = ecs_world.insert(
            slingshot,
            (
                InBackback {
                    owner: entity,
                    assigned_char: 'b',
                },
                Equipped {
                    owner: entity,
                    body_location: BodyLocation::BothHands,
                },
            ),
        );

        // Give the farmer some ammo
        for _ in 0..2 {
            let slingshot_ammo = Spawn::slingshot_ammo(ecs_world, 0, 0);
            let _ = ecs_world.remove_one::<Position>(slingshot_ammo);
            let _ = ecs_world.insert(
                slingshot_ammo,
                (InBackback {
                    owner: entity,
                    assigned_char: 'c',
                },),
            );
        }
    }

    pub fn raw_gold(ecs_world: &mut World, x: i32, y: i32) -> Entity {
        let item_tile_index = (0, 5);
        let raw_gold = (
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
                name: "raw gold",
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            DigProductEnum::Gold,
        );

        ecs_world.spawn(raw_gold)
    }
}
