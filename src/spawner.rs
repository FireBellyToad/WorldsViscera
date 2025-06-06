use std::collections::HashSet;

use hecs::World;
use macroquad::math::Rect;

use crate::components::combat::*;
use crate::components::common::*;
use crate::components::items::{Edible, Invokable, Item};
use crate::components::map::Map;
use crate::components::monster::Monster;
use crate::components::player::Player;
use crate::constants::*;
use crate::utils::assets::TextureName;
use crate::utils::roll::Roll;

/// Spawner of game entities
pub struct Spawn {}

impl Spawn {
    /// Spawn player
    pub fn player(ecs_world: &mut World, map: &Map) {
        // Roll appropriate stats
        let rolled_toughness = Roll::stat();
        let rolled_dexterity = Roll::stat();
        // TODO Player with Soldier background must have 1+2d3 starting stamina
        let rolled_stamina = Roll::d6() + 1;

        let player_entity = (
            Player {},
            Position {
                x: map.rooms[0].center()[0] as i32,
                y: map.rooms[0].center()[1] as i32,
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
                range: VIEW_RADIUS,
                must_recalculate: true,
            },
            Named {
                name: String::from("Player"),
            },
            CombatStats {
                //TOdO Random
                current_stamina: rolled_stamina,
                max_stamina: rolled_stamina,
                base_armor: 2,
                unarmed_attack_dice: 6,
                current_toughness: rolled_toughness,
                max_toughness: rolled_toughness,
                current_dexterity: rolled_dexterity,
                max_dexterity: rolled_dexterity,
            },
            SufferingDamage { damage_received: 0 },
        );

        ecs_world.spawn(player_entity);
    }

    /// Spawn entities inside a room
    pub fn in_room(ecs_world: &mut World, room: &Rect) {
        // Monsters
        let mut monster_spawn_points: HashSet<usize> = HashSet::new();
        let monster_number = Roll::dice(1, MAX_MONSTERS_ON_ROOM_START) - 1;

        // Generate spawn points within room
        for _m in 0..monster_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = (room.x + Roll::dice(1, room.w as i32 - 1) as f32) as usize;
                let y = (room.y + Roll::dice(1, room.h as i32 - 1) as f32) as usize;
                let index = (y * MAP_WIDTH as usize) + x;

                // avoid duplicate spawnpoints
                if monster_spawn_points.insert(index) {
                    break;
                }
            }
        }

        // Actually spawn the monsters
        for &index in monster_spawn_points.iter() {
            let x = index % MAP_WIDTH as usize;
            let y = index / MAP_WIDTH as usize;
            Self::random_monster(ecs_world, x as i32, y as i32);
        }

        // Items
        let mut item_spawn_points: HashSet<usize> = HashSet::new();
        let items_number = Roll::dice(1, MAX_ITEMS_ON_ROOM_START) - 1;

        // Generate span points within room
        for _i in 0..items_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = (room.x + Roll::dice(1, room.w as i32 - 1) as f32) as usize;
                let y = (room.y + Roll::dice(1, room.h as i32 - 1) as f32) as usize;
                let index = (y * MAP_WIDTH as usize) + x;

                // avoid duplicate spawnpoints
                if item_spawn_points.insert(index) {
                    break;
                }
            }
        }

        // Actually spawn the potions
        for &index in item_spawn_points.iter() {
            let x = index % MAP_WIDTH as usize;
            let y = index / MAP_WIDTH as usize;
            Self::random_item(ecs_world, x as i32, y as i32);
        }
    }

    /// Spawn a random monster
    pub fn random_monster(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::dice(1, 4);

        // Dvergar is stronger, shuold be less common
        match dice_roll {
            1 => Self::dvergar(ecs_world, x, y),
            _ => Self::deep_one(ecs_world, x, y),
        }
    }

    fn deep_one(ecs_world: &mut World, x: i32, y: i32) {
        Self::create_monster(
            ecs_world,
            "Deep One".to_string(),
            CombatStats {
                current_stamina: 3,
                max_stamina: 3,
                base_armor: 1,
                unarmed_attack_dice: 4,
                current_toughness: 8,
                max_toughness: 8,
                current_dexterity: 10,
                max_dexterity: 10,
            },
            1.0, //TODO fix
            x,
            y,
        );
    }

    fn dvergar(ecs_world: &mut World, x: i32, y: i32) {
        Self::create_monster(
            ecs_world,
            "Dvergar".to_string(),
            CombatStats {
                current_stamina: 4,
                max_stamina: 4,
                base_armor: 2,
                unarmed_attack_dice: 6,
                current_toughness: 10,
                max_toughness: 10,
                current_dexterity: 8,
                max_dexterity: 8,
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
        tile_index: f32,
        x: i32,
        y: i32,
    ) {
        let monster_entity = (
            Monster {},
            Position { x: x, y: y },
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
                range: VIEW_RADIUS,
                must_recalculate: true,
            },
            Named { name: name },
            BlocksTile {},
            combat_stats,
            SufferingDamage { damage_received: 0 },
        );

        ecs_world.spawn(monster_entity);
    }

    /// Spawn a random monster
    pub fn random_item(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::dice(1, 3);
        // Dvergar is stronger, shuold be less common
        match dice_roll {
            1 => Self::wand(ecs_world, x, y),
            _ => Self::meat(ecs_world, x, y),
        }
    }

    fn meat(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = 0;
        let meat = (
            Position { x, y },
            Renderable {
                texture_name: TextureName::Items,
                texture_region: Rect {
                    x: (item_tile_index * TILE_SIZE) as f32,
                    y: 0.0,
                    w: TILE_SIZE_F32,
                    h: TILE_SIZE_F32,
                },
                z_index: 0,
            },
            Named {
                name: String::from("Fresh meat"),
            },
            Item { item_tile_index },
            Edible {
                nutrition_amount: 6,
            },
        );

        ecs_world.spawn(meat);
    }

    fn wand(ecs_world: &mut World, x: i32, y: i32) {
        let item_tile_index = 1;
        let wand = (
            Position { x, y },
            Renderable {
                texture_name: TextureName::Items,
                texture_region: Rect {
                    x: (item_tile_index * TILE_SIZE) as f32,
                    y: 0.0,
                    w: TILE_SIZE_F32,
                    h: TILE_SIZE_F32,
                },
                z_index: 0,
            },
            Named {
                name: String::from("Thunder wand"),
            },
            Item { item_tile_index },
            Invokable {},
            InflictsDamage {
                number_of_dices: 2,
                dice_size: 4,
            },
        );

        ecs_world.spawn(wand);
    }
}
