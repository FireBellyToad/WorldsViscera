use hecs::World;

use crate::{components::common::*, map::{get_index_from_xy, Map}};

pub struct MapIndexing {}

impl MapIndexing {
    pub fn index_map(ecs_world: &World) {
        let mut blockers = ecs_world.query::<(&Position, &BlocksTile)>();
        let mut map_query = ecs_world.query::<&mut Map>();
        let (_e, map) = map_query.iter().last().expect("Map is not in hecs::World");

        map.populate_blocked();

        for (_e,(position, _blocks)) in &mut blockers {
            let index = get_index_from_xy(position.x, position.y);
            map.blocked_tiles[index] = true;
        }
    }
}
