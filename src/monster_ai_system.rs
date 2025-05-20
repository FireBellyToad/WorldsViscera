use std::ops::{Deref, DerefMut};

use bracket_lib::prelude::{DistanceAlg, Point, a_star_search, console};
use specs::prelude::*;
use specs_derive::Component;

use crate::{
    components::{Monster, Name, Position, Targetable, Viewshed},
    map::Map,
    player::Player,
};

#[derive(Component)]
pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Targetable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, player, mut viewshed, mut position, monster, name, target) = data;

        // For each viewshed position monster component join
        for (viewshed, _monster, name, monster_position) in
            (&mut viewshed, &monster, &name, &mut position).join()
        {
            // Get the player perceived position
            for (_player, target) in (&player, &target).join() {
                let player_pos = &target.target_position;
                //This is the same as println but for WebAssembly
                if viewshed
                    .visible_tiles
                    .contains(&Point::constant(player_pos.x, player_pos.y))
                {
                    // If the monster is near the player, attack
                    let distance_from_target = DistanceAlg::Pythagoras.distance2d(
                        Point::new(monster_position.x, monster_position.y),
                        Point::new(player_pos.x, player_pos.y),
                    );
                    
                    if distance_from_target < 1.5 {
                        console::log(format!("{} attacks!", name.name));
                        return;
                    }

                    // Get path from monster to player
                    let path = a_star_search(
                        map.get_index_from_xy(monster_position.x, monster_position.y) as i32,
                        map.get_index_from_xy(player_pos.x, player_pos.y) as i32,
                        map.deref_mut(), //which is like &*map
                    );

                    if path.success && path.steps.len() > 1  {
                        //Move to next calculated step
                        monster_position.x = path.steps[1] as i32 % map.width;
                        monster_position.y = path.steps[1] as i32 / map.width;
                        viewshed.must_recalculate = true;
                    }
                }
            }
        }
    }
}
