use hecs::World;

use crate::{components::common::*, maps::zone::Zone};

pub struct MapIndexing {}

impl MapIndexing {
    pub fn run(ecs_world: &World) {
        let mut entites = ecs_world.query::<&Position>();
        let mut blockers = ecs_world.query::<(&Position, &BlocksTile)>();
        let mut map_query = ecs_world.query::<&mut Zone>();
        let (_e, zone) = map_query
            .iter()
            .last()
            .expect("Zone is not in hecs::World");

        zone.populate_blocked();
        zone.clear_content_index();
        //index all blocked tiles
        for (_e, (position, _b)) in &mut blockers {
            let index = Zone::get_index_from_xy(position.x, position.y);
            zone.blocked_tiles[index] = true;
        }

        //index all the things in the zone based on their position
        for (entity, position) in &mut entites {
            let index = Zone::get_index_from_xy(position.x, position.y);
            zone.tile_content[index].push(entity);
        }
    }
}
