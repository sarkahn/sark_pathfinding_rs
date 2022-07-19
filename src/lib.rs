//! A simple implementation of the [astar pathfinding algorithm](https://www.redblobgames.com/pathfinding/a-star/implementation.html)
//! from red blob games.
//!
//! In order to use the pathfinder you must have a path map for it to navigate.
//!  You can define one by implementing the [PathingMap] trait, or you can use 
//! the built-in [PathMap2d].
//!
//! # Example
//!
//! ```rust
//! use sark_pathfinding::*;
//! 
//! // Create a 50x50 map with all nodes set to false (no obstacles)
//! let mut map = PathMap2d::default([50,50]);
//! 
//! // Set position 5,5 to block pathing
//! map[[5,5]] = true;
//! 
//! let mut astar = AStar::new(20);
//! let path = astar.find_path(&map, [4,4], [10,10]).unwrap();
//! ```

mod pathing_map;

use ahash::RandomState;
use std::collections::{binary_heap::BinaryHeap, HashMap};

pub use glam::IVec2;
pub use pathing_map::PathingMap;
pub use sark_grids::{Grid, GridPoint, Size2d};

/// Simple pathing map. Nodes can be accessed via 1d or 2d index.
/// 
/// # Example
/// 
/// ```
/// use sark_pathfinding::PathMap2d;
/// 
/// // Create a 30x30 map with all nodes set to false (not obstacles)
/// let mut map = PathMap2d::default([30,30]);
/// // Set first node to obstacle
/// map[0] = true;
/// 
/// // Set position (2,2) to obstacle
/// map[[2,2]] = true;
/// ```
pub type PathMap2d = Grid<bool>;

/// Struct for running the pathfinding algorithm
#[derive(Default)]
pub struct AStar {
    frontier: BinaryHeap<Cell>,
    parents: HashMap<IVec2, IVec2, RandomState>,
    costs: HashMap<IVec2, usize, RandomState>,
    path: Option<Vec<IVec2>>,
}

impl AStar {
    /// Create a new astar object for pathfinding.
    ///
    /// The `length` parameter determines the initial size of the internal containers. These
    /// containers will grow as needed when running the pathfinding algorithm, but setting a
    /// reasonable initial size could avoid performance issues from excessive allocations.
    pub fn new(len: usize) -> Self {
        Self {
            frontier: BinaryHeap::with_capacity(len),
            parents: HashMap::with_capacity_and_hasher(len / 2, RandomState::default()),
            costs: HashMap::with_capacity_and_hasher(len / 2, RandomState::default()),
            path: None,
        }
    }

    /// Construct an astar struct from the given size (width * height).
    ///
    /// The `size` parameter determines the initial size of the internal containers. These
    /// containers will grow as needed when running the pathfinding algorithm, but setting a
    /// reasonable initial size could avoid performance issues from excessive allocations.
    pub fn from_size(size: impl Size2d) -> Self {
        Self::new(size.len())
    }

    /// Find a path on the map. Returns `None` if no path could be found.
    ///
    /// A reference to the resulting path will be returned, the actual path will be stored in the [AStar] struct.
    /// The reference can retrieved via the `path()` function or the struct can be converted into `Vec<IVec2>`
    /// to take ownership.
    pub fn find_path(
        &mut self,
        map: &impl PathingMap,
        start: impl GridPoint,
        end: impl GridPoint,
    ) -> Option<&Vec<IVec2>> {
        let start = start.as_ivec2();
        let end = end.as_ivec2();

        self.frontier.push(Cell::new(start, 0));

        self.costs.insert(start, 0);

        while !self.frontier.is_empty() {
            let curr = self.frontier.pop().unwrap().value;

            if curr.eq(&end) {
                break;
            }

            for next in map.get_available_exits(curr) {
                let new_cost = self.costs[&curr] + map.get_cost(curr, next);

                let next_cost = self.costs.get(&next);

                if next_cost.is_none() || new_cost < *next_cost.unwrap() {
                    self.costs.insert(next, new_cost);
                    let priority = new_cost + map.get_distance(next, end);
                    self.frontier.push(Cell::new(next, priority));
                    self.parents.insert(next, curr);
                }
            }
        }

        if !self.parents.contains_key(&end) {
            return None;
        }

        let mut curr = end;

        let path = self.path.get_or_insert(Vec::new());
        path.clear();

        while !curr.eq(&start) {
            path.push(curr);
            curr = self.parents[&curr];
        }

        path.push(start);

        path.reverse();

        Some(path)
    }

    /// Clear internal data.
    pub fn clear(&mut self) {
        self.path = None;
        self.frontier.clear();
        self.parents.clear();
        self.costs.clear();
    }

    /// An iterator over all nodes visited during pathfinding.
    pub fn visited(&self) -> impl Iterator<Item = IVec2> + '_ {
        self.parents.iter().map(|(k, _)| k).cloned()
    }

    /// Retrieve the result of the pathfinding operation.
    ///
    /// `None` if `find_path` hasn't been called yet or if no path was found.
    pub fn path(&self) -> Option<&Vec<IVec2>> {
        self.path.as_ref()
    }
}

impl From<AStar> for Vec<IVec2> {
    fn from(astar: AStar) -> Self {
        astar.path.unwrap()
    }
}

#[derive(Eq)]
struct Cell {
    cost: usize,
    value: IVec2,
}

impl Cell {
    pub fn new(value: IVec2, cost: usize) -> Self {
        Self { value, cost }
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
        //.then_with(||other.value.cmp(&self.value))
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.value == other.value
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn right_test() {
        let map = PathMap2d::default([10, 10]);

        let mut astar = AStar::new(10 * 10);
        let path = astar.find_path(&map, [0, 0], [5, 0]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([0, 0], path[0].to_array());
        assert_eq!([5, 0], path[5].to_array());
    }

    #[test]
    fn down_test() {
        let map = PathMap2d::default([10, 10]);

        let mut astar = AStar::new(10 * 10);
        let path = astar.find_path(&map, [5, 5], [5, 0]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([5, 5], path[0].to_array());
        assert_eq!([5, 0], path[5].to_array());
    }

    #[test]
    fn up_test() {
        let map = PathMap2d::default([10, 10]);

        let mut astar = AStar::new(10 * 10);
        let path = astar.find_path(&map, [5, 4], [5, 9]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([5, 4], path[0].to_array());
        assert_eq!([5, 9], path[5].to_array());
    }

    #[test]
    fn left_test() {
        let map = PathMap2d::default([10, 10]);

        let mut astar = AStar::new(10 * 10);
        let path = astar.find_path(&map, [9, 5], [4, 5]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([9, 5], path[0].to_array());
        assert_eq!([4, 5], path[5].to_array());
    }
}
