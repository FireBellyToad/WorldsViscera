pub struct ParticleAnimation {
    pub current_frame: usize,
    pub frames: Vec<Vec<(i32, i32)>>,
    pub particle_type: u32,
    pub exclude_first_frame: bool,
}

impl ParticleAnimation {
    /// Create a line effect animation.
    pub fn new_line(line_effect: Vec<(i32, i32)>, particle_type: u32) -> ParticleAnimation {
        // Prepare all the "frames" that will be rendered.
        // this means that we will create a vector of (x,y) for each frame,
        // to give the impression of a growing ray.
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
            current_frame: 0,
            frames,
            particle_type,
            exclude_first_frame: true,
        }
    }

    /// Create a new particle animation with a single frame at the given position.
    pub fn simple_particle(x: i32, y: i32, particle_type: u32) -> ParticleAnimation {
        // Six frame to be sure that is visible
        Self {
            current_frame: 0,
            frames: vec![
                vec![(x, y)],
                vec![(x, y)],
                vec![(x, y)],
                vec![(x, y)],
                vec![(x, y)],
                vec![(x, y)],
            ],
            particle_type,
            exclude_first_frame: false,
        }
    }
}
