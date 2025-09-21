use std::collections::HashMap;

use crate::components::combat::{CombatStats, SufferingDamage};
use crate::components::common::{
    CanListen, CanSmell, MyTurn, Named, Position, ProduceSound, Renderable, SmellIntensity,
    Smellable, Viewshed,
};
use crate::components::health::{CanAutomaticallyHeal, Hunger, Thirst};
use crate::components::items::{Edible, Item, Perishable, ProduceLight, Quaffable, TurnedOn};
use crate::components::player::Player;
use crate::constants::*;
use crate::maps::zone::{TileType, Zone};
use crate::spawning::items::*;
use crate::spawning::monsters::*;
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
                name: "Player".to_string(),
            },
            CombatStats {
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
            SufferingDamage { damage_received: 0 },
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
                freshwater_viperfish(ecs_world, x, y)
            } else {
                Spawn::random_terrain_monster(ecs_world, x, y);
            }
        }
        // Actually spawn the potions
        for &index in zone.item_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);
            Spawn::random_item(ecs_world, x, y);
        }

        // Spawn special entities
        for (index, tile) in zone.tiles.iter().enumerate() {
            let (x, y) = Zone::get_xy_from_index(index);
            Spawn::tile_entity(ecs_world, x, y, tile);
        }
    }

    /// Spawn a random monster
    pub fn random_terrain_monster(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::dice(1, 5);

        // Dvergar is stronger, shuold be less common
        match dice_roll {
            1 => dvergar(ecs_world, x, y),
            2 => gremlin(ecs_world, x, y),
            _ => deep_one(ecs_world, x, y),
        }
    }

    /// Spawn a random monster
    pub fn random_item(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::d20();
        //TODO test
        lantern(ecs_world, x, y);
        match dice_roll {
            1 => wand(ecs_world, x, y),
            2 => lantern(ecs_world, x, y),
            3 | 4 => shiv(ecs_world, x, y),
            5 | 6 => flask_of_oil(ecs_world, x, y),
            7 | 8 => rockpick(ecs_world, x, y),
            9 => maul(ecs_world, x, y),
            _ => mushroom(ecs_world, x, y),
        }
    }

    /// Spawn a corpse
    pub fn corpse(ecs_world: &mut World, x: i32, y: i32, name: String, edible: Edible) {
        let item_tile_index = (0, 0);
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
                name: format!("{} corpse", name),
            },
            Item {
                item_tile: item_tile_index,
            },
            edible,
            Perishable {
                rot_counter: STARTING_ROT_COUNTER + Roll::d20(),
            },
        );

        ecs_world.spawn(meat);
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
