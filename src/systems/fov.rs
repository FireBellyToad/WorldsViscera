use adam_fov_rs::{IVec2, compute_fov};
use hecs::World;

use crate::{
    components::{common::*, player::Player},
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::zone::Zone,
    utils::common::Utils,
};

use adam_fov_rs::GridPoint;

pub struct FieldOfView {}

impl FieldOfView {
    pub fn calculate(ecs_world: &World) {
        let player_entity_id = Player::get_entity_id();

        let mut zone_query = ecs_world.query::<&mut Zone>();
        let (_, zone) = zone_query
            .iter()
            .last()
            .expect("Zone is not in hecs::World");

        //Deconstruct data into tuple
        let mut viewsheds = ecs_world.query::<(&mut Viewshed, &Position)>();
        //For each Entity with Components Viewshed and Position
        for (entity, (viewshed, position)) in &mut viewsheds {
            if viewshed.must_recalculate {
                viewshed.must_recalculate = false;
                viewshed.visible_tiles.clear();

                FieldOfView::compute(zone, viewshed, position.x, position.y);

                //recalculate rendered view if entity is Player
                if entity.id() == player_entity_id {
                    zone.visible_tiles.fill(false);
                    for &index in viewshed.visible_tiles.iter() {
                        let (x, y) = Zone::get_xy_from_index(index);
                        let distance = Utils::distance(&x, &position.x, &y, &position.y);

                        // if is lit, that we can show and reveal
                        // Adiacent tiles are always visible
                        if zone.lit_tiles[index] || distance < 2.0 {
                            zone.revealed_tiles[index] = true;
                            zone.visible_tiles[index] = true;
                        }
                    }
                }
            }
        }
    }

    /// Wrapper to riutilize standard compute fov everywhere, given a viewshed
    pub fn compute(zone: &mut Zone, viewshed: &mut Viewshed, x: i32, y: i32) {
        // Utility lambda for opaque tiles
        let is_opaque = |position: IVec2| zone.is_tile_opaque(&position[0], &position[1]);

        // Utility lambda for setting visible tiles
        let set_to_visible = |position: IVec2| {
            viewshed
                .visible_tiles
                .push(Zone::get_index_from_xy(&position[0], &position[1]));
        };

        // Calculate Fov
        compute_fov(
            Point { x, y },
            viewshed.range as usize,
            [MAP_WIDTH, MAP_HEIGHT],
            is_opaque,
            set_to_visible,
        );

        // Sort visible tiles by distance from origin. Needed for better IA handling
        viewshed.visible_tiles.sort_by(|&a_index, &b_index| {
            let (ax, ay) = Zone::get_xy_from_index(a_index);
            let (bx, by) = Zone::get_xy_from_index(b_index);
            Utils::distance(&x, &ax, &y, &ay).total_cmp(&Utils::distance(&x, &bx, &y, &by))
        });
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl GridPoint for Point {
    fn xy(&self) -> adam_fov_rs::IVec2 {
        IVec2::new(self.x, self.y)
    }
}
