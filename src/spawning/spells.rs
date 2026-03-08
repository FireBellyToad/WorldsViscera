use hecs::{Entity, World};

use crate::{
    components::{
        common::Named,
        health::Stunned,
        items::{AmmoType, RangedWeapon},
    },
    spawning::spawner::Spawn,
};

/// Spells are ranged weapons that do not need ammo.
/// They are are monster only abilities.
/// Player can use them only through items.

impl Spawn {
    pub fn daze(ecs_world: &mut World) -> Entity {
        let stun_spell = (
            Named {
                name: "Daze".to_string(),
            },
            RangedWeapon {
                ammo_type: AmmoType::Spell,
                ammo_count_total: 0,
                spell_countdown: 0,
            },
            Stunned { tick_counter: 3 },
        );

        ecs_world.spawn(stun_spell)
    }
}
