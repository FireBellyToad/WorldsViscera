use std::cmp::max;
use std::collections::HashMap;

use crate::components::combat::{CombatStats, SufferingDamage};
use crate::components::common::{
    CanListen, CanSmell, Diggable, Experience, MyTurn, Named, Position, ProduceSound, Renderable,
    SmellIntensity, Smellable, Species, SpeciesEnum, Viewshed,
};
use crate::components::health::{CanAutomaticallyHeal, DiseaseType, Hunger, Thirst};
use crate::components::items::{
    Corpse, Deadly, Edible, Item, Perishable, Poisonous, ProduceLight, Quaffable, TurnedOn,
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
                name: "Player".to_string(),
            },
            CombatStats {
                level: 1,
                current_stamina: rolled_stamina,
                max_stamina: rolled_stamina,
                base_armor: 10,
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
                smell_log: "yourself".to_string(),
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
            (Experience {
                value: 0,
                auto_advance_counter: 0,
            },),
        );
    }

    /// Spawn entities inside a room
    pub fn everyhing_in_map(ecs_world: &mut World, zone: &Zone) {
        // Actually spawn the monsters
        for &index in zone.monster_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);

            if zone.water_tiles[index] {
                Spawn::random_water_monster(ecs_world, x, y, zone.depth);
            } else {
                Spawn::random_terrain_monster(ecs_world, x, y, zone.depth);
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
    pub fn random_terrain_monster(ecs_world: &mut World, x: i32, y: i32, depth: u32) {
        let dice_roll = max(1, Roll::dice(1, 10) + depth as i32);

        // Depth based spawn table, recursive if roll is too high
        match dice_roll {
            (1..=5) => Spawn::giant_slug(ecs_world, x, y),
            (6..=9) => Spawn::giant_cockroach(ecs_world, x, y),
            (10..=12) => Spawn::deep_one(ecs_world, x, y),
            13 => Spawn::calcificator(ecs_world, x, y),
            14 => Spawn::centipede(ecs_world, x, y),
            15 => Spawn::gremlin(ecs_world, x, y),
            16 => Spawn::moleman(ecs_world, x, y),
            17 => Spawn::sulfuric_slug(ecs_world, x, y),
            18 => Spawn::abyssal_one(ecs_world, x, y),
            _ => Spawn::random_terrain_monster(ecs_world, x, y, depth - 1),
        }
    }

    /// Spawn a random water monster
    pub fn random_water_monster(ecs_world: &mut World, x: i32, y: i32, depth: u32) {
        let dice_roll = max(1, Roll::dice(1, 6) + depth as i32);

        // Depth based spawn table, recursive if roll is too high
        match dice_roll {
            (1..=4) => Spawn::cave_shrimp(ecs_world, x, y),
            (5..=9) => Spawn::freshwater_viperfish(ecs_world, x, y),
            _ => Spawn::random_water_monster(ecs_world, x, y, depth - 1),
        }
    }

    /// Spawn a random monster
    pub fn random_item(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::dice(1, 26);
        match dice_roll {
            (1..=3) => Spawn::shiv(ecs_world, x, y),
            (4..=6) => Spawn::flask_of_oil(ecs_world, x, y),
            (7..=9) => Spawn::slingshot_ammo(ecs_world, x, y),
            10 | 11 => {
                let _ = Spawn::rockpick(ecs_world, x, y);
            }
            12 | 13 => {
                let _ = Spawn::slingshot(ecs_world, x, y);
            }
            14 | 15 => Spawn::leather_armor(ecs_world, x, y),
            16 | 17 => Spawn::lantern(ecs_world, x, y),
            18 | 19 => Spawn::leather_cap(ecs_world, x, y),
            20 => Spawn::flask_of_water(ecs_world, x, y),
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
            _ => {}
        };
    }

    /// Spawn random fauna
    pub fn random_fauna(ecs_world: &mut World, x: i32, y: i32) {
        // TODO Expand this function to spawn more types of fauna
        Spawn::random_mushroom(ecs_world, x, y);
    }

    /// Spawn a corpse
    pub fn corpse(
        ecs_world: &mut World,
        x: i32,
        y: i32,
        name: String,
        edible: Edible,
        is_venomous: bool,
        deadly: bool,
        disease_type_opt: Option<DiseaseType>,
    ) {
        let item_tile_index = (0, 0);
        let corpse = (
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
                name: format!("{} corpse", name),
            },
            Item {
                item_tile: item_tile_index,
            },
            edible,
            Corpse {},
            Perishable {
                rot_counter: STARTING_ROT_COUNTER + Roll::d20(),
            },
        );

        let corpse_spawned = ecs_world.spawn(corpse);

        if is_venomous {
            let _ = ecs_world.insert_one(corpse_spawned, Poisonous {});
        } else if deadly {
            let _ = ecs_world.insert_one(corpse_spawned, Deadly {});
        } else if let Some(disease_type) = disease_type_opt {
            let _ = ecs_world.insert_one(corpse_spawned, DiseaseBearer { disease_type });
        }
    }

    /// Spawn special tile entities
    #[allow(clippy::single_match)]
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
                        smell_log: "burning chemicals".to_string(),
                        intensity: SmellIntensity::Strong,
                    },
                    ProduceSound {
                        sound_log: "fire burning".to_string(),
                    },
                    TurnedOn {},
                ));
            }
            TileType::DownPassage => {
                ecs_world.spawn((
                    true,
                    Position { x, y },
                    ProduceSound {
                        sound_log: "breeze from below".to_string(),
                    },
                ));
            }
            TileType::CrackedWall => {
                ecs_world.spawn((
                    true,
                    Position { x, y },
                    Diggable {
                        dig_points: Roll::dice(4, 10),
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
                name: "River water".to_string(),
            },
            Quaffable {
                thirst_dice_number: 2,
                thirst_dice_size: 20,
            },
        ))
    }
}
