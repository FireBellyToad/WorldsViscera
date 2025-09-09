use macroquad::math::Rect;

use crate::utils::assets::TextureName;

pub struct Position {
    pub x: i32,
    pub y: i32,
}
pub struct Renderable {
    pub texture_name: TextureName,
    pub texture_region: Rect,
    pub z_index: i32,
}
pub struct Viewshed {
    pub visible_tiles: Vec<(i32, i32)>,
    pub range: i32,
    pub must_recalculate: bool,
}
pub struct Named {
    pub name: String,
}
pub struct BlocksTile {}

pub struct ProduceCorpse {}

pub struct GameLog {
    pub entries: Vec<String>,
}
pub struct WaitingToAct {
    pub tick_countdown: i32,
}

pub struct MyTurn {}

#[derive(PartialEq, Debug)]
pub enum SmellIntensity {
    None,
    Faint,
    Strong,
}

pub struct Smellable {
    pub smell_log: String,
    pub intensity: SmellIntensity,
}

pub struct CanSmell {
    pub intensity: SmellIntensity,
    pub radius: f32,
}

pub struct Wet {
    pub tick_countdown: i32,
}

pub struct CanListen {
    pub listen_cache: Vec<(i32, bool)>,
}
pub struct ProduceSound {
    pub sound_log: String,
}
