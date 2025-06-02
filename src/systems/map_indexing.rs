use hecs::World;

use crate::{
    components::common::*,
    map::{Map, get_index_from_xy},
};

pub struct MapIndexing {}

impl MapIndexing {
    pub fn run(ecs_world: &World) {
        let mut entites = ecs_world.query::<&Position>();
        let mut blockers = ecs_world.query::<(&Position, &BlocksTile)>();
        let mut map_query = ecs_world.query::<&mut Map>();
        let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

        map.populate_blocked();
        map.clear_content_index();
        //index all blocked tiles
        for (_e, (position, _blocks)) in &mut blockers {
            let index = get_index_from_xy(position.x, position.y);
            map.blocked_tiles[index] = true;
        }

        //index all the things in the map based on their position
        for (entity, position) in &mut entites {
            let index = get_index_from_xy(position.x, position.y);
            map.tile_content[index].push(entity);
        }
    }
}
