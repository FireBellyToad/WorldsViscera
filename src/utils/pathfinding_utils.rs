use pathfinding::prelude::astar;

use crate::map::Map;

pub struct PathfindingUtils {}

impl PathfindingUtils {
    /// Wrapper to pathfinding::astar
    /// Returns Option<(Vec<(i32, i32)>, u32)> which is:
    /// None -> No path found
    /// Some -> a Vector of the path nodes (including start and goal) and the total cost of the path
    pub fn a_star_wrapper(
        origin_x: i32,
        origin_y: i32,
        goal_x: i32,
        goal_y: i32,
        map: &Map,
    ) -> Option<(Vec<(i32, i32)>, u32)> {
        //Calling A start ancd get result
        astar(
            // Starting point
            &(origin_x, origin_y),
            // Must return all the passable adjacent squares form a x,y point.
            // .map(|p| (p, 1)) associate a pathfinding cost of 1 for each square
            |&(x, y)| {
                map.get_adjacent_passable_tiles(x, y)
                    .into_iter()
                    .map(|passable_tile| (passable_tile, 1))
            },
            // Calculate moving cost
            |&(x, y)| ((goal_x.abs_diff(x) + goal_y.abs_diff(y)) / 3),
            // Should tell when the goal is reached
            |&position: &(i32, i32)| position.0 == goal_x && position.1 == goal_y,
        )
    }
}
