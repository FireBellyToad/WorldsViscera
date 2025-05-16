use bracket_lib::prelude::{Point, console};
use specs::prelude::*;
use specs_derive::Component;

use crate::{
    components::common::{Monster, Name, Position, Viewshed}, player::{Player}
};

#[derive(Component)]
pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {

    type SystemData = (
        ReadStorage<'a, Player>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        
        let (player, viewshed, position, monster, name) = data;

        // For each viewshed position monster component join
        for (viewshed, _monster, name) in (&viewshed, &monster, &name).join() {
            for (_player, position) in (&player, &position).join() {
                //This is the same as println but for WebAssembly
                if viewshed
                    .visible_tiles
                    .contains(&Point::constant(position.x, position.y))
                {
                    console::log(format!("{} do stuff", name.name));
                }
            }
        }
    }
}
