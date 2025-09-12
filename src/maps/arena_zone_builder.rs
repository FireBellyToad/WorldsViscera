use crate::{
    constants::*,
    maps::{
        ZoneBuilder,
        zone::{TileType, Zone},
    },
    utils::roll::Roll,
};

/// Builds a simple arena zone, with only the boundary walls
pub struct ArenaZoneBuilder {}

impl ZoneBuilder for ArenaZoneBuilder {
    /// Create new dungeon zone (needed?)
    fn build(depth: i32) -> Zone {
        let mut zone = Zone::new(depth);

        // Create boundaries
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if x != 0 && y != 0 && x != MAP_WIDTH - 1 && y != MAP_HEIGHT - 1 {
                    let index = Zone::get_index_from_xy(x, y);
                    zone.tiles[index] = TileType::Floor
                }
            }
        }

        zone.player_spawn_point = Zone::get_index_from_xy(MAP_WIDTH / 2, MAP_HEIGHT / 2);

        zone.tiles[Zone::get_index_from_xy_f32(MAP_WIDTH_F32 * 0.25, MAP_HEIGHT_F32 * 0.25)] =
            TileType::Brazier;
        zone.tiles[Zone::get_index_from_xy_f32(MAP_WIDTH_F32 * 0.75, MAP_HEIGHT_F32 * 0.25)] =
            TileType::Brazier;
        zone.tiles[Zone::get_index_from_xy_f32(MAP_WIDTH_F32 * 0.25, MAP_HEIGHT_F32 * 0.75)] =
            TileType::Brazier;
        zone.tiles[Zone::get_index_from_xy_f32(MAP_WIDTH_F32 * 0.75, MAP_HEIGHT_F32 * 0.75)] =
            TileType::Brazier;

        // Generate items spawn points within each room
        let items_number = Roll::dice(1, MAX_ITEMS_IN_ZONE) + 15;

        for _ in 0..items_number {
            for _ in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH  - 3) as f32 + 1.0;
                let y = Roll::dice(1, MAP_HEIGHT  - 3) as f32 + 1.0;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid duplicate spawnpoints
                if zone.blocked_tiles[index] {
                    continue;
                } else if zone.item_spawn_points.insert(index) {
                    break;
                }
            }
        }

        // Random starting point for DownPassage
        let passage_index = Zone::get_index_from_xy(MAP_WIDTH / 2, MAP_HEIGHT / 2);
        zone.tiles[passage_index] = TileType::DownPassage;

        zone
    }
}
