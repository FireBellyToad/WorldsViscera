use hecs::World;

use crate::{
    components::{
        common::*,
        items::{InBackback, MustBeFueled, ProduceLight},
    },
    maps::zone::Zone,
    systems::fov::FieldOfView,
};

pub struct MapIndexing {}

impl MapIndexing {
    pub fn run(ecs_world: &World) {
        let mut entites = ecs_world.query::<&Position>();
        let mut blockers = ecs_world.query::<&Position>().with::<&BlocksTile>();
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

        // index all water tiles
        zone.populate_water();

        //index all lit tiles checking all light producers
        zone.lit_tiles.fill(false);
        let all_working_lighters = MapIndexing::get_all_working_lighters(ecs_world);
        for dto in &all_working_lighters {
            // This viewshed is used for light calculation
            let mut viewshed = Viewshed {
                visible_tiles: Vec::new(),
                range: dto.radius,
                must_recalculate: true,
            };

            FieldOfView::compute(zone, &mut viewshed, dto.x, dto.y);

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

    /// Get all the lighters in the zone, even the ones that are stored in the backpack of someone
    fn get_all_working_lighters(ecs_world: &World) -> Vec<ProduceLightPositionDTO> {
        // Extract all light producers that could be laying on the ground OR be in a backpack
        // They could or could NOT have Fuel management (think a an oil lanter VS a brazier)
        // Either way get them
        let mut lighters_query = ecs_world.query::<(
            Option<&Position>,
            Option<&InBackback>,
            Option<&MustBeFueled>,
            &ProduceLight,
        )>();

        lighters_query
            .iter()
            .filter_map(|(_e, (position, in_backpack, fuel, produce_light))| {
                if fuel.is_none() || fuel.expect("Must have Fuel!").fuel_counter > 0 {
                    if let Some(pos) = position {
                        return Some(ProduceLightPositionDTO {
                            radius: produce_light.radius,
                            x: pos.x,
                            y: pos.y,
                        });
                    } else if let Some(back) = in_backpack {
                        // Get position of the owner so we can illuminate from there
                        let pos = ecs_world.get::<&Position>(back.owner).expect("Entity has no Position");
                        return Some(ProduceLightPositionDTO {
                            radius: produce_light.radius,
                            x: pos.x,
                            y: pos.y,
                        });
                    }
                }

                None
            })
            .collect()
        
    }
}

struct ProduceLightPositionDTO {
    radius: i32,
    x: i32,
    y: i32,
}
