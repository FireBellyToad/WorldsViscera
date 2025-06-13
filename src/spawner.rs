use crate::components::combat::{CombatStats, InflictsDamage, SufferingDamage};
use crate::components::common::{BlocksTile, Named, Position, ProduceCorpse, Renderable, Viewshed};
use crate::components::health::{CanAutomaticallyHeal, Hunger};
use crate::components::items::{Edible, Invokable, Item, Perishable};
use crate::components::monster::Monster;
use crate::components::player::Player;
use crate::constants::*;
use crate::maps::zone::Zone;
use crate::systems::hunger_check::HungerStatus;
use crate::utils::assets::TextureName;
use crate::utils::roll::Roll;
use hecs::World;
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
                unarmed_attack_dice: 6,
                current_toughness: rolled_toughness,
                max_toughness: rolled_toughness,
                current_dexterity: rolled_dexterity,
                max_dexterity: rolled_dexterity,
            },
            SufferingDamage { damage_received: 0 },
            CanAutomaticallyHeal { tick_counter: 0 },
            Hunger {
                tick_counter: MAX_HUNGER_TICK_COUNTER,
                current_status: HungerStatus::Normal,
            },
        );

        ecs_world.spawn(player_entity);
    }

    /// Spawn entities inside a room
    pub fn everyhing_in_map(ecs_world: &mut World, zone: &Zone) {
        // Actually spawn the monsters
        for &index in zone.monster_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);
            Self::random_monster(ecs_world, x as i32, y as i32);
        }
        // Actually spawn the potions
        for &index in zone.item_spawn_points.iter() {
            let (x, y) = Zone::get_xy_from_index(index);
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
                range: BASE_MONSTER_VIEW_RADIUS,
                must_recalculate: true,
            },
            Named { name: name },
            BlocksTile {},
            combat_stats,
            SufferingDamage { damage_received: 0 },
            ProduceCorpse {},
        );

        ecs_world.spawn(monster_entity);
    }

    /// Spawn a random monster
    pub fn random_item(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = Roll::dice(1, 3);
        // Dvergar is stronger, shuold be less common
        match dice_roll {
            1 => Self::wand(ecs_world, x, y),
            _ => Self::wand(ecs_world, x, y),
        }
    }

    pub fn corpse(ecs_world: &mut World, x: i32, y: i32, name: String, edible: Edible) {
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
                name: String::from(format!("{} corpse", name)),
            },
            Item { item_tile_index },
            edible,
            Perishable {
                rot_counter: STARTING_ROT_COUNTER + Roll::d20(),
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
