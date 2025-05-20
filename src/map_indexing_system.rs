use specs::prelude::*;
use specs::{ReadStorage, System, WriteExpect};

use crate::{
    components::{BlocksTile, Position},
    map::Map,
};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {

    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, position, blockers) = data;

        map.populate_blocked_tiles();
        // Set to blocked all tiles where a Position Component is present (Player and Monsters)
        for (position, _blocker) in (&position, &blockers).join() {
            let index = map.get_index_from_xy(position.x, position.y);
            map.blocked_tiles[index] = true;
        }
    }
}
