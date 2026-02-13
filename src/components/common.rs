use std::collections::{HashMap, HashSet};

use hecs::Entity;
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
    pub visible_tiles: Vec<usize>,
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
impl GameLog {
    pub fn new() -> Self {
        GameLog {
            entries: Vec::new(),
        }
    }
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
    pub smell_log: Option<String>,
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
    pub listen_cache: HashMap<u32, (Entity, String, bool)>,
    pub radius: f32,
    pub cooldown: i32,
}

pub struct ProduceSound {
    pub sound_log: String,
}

pub struct Species {
    pub value: SpeciesEnum,
}
pub struct Hates {
    pub list: HashSet<u32>,
}

pub struct Experience {
    pub value: u32,
    pub auto_advance_counter: u32,
}

pub struct Immobile {}

#[derive(PartialEq, Debug)]
pub enum SpeciesEnum {
    Human,
    Undergrounder,
    Fish,
    Slime,
    Gastropod,
    Myconid,
    Bug,
    Gremlin,
    DeepSpawn,
    Undead,
}

pub struct Diggable {
    pub dig_points: i32,
}
