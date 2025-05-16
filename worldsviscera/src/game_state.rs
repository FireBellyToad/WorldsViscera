use bracket_lib::prelude::{BTerm, GameState};
use specs::{Join, RunNow, World, WorldExt};

use crate::{
    components::common::{Position, Renderable}, map::Map, player, systems::fov::FovSystem
};

pub struct State {
    pub ecs_world: World, // World of ECS, where the framework lives
}

// State implementations
impl State {
    // Function taht will create and run a LeftWalker System
    fn run_systems(&mut self) {
        let mut visibility = FovSystem {};
        visibility.run_now(&self.ecs_world); //Run system, run!
        self.ecs_world.maintain(); // if any changes are queued by the systems, apply them now to the world
    }
}

impl GameState for State {
    //Code executed on each tick of the Game state
    fn tick(&mut self, context: &mut BTerm) {
        //ctx is a reference to the terminal
        context.cls(); //clean terminal

        //Handle player input
        player::player_input(self, context);

        self.run_systems();

        //Fetch from world all the Tiles
        let map_to_draw = self.ecs_world.fetch::<Map>();
        map_to_draw.draw_map(context);

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
