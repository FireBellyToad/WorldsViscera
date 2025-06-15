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

        // Generate monster and items spawn points within each room
        let monster_number = Roll::dice(1, MAX_MONSTERS_ON_ROOM_START);
        let items_number = Roll::dice(1, MAX_ITEMS_ON_ROOM_START);

        for _m in 0..monster_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH as i32 - 1) as f32;
                let y = Roll::dice(1, MAP_HEIGHT as i32 - 1) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid duplicate spawnpoints
                if zone.monster_spawn_points.insert(index) {
                    break;
                }
            }
        }

        for _i in 0..items_number {
            for _t in 0..MAX_SPAWN_TENTANTIVES {
                let x = Roll::dice(1, MAP_WIDTH as i32 - 1) as f32;
                let y = Roll::dice(1, MAP_HEIGHT as i32 - 1) as f32;
                let index = Zone::get_index_from_xy_f32(x, y);

                // avoid duplicate spawnpoints
                if zone.item_spawn_points.insert(index) {
                    break;
                }
            }
        }

        zone
    }
}
