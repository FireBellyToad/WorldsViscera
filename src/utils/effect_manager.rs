use bresenham::Bresenham;

pub struct EffectManager {}

impl EffectManager {

    //Gets a line of effect
    pub fn new_line(start: (i32, i32), end: (i32, i32)) -> Vec<(i32, i32)> {
        // convert start and end
        let start_isize = (start.0 as isize, start.1 as isize);
        let end_isize = (end.0 as isize, end.1 as isize);

        // get a Bresenham line from start and end and convert it to (i32,i32)
        let mut line: Vec<_> = Bresenham::new(start_isize, end_isize)
            .map(|(x, y)| (x as i32, y as i32))
            .collect();

        //Add last point and remove start point
        line.push(end);
        line = line.drain(1..).collect();

        line
    }
}
