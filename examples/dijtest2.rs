use glam::{IVec2, UVec2};
use sark_pathfinding::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const WALL: i32 = -10000;
const FLOOR: i32 = 10000;

#[derive(Debug, Eq, PartialEq)]
struct Cell {
    position: IVec2,
    cost: i32,
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost) // Reverse for min-heap behavior
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

struct Map {
    cost: Vec<i32>,
    size: UVec2,
}

impl Map {
    fn new(size: UVec2) -> Self {
        Self {
            cost: vec![WALL; size.x as usize * size.y as usize],
            size,
        }
    }

    fn add_goal(&mut self, xy: IVec2, value: i32) {
        self.cost[xy_to_index(xy, self.size.x as usize)] = value;
    }

    fn add_floor(&mut self, xy: IVec2) {
        self.cost[xy_to_index(xy, self.size.x as usize)] = FLOOR;
    }

    fn compute_dijkstra_map(&mut self) {
        let mut heap = BinaryHeap::new();

        // Initialize the heap with all goal cells
        for y in 0..self.size.y as i32 {
            for x in 0..self.size.x as i32 {
                let pos = IVec2::new(x, y);
                let index = xy_to_index(pos, self.size.x as usize);
                if self.cost[index] == 0 {
                    heap.push(Reverse(Cell {
                        position: pos,
                        cost: 0,
                    }));
                }
            }
        }

        // Directions for cardinal neighbors: up, down, left, right
        let directions = [
            IVec2::new(0, -1),
            IVec2::new(0, 1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
        ];

        while let Some(Reverse(Cell { position, cost })) = heap.pop() {
            for &dir in &directions {
                let neighbor_pos = position + dir;

                // Skip out-of-bounds neighbors
                if !self.in_bounds(neighbor_pos) {
                    continue;
                }

                let neighbor_index = xy_to_index(neighbor_pos, self.size.x as usize);

                // Skip walls
                if self.cost[neighbor_index] == WALL {
                    continue;
                }

                // Calculate the new cost for the neighbor
                let new_cost = cost + 1;

                // Update if the new cost is better
                if new_cost < self.cost[neighbor_index] {
                    self.cost[neighbor_index] = new_cost;
                    heap.push(Reverse(Cell {
                        position: neighbor_pos,
                        cost: new_cost,
                    }));
                }
            }
        }
    }

    fn in_bounds(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.size.x as i32 && pos.y < self.size.y as i32
    }
}

fn xy_to_index(xy: IVec2, width: usize) -> usize {
    xy.y as usize * width + xy.x as usize
}

fn main() {
    let size = UVec2::new(5, 5);
    let mut map = Map::new(size);

    // Define some floors and goals
    map.add_floor(IVec2::new(0, 0));
    map.add_floor(IVec2::new(1, 0));
    map.add_floor(IVec2::new(1, 1));
    map.add_goal(IVec2::new(2, 2), 0);
    map.add_floor(IVec2::new(2, 3));
    map.add_floor(IVec2::new(3, 3));

    // Compute the Dijkstra map
    map.compute_dijkstra_map();

    // Print the resulting map costs
    for y in (0..size.y).rev() {
        for x in 0..size.x {
            let index = xy_to_index(IVec2::new(x as i32, y as i32), size.x as usize);
            print!("{:>6} ", map.cost[index]);
        }
        println!();
    }
}
