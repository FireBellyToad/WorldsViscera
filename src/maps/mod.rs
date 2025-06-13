use crate::maps::zone::Zone;

pub mod dungeon_zone_builder;
pub mod zone;
pub mod arena_zone_builder;
pub mod drunken_walk_zone_builder;

/// Trait for Zone Builders
pub trait ZoneBuilder {
    fn build() -> Zone;
}
