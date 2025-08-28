use crate::components::combat::{CanHide, CombatStats, InflictsDamage, SufferingDamage};
use crate::components::common::{
    BlocksTile, CanSmell, MyTurn, Named, Position, ProduceCorpse, Renderable, SmellIntensity,
    Smellable, Viewshed,
};
use crate::components::health::{CanAutomaticallyHeal, Hunger, Thirst};
use crate::components::items::{
    Deadly, Edible, Invokable, InvokablesEnum, Item, MustBeFueled, Perishable, ProduceLight,
    Quaffable, Refiller, Unsavoury,
};
use crate::components::monster::{Aquatic, Monster};
use crate::components::player::Player;
use crate::constants::*;
use crate::maps::zone::{TileType, Zone};
use crate::systems::hunger_check::HungerStatus;
use crate::systems::thirst_check::ThirstStatus;
use crate::utils::assets::TextureName;
use crate::utils::roll::Roll;
use hecs::{Entity, World};
use macroquad::math::Rect;

/// Spawner of game entities
pub struct Spawn {}

impl Spawn {
    /// Spawn player
    pub fn player(ecs_world: &mut World, zone: &Zone) {
        // Roll appropriate stats
        let rolled_toughness = Roll::stat();
        let rolled_dexterity = Roll::stat();
        // TODO Player with Soldier background must have 5+2d3 starting stamina
        let rolled_stamina = Roll::d6() + 5;

        let (spawn_x, spawn_y) = Zone::get_xy_from_index(zone.player_spawn_point);

        let player_entity = (
            Player {},
            Position {
                x: spawn_x,
                y: spawn_y,
            },
            Renderable {
                texture_name: TextureName::Creatures,
                texture_region: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: TILE_SIZE_F32,
                    h: TILE_SIZE_F32,
                },
                z_index: 1,
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: BASE_VIEW_RADIUS,
                must_recalculate: true,
            },
            Named {
                name: String::from("Player"),
            },
            CombatStats {
                current_stamina: rolled_stamina,
                max_stamina: rolled_stamina,
                base_armor: 0,
                unarmed_attack_dice: 4,
                current_toughness: rolled_toughness,
                max_toughness: rolled_toughness,
                current_dexterity: rolled_dexterity,
                max_dexterity: rolled_dexterity,
                speed: NORMAL,
            },
            SufferingDamage { damage_received: 0 },
            CanAutomaticallyHeal { tick_counter: 0 },
            Hunger {
                tick_counter: MAX_HUNGER_TICK_COUNTER,
                current_status: HungerStatus::Normal,
            },
            Thirst {
                tick_counter: MAX_THIRST_TICK_COUNTER,
                current_status: ThirstStatus::Normal,
            },
            MyTurn {},
            CanSmell {
                intensity: SmellIntensity::Faint,
                radius: PLAYER_SMELL_RADIUS,
            },
            Smellable {
                intensity: SmellIntensity::Faint,
                smell_log: String::from("yourself"),
            },
        );

