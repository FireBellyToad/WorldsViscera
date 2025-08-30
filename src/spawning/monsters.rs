use hecs::{Entity, World};
use macroquad::math::Rect;

use crate::{
    components::{
        combat::{CanHide, CombatStats, SufferingDamage},
        common::{
            BlocksTile, MyTurn, Named, Position, ProduceCorpse, Renderable, SmellIntensity,
            Smellable, Viewshed,
        },
        monster::{Aquatic, Monster},
    },
    constants::{BASE_MONSTER_VIEW_RADIUS, FAST, NORMAL, SLOW, TILE_SIZE_F32},
    utils::assets::TextureName,
};

pub fn deep_one(ecs_world: &mut World, x: i32, y: i32) {
    create_monster(
        ecs_world,
        String::from("Deep One"),
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
        Smellable {
            smell_log: String::from("dried human sweat"),
            intensity: SmellIntensity::Faint,
        },
        1.0,
        x,
        y,
    );
}

pub fn freshwater_viperfish(ecs_world: &mut World, x: i32, y: i32) {
    let freshwater_viperfish = create_monster(
        ecs_world,
        String::from("Freshwater viperfish"),
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
        Smellable {
            smell_log: String::from("fish"),
            intensity: SmellIntensity::None,
        },
        4.0,
        x,
        y,
    );

    let _ = ecs_world.insert(freshwater_viperfish, (Aquatic {}, CanHide { cooldown: 0 }));
}

pub fn gremlin(ecs_world: &mut World, x: i32, y: i32) {
    create_monster(
        ecs_world,
        String::from("Gremlin"),
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
        Smellable {
            smell_log: String::from("cheap leather"),
            intensity: SmellIntensity::Faint,
        },
        3.0, 
        x,
        y,
    );
}

pub fn dvergar(ecs_world: &mut World, x: i32, y: i32) {
    create_monster(
        ecs_world,
        String::from("Dvergar"),
        CombatStats {
            current_stamina: 4,
            max_stamina: 4,
            base_armor: 2,
            unarmed_attack_dice: 6,
            current_toughness: 10,
            max_toughness: 10,
            current_dexterity: 8,
            max_dexterity: 8,
            speed: SLOW,
        },
        Smellable {
            smell_log: String::from("coal drenched in vinegar"),
            intensity: SmellIntensity::Faint,
        },
        2.0,
        x,
        y,
    );
}

/// Generic monster creation
pub fn create_monster(
    ecs_world: &mut World,
    name: String,
    combat_stats: CombatStats,
    smells: Smellable,
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
        Named { name: name },
        BlocksTile {},
        combat_stats,
        SufferingDamage { damage_received: 0 },
        ProduceCorpse {},
        MyTurn {},
        smells,
    );

    ecs_world.spawn(monster_entity)
}
