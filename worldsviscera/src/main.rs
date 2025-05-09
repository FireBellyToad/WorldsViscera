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

//State common implementations
impl State {

    // Function taht will create and run a LeftWalker System 
    fn run_systems(&mut self) {
        let mut left_walker = LeftWalker{};
        left_walker.run_now(&self.ecs_world); //Run system, run!
        self.ecs_world.maintain(); // if any changes are queued by the systems, apply them now to the world
    }
}

//GameState Trait implementation needed to create a tick function
impl GameState for State {
    //Code executed on each tick of the Game state
    fn tick(&mut self, context: &mut BTerm) {
        //ctx is a reference to the terminal
        context.cls(); //clean terminal

        // The current state will run!
        self.run_systems();

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

#[derive(Component)]
struct LeftMover {}

//Empty structure to attach logic
struct LeftWalker {}

// Implementing "System" Trait for LeftWalker.
// The System is asking us what it needs to be done
// 'a specifies that the lifetime must be long enough to make the System run
impl<'a> System<'a> for LeftWalker {

    // SystemData is an alias of the tuple (ReadStorage, WriteStorage)
    // Here we define what kind of SystemData the "run" function will use and how
    // We can READ LeftMover Components and READ AND WRITE Position components
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    // System Trait implementation
    // We get the data we defined as SystemData (readalbe LeftMover components and writeable Position components)
    // and do stuff with it
    fn run(&mut self, (left_mover, mut position): Self::SystemData) {

        //For each entity that have BOTH LeftMover and Position Component, move left
        //_left_mover is never used here
        for (_left_mover, position) in (&left_mover, &mut position).join() {
            position.x -= 1;
            if position.x < 0 {
                position.x = 79;
            }
        }
    }
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
    gs.ecs_world.register::<LeftMover>();

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
            .with(LeftMover {}) // This entity has component LeftMover
            .build();
    }

    //I prefer this syntax for now. I need to explicit the main_loop origin
    //Main Engine loop
    bracket_lib::terminal::main_loop(context, gs) //must not have a semicolon: this function returns a BError!
}
