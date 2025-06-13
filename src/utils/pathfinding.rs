use pathfinding::prelude::dijkstra;

use crate::maps::zone::Zone;

pub struct Pathfinding {}

impl Pathfinding {
    /// Wrapper to pathfinding::dijkstra
    /// Returns Option<(Vec<(i32, i32)>, u32)> which is:
    /// None -> No path found
    /// Some -> a tuple containing: 0 -> Vector of the path nodes (including start and goal) 1 -> the total cost of the path
    pub fn dijkstra_wrapper(
        origin_x: i32,
        origin_y: i32,
        goal_x: i32,
        goal_y: i32,
        zone: &Zone,
        use_manhattan_distance: bool,
    ) -> Option<(Vec<(i32, i32)>, u32)> {
        //Calling dijkstra and get result
        dijkstra(
            // Starting point
            &(origin_x, origin_y),
            // Must return all the passable adjacent squares form a x,y point.
            // .map(|p| (p, 1)) associate a pathfinding cost of 1 for each square
            // new not-passable tiles must be implemented inside "zone.get_adjacent_passable_tiles(x, y)"
            |&(x, y)| {
                zone.get_adjacent_passable_tiles(x, y, use_manhattan_distance)
                    .into_iter()
                    .map(|passable_tile| (passable_tile, 1))
            },
            // Should tell when the goal is reached
            |&position: &(i32, i32)| position.0 == goal_x && position.1 == goal_y,
        )
    }
}
