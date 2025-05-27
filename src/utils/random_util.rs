pub struct RandomUtils {}

impl RandomUtils {

    pub fn dice(number: i32, size: i32) -> i32{
        let mut result = 0;
         
        for throw in 0..number {
            result += gen_range(1,size);
        }

        result
    }
}