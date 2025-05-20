//We are using some classes of bracket_lib to access all its libraries
use bracket_lib::{
    color::*,
    prelude::to_cp437,
    random::RandomNumberGenerator,
    terminal::{BError, BTerm, BTermBuilder, FontCharType, GameState, RGB},
};

use map_indexing_system::MapIndexingSystem;
use monster_ai_system::MonsterAI;
use specs::prelude::*;

mod components;
mod map;
mod monster_ai_system;
mod map_indexing_system;
pub mod player;
mod rect;
mod visibility_system;

use components::*;
use map::*;
use player::*;
use visibility_system::*;

//Utility Struct to attach stuff to it
struct State {
    ecs_world: World, // World of ECS, where the framework lives
    run_state: RunState,
}

//State common implementations
impl State {
    // Function taht will create and run a LeftWalker System
    fn run_systems(&mut self) {
        let mut visibility = VisibilitySystem {};
        visibility.run_now(&self.ecs_world); //Run system, run!
        let mut monster_ai = MonsterAI {};
        monster_ai.run_now(&self.ecs_world); //Run system, run!
        let mut map_indexing = MapIndexingSystem {};
        map_indexing.run_now(&self.ecs_world); //Run system, run!
        self.ecs_world.maintain(); // if any changes are queued by the systems, apply them now to the world
    }
}

//GameState Trait implementation needed to create a tick function
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
        // specs.join is equivalent to a Database Join:
        // get a list of tuples that have BOTH  Position and Renderable Component
        // then, for each tuple, do render
        for (pos, render) in (&positions, &renderables).join() {
            let index = map_to_draw.get_index_from_xy(pos.x, pos.y);
            //Only if visible
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

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running,
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
        run_state: RunState::Running,
    };

    //Insert into ECS a new map
    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].get_center(); // make the player start in the center of the first available room
    //Must be placed here or else map will be owned by gs.ecs_world.insert(map);

    //Here the ECS world register Position and Renderable types inside its system
    //This seems like is working with a Generic / Pseudoreflection mechanism!
    gs.ecs_world.register::<Position>();
    gs.ecs_world.register::<Renderable>();
    gs.ecs_world.register::<Player>();
    gs.ecs_world.register::<Viewshed>();
    gs.ecs_world.register::<Monster>();
    gs.ecs_world.register::<Name>();
    gs.ecs_world.register::<Targetable>();
    gs.ecs_world.register::<BlocksTile>();

    //Insert player "@" into world
    gs.ecs_world
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: to_cp437('@'),
            foreground: RGB::named(YELLOW),
            background: RGB::named(BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            // FOV component
            visible_tiles: Vec::new(),
            range: player::VIEW_RADIUS,
            must_recalculate: true,
        })
        .with(Targetable {
            target_position: Position {
                x: player_x,
                y: player_y,
            },
        })
        .build();

    //For each room except the first one
    let mut rng = RandomNumberGenerator::new();
    //.enumerate also expose an index of the current iteration
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.get_center();
        let monster_glyph: FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                monster_glyph = to_cp437('g');
                name = String::from("Goblin");
            }
            _ => {
                monster_glyph = to_cp437('o');
                name = String::from("Orc");
            }
        }

        //Insert monster
        gs.ecs_world
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                foreground: RGB::named(RED),
                background: RGB::named(BLACK),
                glyph: monster_glyph,
            })
            .with(Viewshed {
                // FOV component
                visible_tiles: Vec::new(),
                range: player::VIEW_RADIUS,
                must_recalculate: true,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", &name, i),
            })
            .with(BlocksTile {})
            .build();
    }
    gs.ecs_world.insert(map);

    //I prefer this syntax for now. I need to explicit the main_loop origin
    //Main Engine loop
    bracket_lib::terminal::main_loop(context, gs) //must not have a semicolon: this function returns a BError!
}
