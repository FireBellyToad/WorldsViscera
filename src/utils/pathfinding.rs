use pathfinding::prelude::dijkstra;

use crate::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    maps::zone::Zone,
    utils::roll::Roll,
};

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
        move_only_in_water: bool,
    ) -> Option<(Vec<(i32, i32)>, u32)> {
        //Calling dijkstra and get result
        dijkstra(
            // Starting point
            &(origin_x, origin_y),
            // Must return all the passable adjacent squares form a x,y point.
            // .map(|p| (p, 1)) associate a pathfinding cost of 1 for each square
            // new not-passable tiles must be implemented inside "zone.get_adjacent_passable_tiles(x, y)"
            |&(x, y)| {
                zone.get_adjacent_passable_tiles(x, y, use_manhattan_distance, move_only_in_water)
                    .into_iter()
                    .map(|passable_tile| (passable_tile, 1))
            },
            // Should tell when the goal is reached
            |&position: &(i32, i32)| position.0 == goal_x && position.1 == goal_y,
        )
    }

    pub fn snake_path(
        origin_x: i32,
        origin_y: i32,
        goal_x: i32,
        goal_y: i32,
    ) -> Option<(Vec<(i32, i32)>, u32)> {
        let mut virtual_blocked_tiles = vec![false; (MAP_WIDTH * MAP_HEIGHT) as usize];

        // Initialize blocked tiles randomly
        for tile in virtual_blocked_tiles.iter_mut() {
            *tile = Roll::dice(1, 4) == 1;
        }

        //Calling dijkstra and get result
        dijkstra(
            // Starting point
            &(origin_x, origin_y),
            // Must return all the passable adjacent squares form a x,y point.
            // .map(|p| (p, 1)) associate a pathfinding cost of 1 for each square
            // new not-passable tiles must be implemented inside "zone.get_adjacent_passable_tiles(x, y)"
            |&(x, y)| Pathfinding::recursive_random_block(x, y, &virtual_blocked_tiles),
            // Should tell when the goal is reached
            |&position: &(i32, i32)| position.0 == goal_x && position.1 == goal_y,
        )
    }

    /// Casual random block generator: generate 1 or more passable tile casually to have a snake
    fn recursive_random_block(
        x_pos: i32,
        y_pos: i32,
        virtual_blocked_tiles: &Vec<bool>,
    ) -> Vec<((i32, i32), u32)> {
        let mut successors = Vec::new();
        for x in x_pos - 1..=x_pos + 1 {
            for y in y_pos - 1..=y_pos + 1 {
                if !virtual_blocked_tiles[Zone::get_index_from_xy(&x, &y)] {
                    successors.push(((x, y), 1));
                }
            }
        }

        // Fallback
        if successors.is_empty() {
            for x in x_pos - 1..=x_pos + 1 {
                for y in y_pos - 1..=y_pos + 1 {
                    successors.push(((x, y), 1));
                }
            }
        }
        successors
    }
}
