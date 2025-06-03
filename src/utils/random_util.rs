use macroquad::rand::gen_range;

pub struct RandomUtils {}


/// Random Utils to wrap macroquad::rand
impl RandomUtils {

    /// Roll a size-faced number of dice
    pub fn dice(number: i32, size: i32) -> i32{
        let mut result = 0;
         
        for _throw in 0..number {
            result += gen_range(1,size+1);
        }

        result
    }

    /// Roll a d20. Will be used a lot
    pub fn d20() -> i32 {
        Self::dice(1,20)
    }

    /// Roll a d6. Will be used a lot
    pub fn d6() -> i32 {
        Self::dice(1,6)
    }


    /// Roll 3d6. Will be used a lot
    pub fn stat_roll() -> i32 {
        Self::dice(3,6)
    }
}