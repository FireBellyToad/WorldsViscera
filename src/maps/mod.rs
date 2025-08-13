use crate::maps::zone::Zone;

pub mod dungeon_zone_builder;
pub mod zone;
pub mod arena_zone_builder;
pub mod drunken_walk_zone_builder;
pub mod river_builder;

/// Trait for Zone Builders
pub trait ZoneBuilder {
    fn build(depth: i32) -> Zone;
}

/// Trait for Zone Feature Builders
pub trait ZoneFeatureBuilder {
    fn build(zone: &mut Zone) ;
}
