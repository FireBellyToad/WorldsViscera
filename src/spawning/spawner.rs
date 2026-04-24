use std::cmp::max;
use std::collections::HashMap;

use crate::components::combat::{CombatStats, SufferingDamage};
use crate::components::common::{
    BlocksTile, CanListen, CanSmell, DigProductEnum, Diggable, Experience, Immunity, Inspectable,
    Lock, MyTurn, Named, Position, ProduceSound, Renderable, SmellIntensity, Smellable, Species,
    SpeciesEnum, Viewshed,
};
use crate::components::health::{CanAutomaticallyHeal, DiseaseType, Hunger, Thirst};
use crate::components::items::{
    Corpse, Deadly, Edible, Item, Perishable, Poisonous, ProduceLight, Quaffable, Rotten, TurnedOn,
};
use crate::components::monster::DiseaseBearer;
use crate::components::player::Player;
use crate::constants::*;
use crate::maps::zone::{TileType, Zone};
use crate::systems::hunger_check::HungerStatus;
use crate::systems::thirst_check::ThirstStatus;
use crate::utils::assets::TextureName;
use crate::utils::roll::Roll;
use hecs::{Entity, World};
use macroquad::math::Rect;

pub struct CorpseSpawnData {
    pub x: i32,
    pub y: i32,
    pub name: &'static str,
    pub edible: Edible,
    pub is_venomous: bool,
    pub is_deadly: bool,
    pub disease_type_opt: Option<DiseaseType>,
    pub is_undead: bool,
}

/// Spawner of game entities
pub struct Spawn {}

impl Spawn {
    /// Spawn player
    pub fn player(ecs_world: &mut World, zone: &Zone) -> Entity {
        // Roll appropriate stats
        let rolled_toughness = Roll::stat();
        let rolled_dexterity = Roll::stat();
        // TODO Player with Soldier background must have 5+2d3 starting stamina
        let rolled_stamina = Roll::d6() + 5;

        let (spawn_x, spawn_y) = Zone::get_xy_from_index(zone.player_spawn_point);

        let player_components = (
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
                name: "Player",
                attack_verb: Some("hit"),
            },
            CombatStats {
                level: 1,
                current_stamina: rolled_stamina,
                max_stamina: rolled_stamina,
                base_armor: 0,
                unarmed_attack_dice: 2,
                current_toughness: rolled_toughness,
                max_toughness: rolled_toughness,
                current_dexterity: rolled_dexterity,
                max_dexterity: rolled_dexterity,
                speed: NORMAL,
            },
            SufferingDamage {
                damage_received: 0,
                toughness_damage_received: 0,
                dexterity_damage_received: 0,
                damager: None,
            },
            CanAutomaticallyHeal { tick_counter: 0 },
            Hunger {
                tick_counter: MAX_HUNGER_TICK_COUNTER,
                current_status: HungerStatus::Satiated,
            },
            Thirst {
                tick_counter: MAX_THIRST_TICK_COUNTER,
                current_status: ThirstStatus::Quenched,
            },
            MyTurn {},
            CanSmell {
                intensity: SmellIntensity::Faint,
                radius: PLAYER_SMELL_RADIUS,
            },
            Smellable {
                intensity: SmellIntensity::Faint,
                smell_log: Some("yourself"),
            },
            CanListen {
                listen_cache: HashMap::new(),
                radius: PLAYER_LISTEN_RADIUS,
                cooldown: 0,
            },
            Species {
                value: SpeciesEnum::Human,
            },
        );

        let player_entity = ecs_world.spawn(player_components);

        let _ = ecs_world.insert(
            player_entity,
            (
                Experience {
                    value: 0,
                    auto_advance_counter: 0,
                },
                BlocksTile {},
                Immunity { to: HashMap::new() },
            ),
        );

