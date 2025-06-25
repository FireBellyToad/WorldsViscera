use hecs::Entity;

use crate::{
    engine::{
        gameengine::GameEngine,
        state::{EngineState, RunState},
    },
    utils::particle_animation::ParticleAnimation,
};

pub struct ParticleManager {}

/// Particle Manager
impl ParticleManager {
    pub fn run(game_state: &mut EngineState) {
        let mut anim_to_remove: Vec<Entity> = Vec::new();

        //Animate with current timing
        {
            let mut animations = game_state.ecs_world.query::<&mut ParticleAnimation>();
            for (e, a) in &mut animations {
                if a.current_frame < a.frames.len() {
                    a.current_frame += 1;
                } else {
                    anim_to_remove.push(e);
                }
            }
        }
        // Remove finished animations
        for e in anim_to_remove {
            let _ = game_state.ecs_world.despawn(e);
            //FIXME This is bad, what about monsters using wands?
            game_state.run_state = RunState::DoTick;
        }
    }

    // Check if animations are present in ECS World, and set the appropriate delay for better animation
    pub fn check_if_animations_are_present(
        game_engine: &mut GameEngine,
        game_state: &mut EngineState,
    ) {
        let mut animations = game_state.ecs_world.query::<&ParticleAnimation>();

        for _ in &mut animations {
            game_state.run_state = RunState::DrawParticles;
            game_engine.set_delay(1.0);
            // Only one animation must be present to enter into the DrawParticles state
            break;
        }
    }
}
