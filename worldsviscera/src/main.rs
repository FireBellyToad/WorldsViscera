//We are using some classes of bracket_lib to access all its libraries
use bracket_lib::{
    color::*,
    prelude::{VirtualKeyCode, to_cp437},
    random::RandomNumberGenerator,
    terminal::{BError, BTerm, BTermBuilder, FontCharType, GameState, RGB},
};
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 50;

//Utility Struct to attach stuff to it
struct State {
    ecs_world: World, // World of ECS, where the framework lives
}

//State common implementations
impl State {
    // Function taht will create and run a LeftWalker System
    fn run_systems(&mut self) {
        let mut left_walker = LeftWalker {};
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

        //Handle player input
        player_input(self, context);

        // The current state will run!
        self.run_systems();

        //Fetch from world all the Tiles
        let map_to_draw = self.ecs_world.fetch::<Vec<TileType>>();
        draw_map(&map_to_draw, context);

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

//Why Player is Component?
#[derive(Component, Debug)]
struct Player {}

fn try_move_player(delta_x: i32, delta_y: i32, ecs_world: &mut World) {
    //Get all entities with Position an Player components
    let mut positions = ecs_world.write_storage::<Position>();
    let mut players = ecs_world.write_storage::<Player>();
    let map = ecs_world.fetch::<Vec<TileType>>();

    // For each one that have both of them (only one, the Player), change position if space is free
    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_index = get_index_from_xy(pos.x + delta_x, pos.y + delta_y);
        if map[destination_index] != TileType::Wall {
            pos.x = min(MAP_WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_HEIGHT - 1, max(0, pos.y + delta_y));
        }
    }
}

fn player_input(game_state: &mut State, context: &mut BTerm) {
    //Move Player
    match context.key {
        None => {} // Do nothing if none is pressed
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut game_state.ecs_world),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut game_state.ecs_world),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut game_state.ecs_world),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut game_state.ecs_world),
            _ => {} // Do nothing for all other keys
        },
    }
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
                position.x = MAP_WIDTH - 1;
            }
        }
    }
}

// Copy Trait will handle a "=" operator as a copy instead of a moving (x = y will leave both variables valid)
#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

// Get vector index from x and y position on map
pub fn get_index_from_xy(x: i32, y: i32) -> usize {
    (y as usize * (MAP_WIDTH as usize)) + x as usize // Remember this is returned, no semicolon
}

fn new_map() -> Vec<TileType> {
    //vec! is a factory macro for Vectors (ArrayList in Java)
    //here, vec! creates a 4000 (80 x 50) items long vector full of "Floor" tiles
    let mut map = vec![TileType::Floor; (MAP_WIDTH * MAP_HEIGHT) as usize];

    //Make boundary walls
    for x in 0..MAP_WIDTH {
        map[get_index_from_xy(x, 0)] = TileType::Wall;
        map[get_index_from_xy(x, MAP_HEIGHT - 1)] = TileType::Wall;
    }

    for y in 0..MAP_HEIGHT {
        map[get_index_from_xy(0, y)] = TileType::Wall;
        map[get_index_from_xy(MAP_WIDTH - 1, y)] = TileType::Wall;
    }

    //Get thread local RNG
    let mut rng = RandomNumberGenerator::new();

    //Random walls around the map
    for _i in 0..400 {
        let wall_x = rng.roll_dice(1, MAP_WIDTH - 1);
        let wall_y = rng.roll_dice(1, MAP_HEIGHT - 1);
        let index = get_index_from_xy(wall_x, wall_y);

        //Place wall in map unless x and y are on player position
        if index != get_index_from_xy(MAP_WIDTH / 2, MAP_HEIGHT / 2) {
            map[index] = TileType::Wall
        }
    }

    //Return new map
    map
}

fn draw_map(map: &[TileType], context: &mut BTerm) {
    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            match map[get_index_from_xy(x, y)] {
                TileType::Floor => {
                    context.set(x, y, RGB::named(GRAY50), RGB::named(BLACK), to_cp437('.'))
                }
                TileType::Wall => {
                    context.set(x, y, RGB::named(GREEN1), RGB::named(BLACK), to_cp437('#'))
                }
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

    //Insert into ECS a new map
    gs.ecs_world.insert(new_map());

    //Here the ECS world register Position and Renderable types inside its system
    //This seems like is working with a Generic / Pseudoreflection mechanism!
    gs.ecs_world.register::<Position>();
    gs.ecs_world.register::<Renderable>();
    gs.ecs_world.register::<LeftMover>();
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

    //I prefer this syntax for now. I need to explicit the main_loop origin
    //Main Engine loop
    bracket_lib::terminal::main_loop(context, gs) //must not have a semicolon: this function returns a BError!
}
