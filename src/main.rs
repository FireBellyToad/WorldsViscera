use constants::*;
use macroquad::prelude::*;
use map::Map;

mod constants;
mod map;

//Game configuration
fn get_game_configuration() -> Conf {
    Conf {
        window_title: String::from("World's Viscera"),
        fullscreen: false,
        window_height: WINDOW_HEIGHT,
        window_width: WINDOW_WIDTH,
        window_resizable: false,
        //use the default options:
        ..Default::default()
    }
}

#[macroquad::main(get_game_configuration)]
async fn main() {

    let map = Map {
        tileset: load_texture("assets/tiles.png").await.unwrap(),
    };

    loop {
        map.draw_map();

        // Quit game on Q
        if is_key_pressed(KeyCode::Q) {
            break;
        }

        // needed for the engine
        next_frame().await;
    }
}
