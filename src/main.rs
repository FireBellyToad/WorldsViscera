use macroquad::prelude::*;

const MAP_WIDTH: i32 = 40;
const MAP_HEIGHT: i32 = 25;
const TILE_SIZE: i32 = 32;
const UI_BORDER: i32 = 8;
const WINDOW_WIDTH: i32 = (UI_BORDER * 2) + (MAP_WIDTH * TILE_SIZE);
const WINDOW_HEIGHT: i32 = (UI_BORDER * 2) + (MAP_HEIGHT * TILE_SIZE);

//snip
fn conf() -> Conf {
    Conf {
        window_title: String::from("World's Viscera"),
        fullscreen: false,
        window_height: WINDOW_HEIGHT,
        window_width: WINDOW_WIDTH,
        window_resizable: false,
        //you can add other options too, or just use the default ones:
        ..Default::default()
    }
}

enum TileType {
    Floor,
    Wall,
}
//then pass the function to the attribute
#[macroquad::main(conf)]

async fn main() {
    let tileset = load_texture("assets/tiles.png").await.unwrap();
    let tiles_index = get_tile_index(TileType::Floor) * TILE_SIZE as f32;
    println!("tiles_index {tiles_index}");

    //Draw a 80 x 50 map made up of 32 x 32 tiles
    loop {
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                draw_texture_ex(
                    &tileset,
                    (UI_BORDER + (x * TILE_SIZE)) as f32,
                    (UI_BORDER + (y * TILE_SIZE)) as f32,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect {
                            x: tiles_index,
                            y: 0.0,
                            w: TILE_SIZE as f32,
                            h: TILE_SIZE as f32,
                        }),
                        ..Default::default()
                    },
                );
            }
        }

        // Quit game on Q
        if is_key_pressed(KeyCode::Q) {
            break;
        }

        // needed for the engine
        next_frame().await;
    }
}

fn get_tile_index(tile_type: TileType) -> f32 {
    match tile_type {
        TileType::Floor => 0.0,
        TileType::Wall => 1.0,
    }
}
