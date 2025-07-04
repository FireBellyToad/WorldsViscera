use std::collections::HashMap;

use macroquad::texture::{load_texture, Texture2D};



// Needed for hashmap
#[derive(Hash,Eq, PartialEq)]
pub enum TextureName {
    Creatures,
    Tiles,
    Items,
    Particles
}

pub struct Load {}

impl Load {
    
    pub async fn assets() -> HashMap<TextureName, Texture2D> {
        let mut assets = HashMap::new();
        assets.insert(
            TextureName::Creatures,
            load_texture("assets/creatures.png").await.unwrap(),
        );
        assets.insert(
            TextureName::Tiles,
            load_texture("assets/tiles.png").await.unwrap(),
        );
        assets.insert(
            TextureName::Items,
            load_texture("assets/items.png").await.unwrap(),
        );
        assets.insert(
            TextureName::Particles,
            load_texture("assets/particles.png").await.unwrap(),
        );
        assets
    }
}
