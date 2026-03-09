use hecs::{Entity, World};

use crate::{
    components::{
        combat::InflictsDamage,
        common::Named,
        health::Stunned,
        items::{Spell, SpellType},
    },
    spawning::spawner::Spawn,
};

/// Spells are ranged weapons that do not need ammo.
/// They are are monster only abilities.
/// Player can use them only through items.
impl Spawn {
    pub fn daze(ecs_world: &mut World) -> Entity {
        let daze_spell = (
            Named {
                name: "Daze".to_string(),
            },
            Spell {
                spell_type: SpellType::Daze,
                spell_cooldown: 0,
            },
            Stunned { tick_counter: 3 },
        );

        ecs_world.spawn(daze_spell)
    }

    pub fn burning_spray(ecs_world: &mut World) -> Entity {
        let daze_spell = (
            Named {
                name: "Burning Spray".to_string(),
            },
            Spell {
                spell_type: SpellType::BurningSpray,
                spell_cooldown: 0,
            },
            InflictsDamage {
                number_of_dices: 1,
                dice_size: 6,
            },
        );

        ecs_world.spawn(daze_spell)
    }
}
