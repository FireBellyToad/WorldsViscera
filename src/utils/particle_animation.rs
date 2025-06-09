use bresenham::Bresenham;

pub struct ParticleAnimation {
    pub current_frame: usize,
    pub frames: Vec<Vec<(i32, i32)>>,
}

impl ParticleAnimation {
    pub fn new_line(start: (i32, i32), end: (i32, i32)) -> ParticleAnimation {

        //FIXME maybe we could create the line of effect outiside, so we can use it in real gameplay?
        // convert start and end
        let start_isize = (start.0 as isize, start.1 as isize);
        let end_isize = (end.0 as isize, end.1 as isize);

        // get a Bresenham line from start and end and convert it to (i32,i32)
        let mut line_frames: Vec<_> = Bresenham::new(start_isize, end_isize)
            .map(|(x, y)| (x as i32, y as i32))
            .collect();

        //Add last point and remove start point
        line_frames.push(end);
        line_frames = line_frames.drain(1..).collect();

        // Prepare all the "frames" that will be rendered.
        // this means that we will create a vector of (x,y) for each frame,
        // to give the impression of a growing ray.
        let total_frames = line_frames.len();
        let mut frames: Vec<Vec<(i32, i32)>> = Vec::new();
        let mut loaded_frame_renders: usize = 0;
        let mut pointer: usize = 0;

        for _ in 0..total_frames {
            let mut frame_renders: Vec<(i32, i32)> = Vec::new();
            while pointer <= loaded_frame_renders {
                frame_renders.push(line_frames[pointer]);
                pointer += 1;
            }

            loaded_frame_renders += 1;
            pointer = 0;
            frames.push(frame_renders);
        }

        // Return the particle line effect
        Self {
            current_frame: 0,
            frames,
        }
    }
}
