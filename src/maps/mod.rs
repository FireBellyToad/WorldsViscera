use crate::maps::game_map::GameMap;

pub mod dungeon_map_builder;
pub mod game_map;
pub mod arena_map_builder;
pub mod drunken_walk_map_builder;

/// Trait for GameMap Builders
pub trait GameMapBuilder {
    fn build() -> GameMap;
}
