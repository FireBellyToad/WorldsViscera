use hecs::World;

use crate::maps::zone::Zone;

pub mod arena_zone_builder;
pub mod cracks_builder;
pub mod drunken_walk_zone_builder;
pub mod dungeon_zone_builder;
pub mod mushroom_field_builder;
pub mod river_builder;
pub mod test_zone_builder;
pub mod zone;

/// Trait for Zone Builders
pub trait ZoneBuilder {
    fn build(depth: u32, ecs_world: &mut World) -> Zone;
}

/// Trait for Zone Feature Builders
pub trait ZoneFeatureBuilder {
    fn build(zone: &mut Zone, ecs_world: &mut World) -> Vec<usize>;
}

enum ZoneFeatureBuilderOrigin {
    Top,
    Left,
}
