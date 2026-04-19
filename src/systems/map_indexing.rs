use hecs::World;

use crate::{
    components::{
        common::*,
        items::{InBackback, MustBeFueled, ProduceLight, TurnedOn},
    },
    engine::state::GameState,
    maps::zone::Zone,
    systems::fov_manager::FieldOfViewManager,
};

pub struct MapIndexing {}

impl MapIndexing {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;
        let zone = game_state
            .current_zone
            .as_mut()
            .expect("must have Some Zone");

        let mut entities_with_pos = ecs_world.query::<(&Position, Option<&BlocksTile>)>();

        //index all blocked tiles
        zone.populate_blocked();
        entities_with_pos
            .iter()
            .filter_map(
                //Extract entities that blocks tiles without querying World again
                |(_, (pos, blocks_opt))| {
                    if blocks_opt.is_some() {
                        Some(pos)
                    } else {
                        None
                    }
                },
            )
            .for_each(|position| {
                let index = Zone::get_index_from_xy(&position.x, &position.y);
                zone.blocked_tiles[index] = true;
            });

        // index all water tiles
        zone.populate_water();

        //index all lit tiles checking all light producers
        MapIndexing::handle_zone_lighting(ecs_world, zone);

        zone.clear_content_index();
        //index all the things in the zone based on their position
        for (entity, (position, ..)) in &mut entities_with_pos {
            zone.tile_content[Zone::get_index_from_xy(&position.x, &position.y)].push(entity);
        }
    }

    /// Get all the lighters in the zone, even the ones that are stored in the backpack of someone
    fn handle_zone_lighting(ecs_world: &World, zone: &mut Zone) {
        //Init light mapping by setting all to false ("dark")
        zone.lit_tiles.fill(false);

        // Get all light producers that could be laying on the ground OR be in a backpack
        // They could or could NOT have Fuel management (think a an oil lanter VS a brazier)
        // Either way get them
        let mut lighters_query = ecs_world
            .query::<(
                Option<&Position>,
                Option<&InBackback>,
                Option<&MustBeFueled>,
                &ProduceLight,
            )>()
            .with::<&TurnedOn>();

        lighters_query
            .iter()
            .filter_map(|(_, (position, in_backpack, fuel, produce_light))| {
                if fuel.is_none() || fuel.expect("Must have Fuel!").fuel_counter > 0 {
                    if let Some(pos) = position {
                        return Some(ProduceLightPositionDTO {
                            radius: produce_light.radius,
                            x: pos.x,
                            y: pos.y,
                        });
                    } else if let Some(back) = in_backpack {
                        // Get position of the owner so we can illuminate from there
                        let pos = ecs_world
                            .get::<&Position>(back.owner)
                            .expect("Entity has no Position");
                        return Some(ProduceLightPositionDTO {
                            radius: produce_light.radius,
                            x: pos.x,
                            y: pos.y,
                        });
                    }
                }

                None
            })
            // For each light producer, calculate the light emission and
            // set the tiles to be lighted
            .for_each(|dto| {
                // This viewshed is used for light calculation
                let mut viewshed = Viewshed {
                    visible_tiles: Vec::new(),
                    range: dto.radius,
                    must_recalculate: true,
                };

                FieldOfViewManager::compute(zone, &mut viewshed, dto.x, dto.y);

                for index in viewshed.visible_tiles {
                    zone.lit_tiles[index] = true;
                }
            })
    }
}

struct ProduceLightPositionDTO {
    radius: i32,
    x: i32,
    y: i32,
}
