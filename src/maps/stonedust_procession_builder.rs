use std::collections::LinkedList;

use hecs::World;

use crate::{
    components::monster::{SnakeBody, SnakeHead},
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::{ZoneFeatureBuilder, zone::Zone},
    spawning::spawner::Spawn,
    utils::roll::Roll,
};

pub struct StonedustProcessionBuilder {}

/// Builds a shop in the given zone.
impl ZoneFeatureBuilder for StonedustProcessionBuilder {
    fn build(zone: &mut Zone, ecs_world: &mut World) -> Vec<usize> {
        // 1. Select a random location for the acolyte
        let x = Roll::dice(1, MAP_WIDTH - 1);
        let y = Roll::dice(1, MAP_HEIGHT - 1);
        let acolyte = Spawn::stonedust_acolyte(ecs_world, x, y);

        //2. Generate procession body
        let mut body = LinkedList::new();
        let procession_size = Roll::dice(1, 3) + 1;
        let mut free_x = x;
        let mut free_y = y;
        for _ in 0..procession_size {
            // 3. Search for free space. If worm is too big, it cannot fit and we despawn it
            let adjacent_tiles = zone.get_adjacent_passable_tiles(free_x, free_y, true, false);
            if adjacent_tiles.is_empty() {
                println!("Cannot fit stonedust procession!");
                let _ = ecs_world.despawn(acolyte);
                return Vec::new();
            } else {
                free_x = adjacent_tiles[0].0;
                free_y = adjacent_tiles[0].1;
            }

            let body_part = Spawn::stonedust_cultist(ecs_world, free_x, free_y);
            let _ = ecs_world.insert(body_part, (SnakeBody { head: acolyte },));
            body.push_back(body_part);
        }

        //Join head and body
        let _ = ecs_world.insert(acolyte, (SnakeHead { body },));
        Vec::new()
    }
}
