use macroquad::rand::gen_range;

pub struct Roll {}

/// Random Utils to wrap macroquad::rand
impl Roll {
    /// Roll a size-faced number of dice
    pub fn dice(number: i32, size: i32) -> i32 {
        let mut result = 0;

        for _ in 0..number {
            result += gen_range(1, size + 1);
        }

        result
    }

    /// Roll a d20. Will be used a lot
    pub fn d20() -> i32 {
        Roll::dice(1, 20)
    }

    /// Roll a d6. Will be used a lot
    pub fn d6() -> i32 {
        Roll::dice(1, 6)
    }

    /// Roll a d100. Will be used a lot
    pub fn d100() -> i32 {
        Roll::dice(1, 100)
    }

    /// Roll 3d6. Will be used a lot
    pub fn stat() -> i32 {
        Roll::dice(3, 6)
    }
}
