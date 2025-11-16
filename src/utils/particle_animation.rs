pub struct ParticleAnimation {
    pub current_frame: usize,
    pub frames: Vec<Vec<(i32, i32)>>,
    pub particle_type: u32,
    pub animation_type: ParticleAnimationType,
    pub frame_duration: f32, //TODO this is ambiguous! Refactor ASAP!
}

#[derive(PartialEq)]
pub enum ParticleAnimationType {
    Frame,
    Ray,
    Projectile,
}

impl ParticleAnimation {
    /// Create a line effect animation.
    /// Prepare all the "frames" that will be rendered.
    /// this means that we will create a vector of (x,y) subframes for each frame,
    /// to give the illusion of a growing ray.
    /// Example: [[(x1, y1)], [(x1, y1),(x2, y2)], [(x1, y1),(x2, y2),(x3, y3)],...]
    pub fn new_line(line_effect: Vec<(i32, i32)>, particle_type: u32) -> ParticleAnimation {
        let total_frames = line_effect.len();
        let mut frames: Vec<Vec<(i32, i32)>> = Vec::new();
        let mut pointer: usize = 0;

        for (loaded_frame_renders, _) in (0..total_frames).enumerate() {
            let mut frame_renders: Vec<(i32, i32)> = Vec::new();
            while pointer <= loaded_frame_renders {
                frame_renders.push(line_effect[pointer]);
                pointer += 1;
            }
            pointer = 0;
            frames.push(frame_renders);
        }

        // Return the particle line effect
        Self {
            current_frame: 1, // Skip first frame, will not show anything anyway because of rendering logics
            frames,
            particle_type,
            animation_type: ParticleAnimationType::Ray,
            frame_duration: 3000.0,
        }
    }

    /// Create a new particle animation with a single frame at the given position.
    /// Prepare the single frame that will be rendered.
    /// Keeping the nested array of (x, y) elements for compatibility with other particle effects.
    /// Example: [(x, y)]
    pub fn simple_particle(x: i32, y: i32, particle_type: u32) -> ParticleAnimation {
        Self {
            current_frame: 0,
            frames: vec![vec![(x, y)]],
            particle_type,
            animation_type: ParticleAnimationType::Frame,
            frame_duration: 15000.0,
        }
    }

    /// Create a projectile effect animation.
    /// This animation is like a ray, but instead of creating a list of growing lists of (x,y) elements,
    /// each element is saved inside an array,
    /// Example: [[(x1, y1)],[(x2, y2)],[(x3, y3)],...]
    pub fn new_projectile(line_effect: Vec<(i32, i32)>, particle_type: u32) -> ParticleAnimation {
        let mut frames: Vec<Vec<(i32, i32)>> = Vec::new();

        for frame in line_effect {
            frames.push(vec![frame]);
        }

        // Return the particle line effect
        Self {
            current_frame: 1, // Skip first frame, avoid overlap with the origin
            frames,
            particle_type,
            animation_type: ParticleAnimationType::Projectile,
            frame_duration: 2000.0,
        }
    }
}
