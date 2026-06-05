//! A utility for pathfinding that supports several simple agorithms.

use ahash::{HashMap, HashMapExt};
use glam::IVec2;
use std::collections::hash_map::Entry;

use crate::{min_heap::MinHeap, pathmap::PathMap};

/// Utility for pathfinding that supports several simple algorithms.
///
/// Maintains internal state so it can be re-used to avoid allocations.
///
/// # Example
///
/// ```
/// use sark_pathfinding::*;
/// let mut pathmap = PathMap2d::new([10, 10]);
/// pathmap.add_obstacle([3,0]);
/// pathmap.add_obstacle([3,1]);
/// let mut pathfinder = Pathfinder::new();
/// let path = pathfinder.astar(&pathmap, [0, 0], [5, 0]).unwrap();
/// assert_eq!(6, path.len());
/// ```
#[derive(Default)]
pub struct Pathfinder {
    frontier: MinHeap,
    came_from: HashMap<IVec2, IVec2>,
    costs: HashMap<IVec2, i32>,
    path: Vec<IVec2>,
}

impl Pathfinder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new pathfinder with an initial capacity set for all internal
    /// data structures.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            frontier: MinHeap::with_capacity(capacity),
            came_from: HashMap::with_capacity(capacity),
            costs: HashMap::with_capacity(capacity),
            path: Vec::with_capacity(capacity / 4),
        }
    }

    /// Find a path to a goal using the [A*] algorithm.
    ///
    /// Returns a slice of points representing the path, or [None] if no path
    /// can be found.
    ///
    /// [A*]: https://www.redblobgames.com/pathfinding/a-star/introduction.html#astar
    ///
    /// # Example
    /// ```
    /// use sark_pathfinding::*;
    /// let mut pf = Pathfinder::new();
    /// let mut map = PathMap2d::new([10, 10]);
    /// map.add_obstacle([3,0]);
    /// map.add_obstacle([3,1]);
    /// let path = pf.astar(&map, [0, 0], [5, 0]).unwrap();
    /// assert_eq!(6, path.len());
    pub fn astar(
        &mut self,
        map: &impl PathMap,
        start: impl Into<IVec2>,
        goal: impl Into<IVec2>,
    ) -> Option<&[IVec2]> {
        self.clear();
        let start = start.into();
        let goal = goal.into();
        self.frontier.push(start, 0);
        self.costs.insert(start, 0);

        while let Some(curr) = self.frontier.pop() {
            if curr == goal {
                break;
            }

            for next in map.exits(curr) {
                let new_cost = self.costs[&curr] + map.cost(curr, next);
                if !self.costs.contains_key(&next) || new_cost < self.costs[&next] {
                    self.costs.insert(next, new_cost);
                    self.frontier
                        .push(next, new_cost + map.distance(goal, next));
                    self.came_from.insert(next, curr);
                }
            }
        }
        self.build_path(start, goal)
    }

    //     /// Returns an iterator of the valid exits from the given grid point.
    // fn exits(&self, p: impl Into<IVec2>) -> Self::ExitIterator;
    // /// The cost of moving between two grid points.
    // fn cost(&self, a: impl Into<IVec2>, b: impl Into<IVec2>) -> i32;
    // /// The distance between two grid points.
    // fn distance(&self, a: impl Into<IVec2>, b: impl Into<IVec2>) -> i32;
    // fn is_obstacle(&self, p: impl Into<IVec2>) -> bool;

    pub fn astar_fn<T: Iterator<Item = IVec2>>(
        &mut self,
        start: impl Into<IVec2>,
        goal: impl Into<IVec2>,
        cost: impl Fn(IVec2, IVec2) -> i32,
        dist: impl Fn(IVec2, IVec2) -> i32,
        exits: impl Fn(IVec2) -> T,
    ) -> Option<&[IVec2]> {
        self.clear();
        let start = start.into();
        let goal = goal.into();
        self.frontier.push(start, 0);
        self.costs.insert(start, 0);

        while let Some(curr) = self.frontier.pop() {
            if curr == goal {
                break;
            }

            for next in exits(curr) {
                let new_cost = self.costs[&curr] + cost(curr, next);
                if !self.costs.contains_key(&next) || new_cost < self.costs[&next] {
                    self.costs.insert(next, new_cost);
                    self.frontier.push(next, new_cost + dist(goal, next));
                    self.came_from.insert(next, curr);
                }
            }
        }
        self.build_path(start, goal)
    }

    /// Find a path to a goal using [Dijkstra's Algorithm]. Note that if
    /// the movement cost is uniform across your entire map then you are better
    /// off using [Pathfinder::bfs] instead as it will be faster and give
    /// the same results.
    ///
    /// If a `start` is specified, the algorithm will stop once it reaches the
    /// `goal`. Otherwise it will run until all possible nodes have been visited.
    /// Afterwards, a path can be retrieved via [Pathfinder::build_path].
    ///
    /// Note this is seperate from [crate::DijkstraMap] which is used for calculating
    /// a path to the most valuable goal on a grid.
    ///
    /// [Dijkstra's Algorithm]: https://www.redblobgames.com/pathfinding/a-star/introduction.html#dijkstra
    pub fn dijkstra(
        &mut self,
        map: &impl PathMap,
        start: Option<impl Into<IVec2>>,
        goal: impl Into<IVec2>,
    ) {
        self.clear();

        let start = start.map(|s| s.into());
        let goal = goal.into();

        let p = start.unwrap_or(goal);
        self.frontier.push(p, 0);
        self.costs.insert(p, 0);

        while let Some(curr) = self.frontier.pop() {
            if start.is_some() && curr == goal {
                break;
            }
            for next in map.exits(curr) {
                let new_cost = self.costs[&curr] + map.cost(curr, next);

                let next_cost = self.costs.get(&next);
                if next_cost.is_none() || new_cost < *next_cost.unwrap() {
                    self.costs.insert(next, new_cost);
                    self.frontier.push(next, new_cost);
                    self.came_from.insert(next, curr);
                }
            }
        }
    }

    /// Generate paths to a goal using a [Breadth First Search] algorithm.
    ///
    /// If a `start` is specified, the algorithm will stop once it reaches the
    /// `goal`. Otherwise it will run until all possible nodes have been visited.
    /// Afterwards, a path can be retrieved via [Pathfinder::build_path].
    ///
    /// [Breadth First Search]: https://www.redblobgames.com/pathfinding/a-star/introduction.html#breadth-first-search
    pub fn bfs(
        &mut self,
        map: &impl PathMap,
        start: Option<impl Into<IVec2>>,
        goal: impl Into<IVec2>,
    ) {
        self.clear();

        let start = start.map(|s| s.into());
        let goal = goal.into();

        let p = start.unwrap_or(goal);
        self.frontier.push(p, 0);

        while let Some(curr) = self.frontier.pop() {
            if start.is_some() && curr == goal {
                break;
            }
            for next in map.exits(curr) {
                if let Entry::Vacant(_) = self.came_from.entry(next) {
                    self.frontier.push(next, self.frontier.len() as i32);
                    self.came_from.insert(next, curr);
                }
            }
        }
    }

    /// Attempt to construct a path from start to goal from the previously
    /// populated path data. This function will only work once one of the
    /// pathfinding functions have been used: [Pathfinder::astar], [Pathfinder::dijkstra],
    /// or [Pathfinder::bfs].
    ///
    /// This function is called automatically by [Pathfinder::astar], and the
    /// resulting path can be retrieved directly from that function or via
    /// [Pathfinder::path].
    ///
    /// Returns the constructed path as a slice, or [None] if no pathfinding
    /// functions have been run or if no valid path exists between the given
    /// points.
    pub fn build_path(
        &mut self,
        start: impl Into<IVec2>,
        goal: impl Into<IVec2>,
    ) -> Option<&[IVec2]> {
        let mut curr = goal.into();
        let start = start.into();
        self.path.clear();
        self.path.push(curr);
        while let Some(next) = self.came_from.get(&curr) {
            self.path.push(*next);
            if *next == start {
                self.path.reverse();
                return Some(self.path.as_slice());
            }
            curr = *next;
        }
        None
    }

    /// Clear all internal data.
    pub fn clear(&mut self) {
        self.frontier.clear();
        self.came_from.clear();
        self.costs.clear();
        self.path.clear();
    }

    /// An iterator over all nodes visited during pathfinding.
    pub fn visited(&self) -> impl Iterator<Item = &IVec2> {
        self.came_from.keys()
    }

    /// Retrieve a reference to the `came_from` map which is populated during
    /// pathfinding operations.
    pub fn came_from(&self) -> &HashMap<IVec2, IVec2> {
        &self.came_from
    }

    /// Retrieve a reference to the `costs` map which is populated during
    /// pathfinding operations.
    pub fn costs(&self) -> &HashMap<IVec2, i32> {
        &self.costs
    }

    /// Retrieve a slice of the most recently built path data. If no path
    /// has been built, the slice will be empty.
    pub fn path(&self) -> &[IVec2] {
        &self.path
    }

    pub fn take_path(self) -> Vec<IVec2> {
        self.path
    }
}

