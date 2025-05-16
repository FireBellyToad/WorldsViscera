use bracket_lib::prelude::{BTerm, GameState};
use specs::{Join, RunNow, World, WorldExt, world::Index};

use crate::{
    components::common::{Position, Renderable},
    map::Map,
    player,
    systems::{fov::FovSystem, monster_ai::MonsterAI},
};

// Game state struct
pub struct State {
    pub ecs_world: World,    // World of ECS, where the framework lives
    pub run_state: RunState, //Running state
}

// State implementations
impl State {
    // Function taht will create and run a LeftWalker System
    fn run_systems(&mut self) {
        let mut visibility = FovSystem {};
        visibility.run_now(&self.ecs_world); //Run system, run!
        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.ecs_world); //Run system, run!
        self.ecs_world.maintain(); // if any changes are queued by the systems, apply them now to the world
    }
}

impl GameState for State {
    //Code executed on each tick of the Game state
    fn tick(&mut self, context: &mut BTerm) {
        //ctx is a reference to the terminal
        context.cls(); //clean terminal

        // Run system only while not paused, or else wait for player input.
        // Make the whole game turn based
        if self.run_state == RunState::Running {
            self.run_systems();
            self.run_state = RunState::Paused;
        } else {
            self.run_state = player::player_input(self, context);
        }

        //Fetch from world all the Tiles
        let map_to_draw = self.ecs_world.fetch::<Map>();
        map_to_draw.draw_map(context);

        //We read Position and Renderable currently inserted in world
        let positions = self.ecs_world.read_storage::<Position>();
        let renderables = self.ecs_world.read_storage::<Renderable>();

        //Render all Renderables in their Position
        for (pos, render) in (&positions, &renderables).join() {
            let index = map_to_draw.get_index_from_xy(pos.x, pos.y);
            //Render only if tile underneath is visible to the player
            if map_to_draw.visible_tiles[index] {
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
}

// Game state enums

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
}
