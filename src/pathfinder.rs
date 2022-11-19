use ahash::RandomState;
use glam::IVec2;
use sark_grids::GridPoint;
use std::{
    collections::{binary_heap::BinaryHeap, hash_map::Entry, HashMap},
    hash::Hash,
};

use crate::pathmap::PathMap;

/// Struct for running pathfinding algorithms.
/// 
/// Maintains internal state so it can be re-used to avoid allocations.
#[derive(Default)]
pub struct Pathfinder {
    frontier: BinaryHeap<Cell<IVec2>>,
    parents: HashMap<IVec2, IVec2, RandomState>,
    costs: HashMap<IVec2, i32, RandomState>,
    path: Vec<IVec2>,
}

impl Pathfinder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new pathfinder.
    ///
    /// The `length` parameter determines the initial size of the internal 
    /// containers. These containers will grow as needed when running the 
    /// pathfinding algorithm, but setting a reasonable initial size could avoid 
    /// performance issues from excessive allocations.
    pub fn with_capacity(len: usize) -> Self {
        Self {
            frontier: BinaryHeap::with_capacity(len / 4),
            parents: HashMap::with_capacity_and_hasher(len / 4, RandomState::default()),
            costs: HashMap::with_capacity_and_hasher(len / 4, RandomState::default()),
            path: Vec::with_capacity(len / 4),
        }
    }

    /// Find a path on the given [PathMap]. Returns `None` if no path could be 
    /// found.
    pub fn astar(
        &mut self,
        map: &impl PathMap,
        start: impl GridPoint,
        end: impl GridPoint,
    ) -> Option<&[IVec2]> {
        self.clear();
        let start = start.as_ivec2();
        let end = end.as_ivec2();
        self.frontier.push(Cell::new(start, 0));
        self.costs.insert(start, 0);

        while !self.frontier.is_empty() {
            let curr = self.frontier.pop().unwrap().value;

            if curr == end {
                break;
            }

            for next in map.exits(curr) {
                let new_cost = self.costs[&curr] + map.cost(curr, next);

                let next_cost = self.costs.get(&next);

                if next_cost.is_none() || new_cost < *next_cost.unwrap() {
                    self.costs.insert(next, new_cost);
                    let priority = new_cost + map.distance(next, end) as i32;
                    self.frontier.push(Cell::new(next, priority));
                    self.parents.insert(next, curr);
                }
            }
        }

        self.path.clear();
        if !self.parents.contains_key(&end) {
            return None;
        }

        let mut curr = end;

        while !curr.eq(&start) {
            self.path.push(curr);
            curr = self.parents[&curr];
        }

        self.path.push(start);

        self.path.reverse();

        Some(self.path.as_slice())
    }

    /// Generate a dijsktra map from the given point.
    pub fn dijkstra(&mut self, map: &impl PathMap, origin: impl GridPoint) {
        let start = origin.as_ivec2();
        self.frontier.push(Cell::new(start, 0));
        self.costs.insert(start, 0);

        while !self.frontier.is_empty() {
            let curr = self.frontier.pop().unwrap().value;

            for next in map.exits(curr) {
                let new_cost = self.costs[&curr] + map.cost(curr, next);

                let next_cost = self.costs.get(&next);
                if next_cost.is_none() || new_cost < *next_cost.unwrap() {
                    self.costs.insert(next, new_cost);
                    self.frontier.push(Cell::new(next, new_cost));
                    self.parents.insert(next, curr);
                }
            }
        }
    }

    /// Perform a breadth-first search from the given point. The map of visited
    /// points is populated in `parents()`.
    pub fn bfs(&mut self, map: &impl PathMap, origin: impl GridPoint) {
        let start = origin.as_ivec2();
        self.path.push(start);

        while !self.path.is_empty() {
            let curr = self.path.pop().unwrap();
            for next in map.exits(curr) {
                if let Entry::Vacant(_) = self.parents.entry(next) {
                    self.path.push(next);
                    self.parents.insert(next, curr);
                }
            }
        }
    }

    /// Clear internal data.
    pub fn clear(&mut self) {
        self.path.clear();
        self.frontier.clear();
        self.parents.clear();
        self.costs.clear();
    }

    /// An iterator over all nodes visited during pathfinding.
    pub fn visited(&self) -> impl Iterator<Item = &IVec2> {
        self.parents.keys()
    }

    /// Retrieve a reference to the `parents` map which is populated during 
    /// pathfinding operations.
    pub fn parents(&self) -> &HashMap<IVec2, IVec2, RandomState> {
        &self.parents
    }

    /// Retrieve a reference to the `costs` map which is populated during 
    /// pathfinding operations.
    pub fn costs(&self) -> &HashMap<IVec2, i32, RandomState> {
        &self.costs
    }

    /// Retrieve the `path` which is populated during pathfinding operations.
    pub fn path(&self) -> &[IVec2] {
        self.path.as_slice()
    }
}

impl From<Pathfinder> for Vec<IVec2> {
    fn from(pf: Pathfinder) -> Self {
        pf.path
    }
}

#[derive(Eq)]
struct Cell<T: Eq + Hash + Copy> {
    cost: i32,
    value: T,
}

impl<T: Eq + Hash + Copy> Cell<T> {
    pub fn new(value: T, cost: i32) -> Self {
        Self { value, cost }
    }
}

impl<T: Eq + Hash + Copy> PartialOrd for Cell<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Eq + Hash + Copy> Ord for Cell<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl<T: Eq + Hash + Copy> PartialEq for Cell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.value == other.value
    }
}

#[cfg(test)]
mod test {
    use crate::PathMap2d;

    use super::*;

    #[test]
    fn right_test() {
        let map = PathMap2d::new([10, 10]);

        let mut pf = Pathfinder::new();
        let path = pf.astar(&map, [0, 0], [5, 0]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([0, 0], path[0].to_array());
        assert_eq!([5, 0], path[5].to_array());
    }

    #[test]
    fn down_test() {
        let map = PathMap2d::new([10, 10]);

        let mut astar = Pathfinder::new();
        let path = astar.astar(&map, [5, 5], [5, 0]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([5, 5], path[0].to_array());
        assert_eq!([5, 0], path[5].to_array());
    }

    #[test]
    fn up_test() {
        let map = PathMap2d::new([10, 10]);

        let mut astar = Pathfinder::new();
        let path = astar.astar(&map, [5, 4], [5, 9]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([5, 4], path[0].to_array());
        assert_eq!([5, 9], path[5].to_array());
    }

    #[test]
    fn left_test() {
        let map = PathMap2d::new([10, 10]);

        let mut astar = Pathfinder::new();
        let path = astar.astar(&map, [9, 5], [4, 5]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([9, 5], path[0].to_array());
        assert_eq!([4, 5], path[5].to_array());
    }
}
