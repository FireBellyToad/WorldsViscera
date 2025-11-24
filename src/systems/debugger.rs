use hecs::{Entity, World};

use crate::{
    components::{
        common::Named,
        monster::{Monster, Smart},
    },
    utils::common::ItemsInBackpack,
};

// Debugs stuff at runtime
pub struct Debugger {}

impl Debugger {
    pub fn run(ecs_world: &mut World) {
        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to smell things
            let mut stupid_monsters = ecs_world
                .query::<&Named>()
                .with::<&Monster>()
                .without::<&Smart>();
            let mut items = ecs_world.query::<ItemsInBackpack>();

            for (monster, name) in &mut stupid_monsters {
                if items
                    .iter()
                    .any(|(_, (_, in_backpack, _, _, _, _, _, _, _, _))| {
                        in_backpack.owner.id() == monster.id()
                    })
                {
                    panic!(
                        "Monster {:?} {} has something in backpack even if not smart!!!",
                        monster, name.name
                    )
                }
            }
        }
    }
}