        ecs_world.spawn(player_entity);
    }

    /// Spawn entities inside a room
    pub fn everyhing_in_map(ecs_world: &mut World, zone: &Zone) {
        // Actually spawn the monsters
        for &index in zone.monster_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);
            //TODO improve with spawn table
            if zone.water_tiles[index] {
                Spawn::freshwater_viperfish(ecs_world, x, y)
            } else {
                Spawn::random_terrain_monster(ecs_world, x as i32, y as i32);
            }
        }
        // Actually spawn the potions
        for &index in zone.item_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);
            Spawn::random_item(ecs_world, x as i32, y as i32);
        }

        // Spawn special entities
        for (index, tile) in zone.tiles.iter().enumerate() {
            let (x, y) = Zone::get_xy_from_index(index);
            Spawn::tile_entity(ecs_world, x as i32, y as i32, tile);
        }
    }

    /// Spawn a random monster
    pub fn random_terrain_monster(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::dice(1, 5);

        // Dvergar is stronger, shuold be less common
        match dice_roll {
            1 => Spawn::dvergar(ecs_world, x, y),
            2 => Spawn::gremlin(ecs_world, x, y),
            _ => Spawn::deep_one(ecs_world, x, y),
        }
    }

    fn deep_one(ecs_world: &mut World, x: i32, y: i32) {
        Spawn::create_monster(
            ecs_world,
            String::from("Deep One"),
            CombatStats {
                current_stamina: 3,
                max_stamina: 3,
                base_armor: 1,
                unarmed_attack_dice: 4,
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
            1.0, //TODO fix
            x,
            y,
        );
    }

    fn freshwater_viperfish(ecs_world: &mut World, x: i32, y: i32) {
        let freshwater_viperfish = Spawn::create_monster(
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
            4.0, //TODO fix
            x,
            y,
        );

        let _ = ecs_world.insert(freshwater_viperfish, (Aquatic {}, CanHide { cooldown: 0 }));
    }

    fn gremlin(ecs_world: &mut World, x: i32, y: i32) {
        Spawn::create_monster(
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
            3.0, //TODO fix
            x,
            y,
        );
    }

    fn dvergar(ecs_world: &mut World, x: i32, y: i32) {
        Spawn::create_monster(
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
            2.0, //TODO fix
            x,
            y,
        );
    }

    /// Generic monster creation
    fn create_monster(
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

    /// Spawn a random monster
    pub fn random_item(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::dice(1, 8);
        // Dvergar is stronger, shuold be less common
        match dice_roll {
            1 => Spawn::wand(ecs_world, x, y),
            2 => Spawn::lantern(ecs_world, x, y),
            3 => Spawn::flask_of_oil(ecs_world, x, y),
            _ => Spawn::mushroom(ecs_world, x, y),
        }
    }

    pub fn corpse(ecs_world: &mut World, x: i32, y: i32, name: String, edible: Edible) {
        let item_tile_index = (0,0);
        let meat = (
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
                name: String::from(format!("{} corpse", name)),
            },
            Item { item_tile: item_tile_index },
            edible,
            Perishable {
                rot_counter: STARTING_ROT_COUNTER + Roll::d20(),
            },
        );

        ecs_world.spawn(meat);
    }

    pub fn mushroom(ecs_world: &mut World, x: i32, y: i32) {
        let index = Roll::dice(1, 10) - 1;
        let mushroom_type = MUSHROOM_SPAWN_MAP[index as usize];

        let common_components = (
            Item {
                item_tile: (mushroom_type,1),
            }, // 4 is Item per row, TODO make constat
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
    fn flask_of_water(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = (2,0);
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
                name: String::from("Flask of water"),
            },
            Item { item_tile: item_tile_index },
            Quaffable {
                thirst_dice_number: 4,
                thirst_dice_size: 20,
            },
        );

        ecs_world.spawn(flask_of_water);
    }

    fn lantern(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = (3,0);
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
                name: String::from("Lantern"),
            },
            Item { item_tile: item_tile_index },
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

    fn wand(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = (1,0);
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
                name: String::from("Lightning wand"),
            },
            Item { item_tile: item_tile_index },
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

    fn flask_of_oil(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = (4,0);
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
                name: String::from("Flask of oil"),
            },
            Item { item_tile: item_tile_index },
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

    /// Spawn special tile entities
    fn tile_entity(ecs_world: &mut World, x: i32, y: i32, tile: &TileType) {
        match tile {
            TileType::Brazier => {
                ecs_world.spawn((
                    true,
                    Position { x, y },
                    ProduceLight {
                        radius: BRAZIER_RADIUS,
                    },
                    Smellable {
                        smell_log: String::from("burning chemicals"),
                        intensity: SmellIntensity::Strong,
                    },
                ));
            }
            _ => {}
        }
    }

    /// Generate ad hoc quaffable entity from lake
    pub fn river_water_entity(ecs_world: &mut World) -> Entity {
        ecs_world.spawn((
            Named {
                name: String::from("River water"),
            },
            Quaffable {
                thirst_dice_number: 2,
                thirst_dice_size: 20,
            },
        ))
    }
}