        player_entity
    }

    /// Spawn entities inside a room
    pub fn everyhing_in_map(ecs_world: &mut World, zone: &Zone) {
        // Actually spawn the monsters
        for &index in zone.monster_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);

            if zone.water_tiles[index] {
                Spawn::random_water_monster(ecs_world, x, y, zone.depth);
            } else {
                Spawn::random_terrain_monster(ecs_world, x, y, zone);
            }
        }
        // Actually spawn the items
        for &index in zone.item_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);
            Spawn::random_item(ecs_world, x, y);
        }
        // Actually spawn the fauna
        for &index in zone.fauna_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);
            Spawn::random_fauna(ecs_world, x, y);
        }

        // Spawn special entities
        for (index, tile) in zone.tiles.iter().enumerate() {
            let (x, y) = Zone::get_xy_from_index(index);
            Spawn::tile_entity(ecs_world, x, y, tile);
        }
    }

    /// Spawn a random terrainmonster
    pub fn random_terrain_monster(ecs_world: &mut World, x: i32, y: i32, zone: &Zone) {
        let dice_roll = max(1, Roll::dice(1, 6) + zone.depth as i32);

        // Depth based spawn table, recursive if roll is too high
        match dice_roll {
            (1..=10) => match Roll::dice(1, 22) {
                (1..=5) => Spawn::giant_slug(ecs_world, x, y),
                (6..=8) => Spawn::giant_cockroach(ecs_world, x, y),
                (9..=11) => Spawn::deep_one(ecs_world, x, y),
                (12..=14) => Spawn::living_dead(ecs_world, x, y),
                (15..=17) => Spawn::living_filth(ecs_world, x, y),
                (18..=19) => Spawn::refugee(ecs_world, x, y),
                (20..=21) => Spawn::pseudoscorpion(ecs_world, x, y),
                22 => Spawn::moleman(ecs_world, x, y),
                _ => {}
            },
            (11..=20) => match Roll::dice(1, 27) {
                (1..=5) => Spawn::calcificator(ecs_world, x, y),
                (6..=8) => Spawn::centipede(ecs_world, x, y),
                (9..=11) => Spawn::gremlin(ecs_world, x, y),
                (12..=14) => Spawn::moleman(ecs_world, x, y),
                (15..=17) => Spawn::sulfuric_slug(ecs_world, x, y),
                (18..=19) => Spawn::refugee(ecs_world, x, y),
                (20..=21) => Spawn::bombardier_bettle(ecs_world, x, y),
                (22..=24) => {
                    let _ = Spawn::stonedust_cultist(ecs_world, x, y);
                }
                (25..=27) => Spawn::giant_trogloraptor(ecs_world, x, y),
                _ => {}
            },
            (21..) => match Roll::dice(1, 28) {
                (1..=4) => Spawn::gremlin(ecs_world, x, y),
                (5..=7) => Spawn::moleman(ecs_world, x, y),
                (8..=11) => {
                    let _ = Spawn::stonedust_acolyte(ecs_world, x, y);
                }
                (12..=14) => Spawn::enthropic_gremlin(ecs_world, x, y),
                (15..=17) => Spawn::abyssal_one(ecs_world, x, y),
                (18..=20) => Spawn::sulfuric_slug(ecs_world, x, y),
                (21..=22) => Spawn::refugee(ecs_world, x, y),
                (23..=24) => Spawn::living_fossil(ecs_world, x, y),
                (25..=26) => Spawn::scorpion(ecs_world, x, y),
                27 => Spawn::colossal_worm(ecs_world, x, y, zone),
                28 => Spawn::darkling(ecs_world, x, y),
                _ => {}
            },
            _ => {}
        }
    }

    /// Spawn a random water monster
    pub fn random_water_monster(ecs_world: &mut World, x: i32, y: i32, depth: u32) {
        let dice_roll = max(1, Roll::dice(1, 6) + depth as i32);

        // Depth based spawn table, recursive if roll is too high
        match dice_roll {
            (1..=4) => Spawn::cave_shrimp(ecs_world, x, y),
            (5..=9) => Spawn::cave_crab(ecs_world, x, y),
            (10..=12) => Spawn::freshwater_viperfish(ecs_world, x, y),
            (13..) => Spawn::random_water_monster(ecs_world, x, y, depth - 1),
            _ => {}
        }
    }

    /// Spawn a random monster
    pub fn random_item(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::dice(1, 26);
        match dice_roll {
            (1..=3) => Spawn::shiv(ecs_world, x, y),
            (4..=6) => Spawn::flask_of_oil(ecs_world, x, y),
            (7..=9) => {
                let _ = Spawn::slingshot_ammo(ecs_world, x, y);
            }
            10 | 11 => {
                let _ = Spawn::rockpick(ecs_world, x, y);
            }
            12 | 13 => {
                let _ = Spawn::slingshot(ecs_world, x, y);
            }
            14 | 15 => {
                let _ = Spawn::leather_armor(ecs_world, x, y);
            }
            16 | 17 => {
                let _ = Spawn::lantern(ecs_world, x, y);
            }
            18 | 19 => Spawn::leather_cap(ecs_world, x, y),
            20 => {
                let _ = Spawn::flask_of_water(ecs_world, x, y);
            }
            21 => {
                let _ = Spawn::pickaxe(ecs_world, x, y);
            }
            22 => {
                let _ = Spawn::crowssbow(ecs_world, x, y);
            }
            23 => {
                let _ = Spawn::crossbow_ammo(ecs_world, x, y);
            }
            24 => Spawn::breastplate(ecs_world, x, y),
            25 => {
                let _ = Spawn::wand(ecs_world, x, y);
            }
            26 => Spawn::helmet(ecs_world, x, y),
            27 => Spawn::curing_paste(ecs_world, x, y),
            28 => {
                let _ = Spawn::leather_shoes(ecs_world, x, y);
            }
            29 => Spawn::crampon_boots(ecs_world, x, y),

            _ => {}
        };
    }

    /// Spawn random fauna
    pub fn random_fauna(ecs_world: &mut World, x: i32, y: i32) {
        // TODO Expand this function to spawn more types of fauna
        Spawn::random_mushroom(ecs_world, x, y);
    }

    /// Spawn a corpse
    pub fn corpse(ecs_world: &mut World, data: CorpseSpawnData) {
        let item_tile_index = (0, 0);
        let corpse = (
            Position {
                x: data.x,
                y: data.y,
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
                name: data.name,
                attack_verb: None,
            },
            Item {
                item_tile: item_tile_index,
            },
            data.edible,
            Corpse {},
            Perishable {
                rot_counter: STARTING_ROT_COUNTER + Roll::d20(),
            },
        );

        let corpse_spawned = ecs_world.spawn(corpse);

        if data.is_venomous {
            let _ = ecs_world.insert_one(corpse_spawned, Poisonous {});
        } else if data.is_deadly {
            let _ = ecs_world.insert_one(corpse_spawned, Deadly {});
        } else if let Some(disease_type) = data.disease_type_opt {
            let _ = ecs_world.insert_one(corpse_spawned, DiseaseBearer { disease_type });
        }

        if data.is_undead {
            let _ = ecs_world.insert(
                corpse_spawned,
                (
                    Rotten {},
                    Smellable {
                        intensity: SmellIntensity::Strong,
                        smell_log: Some(data.name),
                    },
                ),
            );
        }
    }

    /// Spawn special tile entitie
    pub fn tile_entity(ecs_world: &mut World, x: i32, y: i32, tile: &TileType) -> Option<Entity> {
        match tile {
            TileType::Brazier => Some(ecs_world.spawn((
                Position { x, y },
                ProduceLight {
                    radius: BRAZIER_RADIUS,
                },
                Smellable {
                    smell_log: Some("burning chemicals"),
                    intensity: SmellIntensity::Strong,
                },
                ProduceSound {
                    sound_log: "fire burning",
                },
                Inspectable {
                    description: "You see an everlasting\nalchemical brazier of\nhuman manifacture",
                    despawn_on_inspect: false,
                },
                TurnedOn {},
            ))),
            TileType::DownPassage => Some(ecs_world.spawn((
                Position { x, y },
                ProduceSound {
                    sound_log: "breeze from below",
                },
            ))),
            TileType::CrackedWall => Some(ecs_world.spawn((
                Position { x, y },
                Diggable {
                    dig_points: Roll::dice(4, 10),
                    produces: DigProductEnum::Stone,
                },
                Inspectable {
                    description: "You see a crack in\nthis stone wall.\nYou believe you can dig it\nwith an appropriate tool",
                    despawn_on_inspect: false,
                },
            ))),
            TileType::GoldMine => Some(ecs_world.spawn((
                Position { x, y },
                Diggable {
                    dig_points: Roll::dice(2, 20),
                    produces: DigProductEnum::Gold,
                },
                Inspectable {
                    description: "You see a some gold\nencrusted in the stone.\nYou believe you can\nextract some of it\nwith an appropriate tool",
                    despawn_on_inspect: false,
                },
            ))),
            TileType::BigCrystal => Some(ecs_world.spawn((
                Position { x, y },
                ProduceLight {
                    radius: CRYSTAL_LIGHT_RADIUS,
                },
                TurnedOn {},
            ))),
            TileType::TripleGoldLock(keys_to_unlock) => Some(ecs_world.spawn((
                Position { x, y },
                Lock {
                    keys_to_unlock: *keys_to_unlock as u8 + 1,
                },
                Inspectable {
                    description: "You see a\nweird golden pillar\nwith three circular holes.\nSeems like they could\naccommodate something", //TODO get message based on depth
                    despawn_on_inspect: false,
                },
            ))),
            TileType::CarvedStone => Some(ecs_world.spawn((
                Position { x, y },
                Inspectable {
                    description: "The message in the\ncarved stone says:\n\"You must go down\"",
                    despawn_on_inspect: false,
                },
            ))),
            TileType::DisembodiedEntity => Some(ecs_world.spawn((
                Position { x, y },
                Named {
                    name: "Disembodied entity",
                    attack_verb: None,
                },
                Inspectable {
                    description: "The disembodied entity says:\n\"You must go down\"",
                    despawn_on_inspect: true,
                },
            ))),
            _ => None,
        }
    }

    /// Generate ad hoc quaffable entity from lake
    pub fn river_water_entity(ecs_world: &mut World) -> Entity {
        ecs_world.spawn((
            Named {
                name: "River water",
                attack_verb: None,
            },
            Quaffable {
                thirst_dice_number: 2,
                thirst_dice_size: 20,
            },
        ))
    }
    /// Generate ad hoc quaffable entity from lake
    pub fn edible_stone(ecs_world: &mut World, dig_roll: i32) -> Entity {
        // Nasty hack: let's roll a number of d1 equal to the dig roll
        ecs_world.spawn((
            Named {
                name: "cracked wall",
                attack_verb: None,
            },
            Edible {
                nutrition_dice_number: dig_roll,
                nutrition_dice_size: 1,
            },
        ))
    }
}
