use hecs::World;

use crate::{
    constants::*,
    maps::{
        ZoneBuilder, ZoneFeatureBuilder,
        gold_mine_builder::GoldMineBuilder,
        mushroom_field_builder::MushroomFieldBuilder,
        stonedust_procession_builder::StonedustProcessionBuilder,
        stonedust_shrine_builder::StonedustShrineBuilder,
        zone::{TileType, Zone},
    },
    utils::{common::Utils, roll::Roll},
};

/// Builds a simple arena zone, with only the boundary walls
#[allow(dead_code)]
pub struct TestZoneBuilder {}

impl ZoneBuilder for TestZoneBuilder {
    /// Create new dungeon zone (needed?)
    fn build(depth: u32, ecs_world: &mut World) -> Zone {
        let mut zone = Zone::new(depth, TileType::Wall);

        // Create boundaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x != 0 && y != 0 && x != MAP_WIDTH - 1 && y != MAP_HEIGHT - 1 {
                    let index = Zone::get_index_from_xy(&x, &y);
                    zone.tiles[index] = TileType::Floor
                }
            }
        }

        //Straight river
        for y in 1..MAP_HEIGHT - 1 {
            zone.tiles[Zone::get_index_from_xy(&10, &y)] = TileType::Water;
        }
        //Lake
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let distance = Utils::distance(&10, &x, &(MAP_HEIGHT / 2), &y);

                if distance < 4.0 {
                    zone.tiles[Zone::get_index_from_xy(&x, &y)] = TileType::Water;
                }
            }
        }

        //Straight crack
        for y in 3..MAP_HEIGHT - 3 {
            zone.tiles[Zone::get_index_from_xy(&40, &y)] = TileType::CrackedWall;
        }

        // Populate water and blocked tiles here, needed for correct spawning
        zone.populate_blocked();
        zone.populate_water();

        zone.player_spawn_point = Zone::get_index_from_xy(&(MAP_WIDTH / 2), &(MAP_HEIGHT / 2));

        zone.tiles[Zone::get_index_from_xy_f32(MAP_WIDTH_F32 * 0.25, MAP_HEIGHT_F32 * 0.25)] =
            TileType::Brazier;
        zone.tiles[Zone::get_index_from_xy_f32(MAP_WIDTH_F32 * 0.75, MAP_HEIGHT_F32 * 0.25)] =
            TileType::Brazier;
        zone.tiles[Zone::get_index_from_xy_f32(MAP_WIDTH_F32 * 0.25, MAP_HEIGHT_F32 * 0.75)] =
            TileType::Brazier;
        zone.tiles[Zone::get_index_from_xy_f32(MAP_WIDTH_F32 * 0.75, MAP_HEIGHT_F32 * 0.75)] =
            TileType::Brazier;

        //Mushroom Field
        MushroomFieldBuilder::build(&mut zone, ecs_world);

        // Generate items spawn points within each room
        let items_number = Roll::dice(1, MAX_ITEMS_IN_ZONE) + 15;
        let monster_number = Roll::dice(1, MAX_MONSTERS_IN_ZONE) + 3;

        for _ in 0..monster_number {
            for _ in 0..MAX_SPAWN_TENTATIVES {
                let x = Roll::dice(1, MAP_WIDTH - 2) as f32;
                let y = Roll::dice(1, MAP_HEIGHT - 2) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid walls, player and duplicate spawnpoints
                if index != zone.player_spawn_point
                    && zone.tiles[Zone::get_index_from_xy_f32(x, y)] != TileType::Wall
                    && zone.monster_spawn_points.insert(index)
                {
                    break;
                }
            }
        }

        for _ in 0..items_number {
            for _ in 0..MAX_SPAWN_TENTATIVES {
                let x = Roll::dice(1, MAP_WIDTH - 3) as f32 + 1.0;
                let y = Roll::dice(1, MAP_HEIGHT - 3) as f32 + 1.0;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid duplicate spawnpoints
                if zone.tiles[index] != TileType::Floor {
                    continue;
                } else if zone.item_spawn_points.insert(index) {
                    break;
                }
            }
        }

        // Add random gold mine
        GoldMineBuilder::build(&mut zone, ecs_world);

        StonedustProcessionBuilder::build(&mut zone, ecs_world);
        StonedustShrineBuilder::build(&mut zone, ecs_world);

        zone
    }
}
