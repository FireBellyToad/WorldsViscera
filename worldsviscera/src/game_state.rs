use bracket_lib::prelude::{BTerm, GameState};
use specs::{Join, World, WorldExt};

use crate::{common_components::{Position, Renderable}, map::{self, TileType}, player};


pub struct State {
    pub ecs_world: World, // World of ECS, where the framework lives
}

//GameState Trait implementation 
impl GameState for State {

    //Code executed on each tick of the Game state
    fn tick(&mut self, context: &mut BTerm) {
        //ctx is a reference to the terminal
        context.cls(); //clean terminal

        //Handle player input
        player::player_input(self, context);

        //Fetch from world all the Tiles
        let map_to_draw = self.ecs_world.fetch::<Vec<TileType>>();
        map::draw_map(&map_to_draw, context);

        //We read Position and Renderable currently inserted in world
        let positions = self.ecs_world.read_storage::<Position>();
        let renderables = self.ecs_world.read_storage::<Renderable>();

        //Render all Renderables in their Position
        for (pos, render) in (&positions, &renderables).join() {
            context.set(
                pos.x,
                pos.y,
                render.foreground,
                render.background,
                render.glyph,
            );
        }
    }
}