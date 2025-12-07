use std::cmp::max;

use hecs::World;

use crate::{
    constants::*,
    maps::{
        ZoneBuilder, ZoneFeatureBuilder,
        cracks_builder::CracksBuilder,
        mushroom_field_builder::MushroomFieldBuilder,
        river_builder::RiverBuilder,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

/// Builds a cavern like zone with Drunken walk algorithm
pub struct DrunkenWalkZoneBuilder {}

impl ZoneBuilder for DrunkenWalkZoneBuilder {
    /// Create new dungeon zone (needed?)
    fn build(depth: u32, ecs_world: &mut World) -> Zone {
        let mut zone = Zone::new(depth);

        // Simple Drunken walk
        let mut current_position = (MAP_WIDTH / 2, MAP_HEIGHT / 2);
        let max_iterations = max(
            DRUNKEN_WALK_MIN_ITERATIONS,
            DRUNKEN_WALK_MAX_ITERATIONS - depth as i32,
        );

        // the more deep we are, the less free space we have
        for _ in 0..max_iterations {
            zone.tiles[Zone::get_index_from_xy(&current_position.0, &current_position.1)] =
                TileType::Floor;

            let mut life_counter = 0;
            let mut unblock_tentatives = 10;
            while life_counter < DRUNKEN_WALK_LIFE_MAX {
                let new_direction_roll = Roll::dice(1, 4);
                let (mut dest_x, mut dest_y) = current_position;

                match new_direction_roll {
                    1 => dest_x += 1,
                    2 => dest_y += 1,
                    3 => dest_x -= 1,
                    4 => dest_y -= 1,
                    _ => {}
                }

                // Avoid boundaries, or else skip iteration
                if dest_x <= 1 || dest_x >= MAP_WIDTH - 1 || dest_y <= 1 || dest_y >= MAP_HEIGHT - 1
                {
                    if unblock_tentatives < 0 {
                        unblock_tentatives = 10;
                        current_position = (MAP_WIDTH / 2, MAP_HEIGHT / 2);
                    } else {
                        unblock_tentatives -= 1;
                        println! {"DrunkenWalkZoneBuilder - unblock_tentatives {}",unblock_tentatives};
                    }
                    continue;
                }

                let index = Zone::get_index_from_xy(&dest_x, &dest_y);
                if zone.tiles[index] == TileType::Wall {
                    zone.tiles[index] = TileType::Floor;
                }
                life_counter += 1;
                current_position = (dest_x, dest_y);
            }
            current_position = (Roll::dice(1, MAP_WIDTH - 2), Roll::dice(1, MAP_HEIGHT - 2));
        }

        // Less river the more deep we are
        let river_number = max(
            0,
            Roll::dice(1, MAX_RIVERS_IN_ZONE) + 1 - (depth as i32 / 3),
        );
        for _ in 0..river_number {
            RiverBuilder::build(&mut zone, ecs_world);
        }

        //Mushroom Field
        if depth.is_multiple_of(MUSHROOM_FIELD_LEVEL) {
            MushroomFieldBuilder::build(&mut zone, ecs_world);
        }

        // Populate water and blocked tiles here, needed for correct spawning
        zone.populate_blocked();
        zone.populate_water();

        // Generate monster and items spawn points within each room
        let monster_number = Roll::dice(1, MAX_MONSTERS_IN_ZONE) + depth as i32 + 1;
        let items_number = max(0, Roll::dice(1, MAX_ITEMS_IN_ZONE) + 3 - depth as i32);
        let fauna_number = max(0, Roll::d20() + 3);
        let braziers_number = max(0, Roll::dice(2, MAX_BRAZIER_IN_ZONE) - depth as i32);
        let (mut try_x, mut try_y);

        // Random braziers
        for _ in 0..braziers_number {
            let mut brazier_index = zone.tiles.len() / 2;
            while zone.tiles[brazier_index] != TileType::Floor {
                try_x = Roll::dice(1, MAP_WIDTH - 2);
                try_y = Roll::dice(1, MAP_HEIGHT - 2);
                brazier_index = Zone::get_index_from_xy(&try_x, &try_y);
            }
            zone.tiles[brazier_index] = TileType::Brazier;
        }

        for _ in 0..monster_number {
            for _ in 0..MAX_SPAWN_TENTATIVES {
                let x = Roll::dice(1, MAP_WIDTH - 2) as f32;
                let y = Roll::dice(1, MAP_HEIGHT - 2) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid walls, player and duplicate spawnpoints
                if index != zone.player_spawn_point
                    && (zone.tiles[Zone::get_index_from_xy_f32(x, y)] == TileType::Floor
                        || zone.tiles[Zone::get_index_from_xy_f32(x, y)] == TileType::Water)
                    && !zone.monster_spawn_points.contains(&index)
                {
                    zone.monster_spawn_points.insert(index);
                    break;
                }
            }
        }

        for _ in 0..items_number {
            for _ in 0..MAX_SPAWN_TENTATIVES {
                let x = Roll::dice(1, MAP_WIDTH - 2) as f32;
                let y = Roll::dice(1, MAP_HEIGHT - 2) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid walls, player and duplicate spawnpoints
                if index != zone.player_spawn_point
                    && zone.tiles[Zone::get_index_from_xy_f32(x, y)] == TileType::Floor
                    && zone.item_spawn_points.insert(index)
                {
                    break;
                }
            }
        }

        for _ in 0..fauna_number {
            for _ in 0..MAX_SPAWN_TENTATIVES {
                let x = Roll::dice(1, MAP_WIDTH - 2) as f32;
                let y = Roll::dice(1, MAP_HEIGHT - 2) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid walls, player and duplicate spawnpoints
                if index != zone.player_spawn_point
                    && zone.tiles[Zone::get_index_from_xy_f32(x, y)] == TileType::Floor
                    && zone.fauna_spawn_points.insert(index)
                {
                    break;
                }
            }
        }

        // First crack generation, used for player spawn point and down passage to ensure we have a path to the exit
        let mut first_crack_tiles = CracksBuilder::build(&mut zone, ecs_world);
        while first_crack_tiles.is_empty() {
            first_crack_tiles = CracksBuilder::build(&mut zone, ecs_world);
        }

        // Random starting point for player, taken from first crack
        for &index in first_crack_tiles.iter().skip(2) {
            if zone.tiles[index] == TileType::Floor {
                zone.player_spawn_point = index;
                break;
            }
        }

        zone.player_spawn_point = zone.tiles.len() / 2;
        while zone.tiles[zone.player_spawn_point] == TileType::Wall {
            try_x = Roll::dice(1, MAP_WIDTH - 2);
            try_y = Roll::dice(1, MAP_HEIGHT - 2);
            zone.player_spawn_point = Zone::get_index_from_xy(&try_x, &try_y);
        }

        // Random point for DownPassage, taken from the craked wall. This ensures that the passage is somehow reachable.
        let down_passage_roll = Roll::dice(1, first_crack_tiles.len() as i32) as usize - 1;
        let down_passage_index = first_crack_tiles[down_passage_roll];
        zone.tiles[down_passage_index] = TileType::DownPassage;

        // Add random cracks
        let cracks_number = Roll::dice(1, MAX_CRACKS_IN_ZONE) + (depth as i32 / 3);
        for _ in 0..cracks_number {
            CracksBuilder::build(&mut zone, ecs_world);
        }

        zone
    }
}
