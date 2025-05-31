use hecs::World;
use macroquad::math::Rect;

use crate::assets::TextureName;
use crate::components::combat::*;
use crate::components::common::*;
use crate::components::monster::Monster;
use crate::components::player::Player;
use crate::components::player::VIEW_RADIUS;
use crate::constants::*;
use crate::map::Map;
use crate::utils::random_util::RandomUtils;

/// Spawner of game entities
pub struct Spawn {}

impl Spawn {
    /// Spawn player
    pub fn player(ecs_world: &mut World, map: &Map) {

        // Roll appropriate stats
        let rolled_toughness = RandomUtils::stat_roll();
        // TODO Player with Soldier background must have 1+2d3 starting stamina
        let rolled_stamina = RandomUtils::d6()+1;

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
                    w: TILE_SIZE as f32,
                    h: TILE_SIZE as f32,
                },
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
            },
            Damageable { damage_received: 0 },
        );

        ecs_world.spawn(player_entity);
    }
    
    /// Spawn a random monster
    pub fn random_monster(ecs_world: &mut World, x: i32, y: i32) {
        let dice_roll = RandomUtils::dice(1, 4);

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
            },
            1.0,
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
            },
            2.0,
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
                    x: tile_index * TILE_SIZE as f32, //TODO fix
                    y: 0.0,
                    w: TILE_SIZE as f32,
                    h: TILE_SIZE as f32,
                },
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: VIEW_RADIUS,
                must_recalculate: true,
            },
            Named { name: name },
            BlocksTile {},
            combat_stats,
            Damageable { damage_received: 0 },
        );

        ecs_world.spawn(monster_entity);
    }
}
