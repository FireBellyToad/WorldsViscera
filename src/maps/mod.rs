use crate::maps::zone::Zone;

pub mod dungeon_map_builder;
pub mod zone;
pub mod arena_map_builder;
pub mod drunken_walk_map_builder;

/// Trait for Zone Builders
pub trait ZoneBuilder {
    fn build() -> Zone;
}