#[cfg(test)]
mod test {
    use crate::{
        DEFAULT_CARDINAL_COST, DEFAULT_DIAGONAL_COST, DIR_8, PathMap2d, is_cardinal,
        octile_heuristic,
    };

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

    #[test]

    fn astar_fn_test() {
        let mut finder = Pathfinder::new();

        let w: i32 = 25;
        let h: i32 = 25;

        let index = |p: IVec2| (p.y * w + p.x) as usize;

        let obstacles = vec![false; (w * h) as usize];

        let obstacles_ref = &obstacles;
        let exits = |a: IVec2| {
            DIR_8.iter().filter_map(move |p| {
                let next = a + *p;
                if next.x < 0 || next.x >= w || next.y < 0 || next.y >= h {
                    return None;
                }
                let i = index(next);
                if !obstacles_ref[i] { Some(next) } else { None }
            })
        };

        let ccost = DEFAULT_CARDINAL_COST;
        let dcost = DEFAULT_DIAGONAL_COST;

        let cost = |a: IVec2, b: IVec2| {
            if is_cardinal(a, b) { ccost } else { dcost }
        };
        let dist = |a: IVec2, b: IVec2| octile_heuristic(a, b, ccost, dcost);

        let start = IVec2::new(0, 0);
        let goal = IVec2::new(4, 4);
        let path = finder
            .astar_fn(start, goal, cost, dist, exits)
            .expect("Invalid path");

        assert_eq!(5, path.len());
        assert_eq!([0, 0], path[0].to_array());
        assert_eq!([1, 1], path[1].to_array());
        assert_eq!([2, 2], path[2].to_array());
        assert_eq!([3, 3], path[3].to_array());
        assert_eq!([4, 4], path[4].to_array());
    }
}
