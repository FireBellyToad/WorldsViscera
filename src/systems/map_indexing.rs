use hecs::World;

use crate::{
    components::{common::*, items::ProduceLight},
    maps::zone::Zone,
    systems::fov::FieldOfView,
};

pub struct MapIndexing {}

impl MapIndexing {
    pub fn run(ecs_world: &World) {
        let mut entites = ecs_world.query::<&Position>();
        let mut blockers = ecs_world.query::<&Position>().with::<&BlocksTile>();
        let mut lighters = ecs_world.query::<(&Position, &ProduceLight)>();
        let mut zone_query = ecs_world.query::<&mut Zone>();
        let (_e, zone) = zone_query
            .iter()
            .last()
            .expect("Zone is not in hecs::World");

        //index all blocked tiles
        zone.populate_blocked();
        for (_e, position) in &mut blockers {
            let index = Zone::get_index_from_xy(position.x, position.y);
            zone.blocked_tiles[index] = true;
        }

        //index all lit tiles checking all light producers
        zone.lit_tiles.fill(false);
        for (_e, (position, produce_light)) in &mut lighters {
            // This viewshed is used for light calculation
            let mut viewshed = Viewshed {
                visible_tiles: Vec::new(),
                range: produce_light.radius,
                must_recalculate: true,
            };

            FieldOfView::compute(zone, &mut viewshed, position.x, position.y);

            for (x, y) in viewshed.visible_tiles {
                let index = Zone::get_index_from_xy(x, y);
                zone.lit_tiles[index] = true;
            }
        }

        zone.clear_content_index();
        //index all the things in the zone based on their position
        for (entity, position) in &mut entites {
            let index = Zone::get_index_from_xy(position.x, position.y);
            zone.tile_content[index].push(entity);
        }
    }
}
