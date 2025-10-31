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
            for (entity, animation) in &mut animations {
                if animation.current_frame < animation.frames.len() {
                    animation.current_frame += 1;
                } else {
                    anim_to_remove.push(entity);
                }
            }
        }
        // Remove finished animations
        for e in anim_to_remove {
            let _ = game_state.ecs_world.despawn(e);
            game_state.run_state = RunState::DoTick;
        }
    }

    // Check if animations are present in ECS World, and set the appropriate delay for better animation
    pub fn check_if_animations_are_present(
        game_engine: &mut GameEngine,
        game_state: &mut EngineState,
    ) -> bool {
        let mut animations = game_state.ecs_world.query::<&ParticleAnimation>();

        // Only one animation must be present to enter into the DrawParticles state
        if (&mut animations).into_iter().next().is_some() {
            game_state.run_state = RunState::DrawParticles;
            game_engine.set_delay(100.0);
            return true;
        }
        false
    }
}
