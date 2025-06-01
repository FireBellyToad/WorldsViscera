use std::collections::HashMap;

use macroquad::texture::{Texture2D, load_texture};

use crate::assets::TextureName;

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
        assets
    }
}
