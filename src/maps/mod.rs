use crate::maps::zone::Zone;

pub mod arena_zone_builder;
pub mod drunken_walk_zone_builder;
pub mod dungeon_zone_builder;
pub mod river_builder;
pub mod test_zone_builder;
pub mod zone;

/// Trait for Zone Builders
pub trait ZoneBuilder {
    fn build(depth: u32) -> Zone;
}

/// Trait for Zone Feature Builders
pub trait ZoneFeatureBuilder {
    fn build(zone: &mut Zone);
}
