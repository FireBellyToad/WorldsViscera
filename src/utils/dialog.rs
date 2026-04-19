use std::collections::HashMap;

use hecs::{Entity, World};
use macroquad::texture::Texture2D;

use crate::{
    engine::state::GameState, systems::trade_system::TradeDtt, utils::assets::TextureName,
};

#[derive(PartialEq, Debug, Clone)]
pub enum DialogAction {
    Eat(Entity),
    Quaff(Entity),
    Trade(TradeDtt),
    StealPick(Entity),
    StealEat(Entity),
    ShowMessage(&'static str),
}

/// Trait for shared behaviours on game dialogs
pub trait Dialog {
    /// Must describe how this dialog handles player input
    fn handle_input(game_state: &mut GameState, action: DialogAction);
    /// Must describe how this dialog draws himself
    fn draw(textures: &HashMap<TextureName, Texture2D>, ecs_world: &World, action: &DialogAction);
}
