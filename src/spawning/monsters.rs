use hecs::{Entity, World};
use macroquad::math::Rect;

use crate::{
    components::{
        combat::{CanHide, CombatStats, SufferingDamage},
        common::{
            BlocksTile, MyTurn, Named, Position, ProduceCorpse, ProduceSound, Renderable,
            SmellIntensity, Smellable, Viewshed,
        },
        health::Hunger,
        items::Edible,
        monster::{Aquatic, IsSmart, Monster, Venomous},
    },
    constants::{
        BASE_MONSTER_VIEW_RADIUS, FAST, MAX_HUNGER_TICK_COUNTER, NORMAL, SLOW, TILE_SIZE_F32,
    },
    spawning::items::dvergar_chain,
    systems::hunger_check::HungerStatus,
    utils::assets::TextureName,
};

pub fn deep_one(ecs_world: &mut World, x: i32, y: i32) {
    create_monster(
        ecs_world,
        "Deep One".to_string(),
        CombatStats {
            current_stamina: 3,
            max_stamina: 3,
            base_armor: 0,
            unarmed_attack_dice: 3,
            current_toughness: 8,
            max_toughness: 8,
            current_dexterity: 10,
            max_dexterity: 10,
            speed: NORMAL,
        },
        Edible {
            nutrition_dice_number: 3,
            nutrition_dice_size: 6,
        },
        Smellable {
            smell_log: "dried human sweat".to_string(),
            intensity: SmellIntensity::Faint,
        },
        ProduceSound {
            sound_log: "someone weezing".to_string(),
        },
        1.0,
        x,
        y,
    );
}

pub fn freshwater_viperfish(ecs_world: &mut World, x: i32, y: i32) {
    let freshwater_viperfish = create_monster(
        ecs_world,
        "Freshwater viperfish".to_string(),
        CombatStats {
            current_stamina: 4,
            max_stamina: 4,
            base_armor: 0,
            unarmed_attack_dice: 4,
            current_toughness: 4,
            max_toughness: 4,
            current_dexterity: 14,
            max_dexterity: 14,
            speed: NORMAL,
        },
        Edible {
            nutrition_dice_number: 3,
            nutrition_dice_size: 4,
        },
        Smellable {
            smell_log: "fish".to_string(),
            intensity: SmellIntensity::None,
        },
        ProduceSound {
            sound_log: "a splash in the water".to_string(),
        },
        4.0,
        x,
        y,
    );

    let _ = ecs_world.insert(freshwater_viperfish, (Aquatic {}, CanHide { cooldown: 0 }));
}

pub fn gremlin(ecs_world: &mut World, x: i32, y: i32) {
    let gremlin = create_monster(
        ecs_world,
        "Gremlin".to_string(),
        CombatStats {
            current_stamina: 2,
            max_stamina: 2,
            base_armor: 0,
            unarmed_attack_dice: 2,
            current_toughness: 7,
            max_toughness: 7,
            current_dexterity: 14,
            max_dexterity: 14,
            speed: FAST,
        },
        Edible {
            nutrition_dice_number: 3,
            nutrition_dice_size: 4,
        },
        Smellable {
            smell_log: "cheap leather".to_string(),
            intensity: SmellIntensity::Faint,
        },
        ProduceSound {
            sound_log: "someone cackling".to_string(),
        },
        3.0,
        x,
        y,
    );

    let _ = ecs_world.insert_one(gremlin, IsSmart {});
}

pub fn centipede(ecs_world: &mut World, x: i32, y: i32) {
    let centipede = create_monster(
        ecs_world,
        "Giant centipede".to_string(),
        CombatStats {
            current_stamina: 3,
            max_stamina: 3,
            base_armor: 1,
            unarmed_attack_dice: 3,
            current_toughness: 6,
            max_toughness: 6,
            current_dexterity: 13,
            max_dexterity: 14,
            speed: NORMAL,
        },
        Edible {
            nutrition_dice_number: 2,
            nutrition_dice_size: 8,
        },
        Smellable {
            smell_log: "something off and dusty".to_string(),
            intensity: SmellIntensity::None,
        },
        ProduceSound {
            sound_log: "skittering of many legs".to_string(),
        },
        5.0,
        x,
        y,
    );

    let _ = ecs_world.insert_one(centipede, Venomous {});
}

pub fn dvergar(ecs_world: &mut World, x: i32, y: i32) {
    let dvergar = create_monster(
        ecs_world,
        "Dvergar".to_string(),
        CombatStats {
            current_stamina: 4,
            max_stamina: 4,
            base_armor: 0,
            unarmed_attack_dice: 6,
            current_toughness: 10,
            max_toughness: 10,
            current_dexterity: 8,
            max_dexterity: 8,
            speed: SLOW,
        },
        Edible {
            nutrition_dice_number: 4,
            nutrition_dice_size: 6,
        },
        Smellable {
            smell_log: "coal drenched in vinegar".to_string(),
            intensity: SmellIntensity::Faint,
        },
        ProduceSound {
            sound_log: "someone mumbling".to_string(),
        },
        2.0,
        x,
        y,
    );

    dvergar_chain(ecs_world, dvergar);
}

/// Generic monster creation
pub fn create_monster(
    ecs_world: &mut World,
    name: String,
    combat_stats: CombatStats,
    edible: Edible,
    smells: Smellable,
    sounds: ProduceSound,
    tile_index: f32,
    x: i32,
    y: i32,
) -> Entity {
    let monster_entity = (
        Monster {},
        Position { x, y },
        Renderable {
            texture_name: TextureName::Creatures,
            texture_region: Rect {
                x: tile_index * TILE_SIZE_F32,
                y: 0.0,
                w: TILE_SIZE_F32,
                h: TILE_SIZE_F32,
            },
            z_index: 1,
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: BASE_MONSTER_VIEW_RADIUS,
            must_recalculate: true,
        },
        Named { name },
        BlocksTile {},
        combat_stats,
        SufferingDamage {
            damage_received: 0,
            toughness_damage_received: 0,
        },
        ProduceCorpse {},
        Hunger {
            tick_counter: MAX_HUNGER_TICK_COUNTER,
            current_status: HungerStatus::Starved,
        },
        MyTurn {},
        smells,
        sounds,
        edible,
    );

    ecs_world.spawn(monster_entity)
}
