mod common_components;
mod game_state;
mod map;
mod player;

use common_components::*;
use game_state::*;

//We are using some classes of bracket_lib to access all its libraries
use bracket_lib::{
    color::{BLACK, YELLOW},
    prelude::to_cp437,
    terminal::{BError, BTermBuilder, RGB},
};
use map::{MAP_HEIGHT, MAP_WIDTH};
use player::Player;
use specs::prelude::*;

fn main() -> BError {
    //This is a context. what is this?
    let context = BTermBuilder::simple80x50() //Create a 80x50 basic terminal
        .with_title("Hello minimal Bracket world")
        .build()?; // ? is used to propagate BErrors inside this main.

    //Main loop needs a struct that implements the GameState Trait and have a World
    //must be mutable so we can change its fields
    let mut gs: State = State {
        ecs_world: World::new(),
    };

    //Create new map
    gs.ecs_world.insert(map::new_map());

    //Here the ECS world register Position and Renderable types inside its system
    //This seems like is working with a Generic / Pseudoreflection mechanism!
    gs.ecs_world.register::<Position>();
    gs.ecs_world.register::<Renderable>();
    gs.ecs_world.register::<Player>();

    //Insert player "@" into world
    gs.ecs_world
        .create_entity()
        .with(Position {
            x: MAP_WIDTH / 2,
            y: MAP_HEIGHT / 2,
        })
        .with(Renderable {
            glyph: to_cp437('@'),
            foreground: RGB::named(YELLOW),
            background: RGB::named(BLACK),
        })
        .with(Player {})
        .build();

    //Main Engine loop
    bracket_lib::terminal::main_loop(context, gs) //must not have a semicolon: this function returns a BError!
}
