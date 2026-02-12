use hecs::World;

use crate::{
    components::{
        combat::CombatStats,
        common::Named,
        health::Paralyzed,
        monster::{Monster, Smart},
        player::Player,
    },
    engine::state::GameState,
    utils::common::ItemsInBackpack,
};

// Debugs stuff at runtime
pub struct Debugger {}

impl Debugger {
    pub fn run(game_state: &mut GameState) {
        let ecs_world = &mut game_state.ecs_world;

        // Scope for keeping borrow checker quiet
        {
            // List of entities that want to smell things
            let mut stupid_monsters = ecs_world
                .query::<&Named>()
                .with::<&Monster>()
                .without::<&Smart>();
            let mut items = ecs_world.query::<ItemsInBackpack>();
            let mut paralyzed_player = ecs_world
                .query::<&CombatStats>()
                .with::<(&Player, &Paralyzed)>();

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

            for (e, combat_stats) in &mut paralyzed_player {
                if combat_stats.current_dexterity > 0 {
                    panic!("Player {:?} has 1+ DEX but is paralyzed", e);
                }
            }
        }
    }
}
