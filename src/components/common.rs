use std::collections::{HashMap, HashSet};

use hecs::Entity;
use macroquad::math::Rect;

use crate::{
    components::health::DiseaseType,
    constants::{BURNING_PARTICLE_TYPE, DAZE_PARTICLE_TYPE, STONE_FELL_PARTICLE_TYPE},
    utils::assets::TextureName,
};

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
    pub name: &'static str,
    pub attack_verb: Option<&'static str>,
}
pub struct BlocksTile {}

pub struct ProduceCorpse {}

/// Game log, used in UI
pub struct GameLog {
    pub entries: Vec<String>,
}
impl GameLog {
    pub fn new() -> Self {
        GameLog {
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the game log.
    /// Entries are stored as owned `String`s, using `String::from` to convert from `&str`.
    pub fn add_entry(&mut self, entry: &str) {
        self.entries.push(String::from(entry));
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
    pub smell_log: Option<&'static str>,
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
    pub listen_cache: HashMap<u32, (Entity, &'static str, bool)>,
    pub radius: f32,
    pub cooldown: i32,
}

pub struct ProduceSound {
    pub sound_log: &'static str,
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
    pub produces: DigProductEnum,
}

#[derive(PartialEq, Debug, Clone)]
pub enum DigProductEnum {
    Gold,
    Stone,
}

// HashSet should handle ImmuntyType enum variants safely like they are actual different immunity
// for example:
//  {ImmunityTypeEnum::Disease(DiseaseType::Fever), ImmunityTypeEnum::Disease(DiseaseType::FleshRot)}
pub struct Immunity {
    pub to: HashMap<ImmunityTypeEnum, u8>,
}

#[derive(PartialEq, Debug, Hash, Eq, Clone, Copy)]
pub enum ImmunityTypeEnum {
    Blindness,
    Disease(DiseaseType),
    DamagingFloor,
    Slipping,
    StoneFellSpell,
}

pub struct Spell {
    pub spell_type: SpellType,
    pub spell_cooldown: u32,
}

#[derive(PartialEq, Debug)]
pub enum SpellType {
    Daze,
    BurningSpray,
    StoneFell,
}

impl SpellType {
    pub fn particle(&self) -> u32 {
        match *self {
            SpellType::Daze => DAZE_PARTICLE_TYPE,
            SpellType::BurningSpray => BURNING_PARTICLE_TYPE,
            SpellType::StoneFell => STONE_FELL_PARTICLE_TYPE,
        }
    }
}

pub struct SpellList {
    pub spells: Vec<Entity>,
}
