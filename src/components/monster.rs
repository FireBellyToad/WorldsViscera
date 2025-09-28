use hecs::Entity;

pub struct Monster {}

pub struct Aquatic {}

pub struct Venomous {}

pub struct IsSmart{}

pub struct WantsToApproach {
    pub target: Option<Entity>
}
