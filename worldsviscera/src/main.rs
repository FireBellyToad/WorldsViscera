//We are using some classes of bracket_lib to access all its libraries
use bracket_lib::{
    color::{BLACK, RED1, YELLOW},
    prelude::to_cp437,
    terminal::{BError, BTerm, BTermBuilder, FontCharType, GameState, RGB},
};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

//Utility Struct to attach stuff to it
struct State {
    ecs_world: World, // World of ECS, where the framework lives
}

//GameState Trait implementation needed to create a tick function
impl GameState for State {

    //Code executed on each tick of the Game state
    fn tick(&mut self, context: &mut BTerm) {
        //ctx is a reference to the terminal
        context.cls(); //clean terminal

        //We read Position and Renderable currently inserted in world
        let positions = self.ecs_world.read_storage::<Position>();
        let renderables = self.ecs_world.read_storage::<Renderable>();

        //Render all Renderables in their Position
        // specs.join is equivalent to a Database Join:
        // get a list of tuples that have BOTH  Position and Renderable Component
        // then, for each tuple, do render
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

//Position
#[derive(Component)] // Macro for deriving all the needed data for Component (think of something like @Component in Java)
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: FontCharType,
    foreground: RGB,
    background: RGB,
}

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

    //Here the ECS world register Position and Renderable types inside its system
    //This seems like is working with a Generic / Pseudoreflection mechanism!
    gs.ecs_world.register::<Position>();
    gs.ecs_world.register::<Renderable>();

    //Insert player "@" into world
    gs.ecs_world
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: to_cp437('@'),
            foreground: RGB::named(YELLOW),
            background: RGB::named(BLACK),
        })
        .build();

    //Insert 10 other entities
    for i in 0..10 {
        gs.ecs_world
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: to_cp437('☺'),
                foreground: RGB::named(RED1),
                background: RGB::named(BLACK),
            })
            .build();
    }

    //I prefer this syntax for now. I need to explicit the main_loop origin
    //Main Engine loop
    bracket_lib::terminal::main_loop(context, gs) //must not have a semicolon: this function returns a BError!
}
