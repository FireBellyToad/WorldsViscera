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
                attack_verb: Some("stun".to_string()),
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
                attack_verb: Some("burn".to_string()),
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

    pub fn stone_fell(ecs_world: &mut World) -> Entity {
        let stone_fell = (
            Named {
                name: "Stone fell".to_string(),
                attack_verb: Some("strike".to_string()),
            },
            Spell {
                spell_type: SpellType::StoneFell,
                spell_cooldown: 0,
            },
            InflictsDamage {
                number_of_dices: 2,
                dice_size: 6,
            },
        );

        ecs_world.spawn(stone_fell)
    }
}
