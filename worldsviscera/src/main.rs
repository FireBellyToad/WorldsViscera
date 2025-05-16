mod components;
mod game_state;
mod map;
mod systems;

use components::{common::{Position, Renderable, Viewshed, Name, Monster}, *};
use game_state::*;

//We are using some classes of bracket_lib to access all its libraries
use bracket_lib::{
    color::{BLACK, RED, YELLOW}, prelude::{to_cp437, FontCharType}, random::RandomNumberGenerator, terminal::{BError, BTermBuilder, RGB}
};
use map::Map;
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
        run_state: RunState::Running
    };

    //Create new map
    let map = Map::new_map_rooms_and_corridors();
    let player_start_position = map.rooms[0].center(); // make the player start in the center of the first available room

    //Here the ECS world register Position and Renderable types inside its system
    //This seems like is working with a Generic / Pseudoreflection mechanism!
    gs.ecs_world.register::<Position>();
    gs.ecs_world.register::<Renderable>();
    gs.ecs_world.register::<Player>();
    gs.ecs_world.register::<Viewshed>();
    gs.ecs_world.register::<Name>();
    gs.ecs_world.register::<Monster>();

    //Insert player "@" into world
    gs.ecs_world
        .create_entity()
        .with(Position {
            x: player_start_position.x,
            y: player_start_position.y,
        })
        .with(Renderable {
            glyph: to_cp437('@'),
            foreground: RGB::named(YELLOW),
            background: RGB::named(BLACK),
        })
        .with(Player {}) 
        .with(Viewshed { // FOV component
            visible_tiles: Vec::new(),
            range: player::VIEW_RADIUS,
            must_recalculate: true,
        })
        .build();

    // FIXME REMOVE!!!
    //For each room except the first one
    let mut rng = RandomNumberGenerator::new();
    //.enumerate also expose an index of the current iteration
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let room_center_point = room.center();
        let monster_glyph: FontCharType;
        let name:String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => { monster_glyph = to_cp437('g'); name = String::from("Goblin"); }
            _ => { monster_glyph = to_cp437('o'); name = String::from("Orc"); }
        }

        //Insert monster
        gs.ecs_world
            .create_entity()
            .with(Position { x: room_center_point.x, y: room_center_point.y })
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
                name: format!("{} #{}", &name, i)
            })
            .build();
    }
    
    gs.ecs_world.insert(map);

    //Main Engine loop
    bracket_lib::terminal::main_loop(context, gs) //must not have a semicolon: this function returns a BError!
}
