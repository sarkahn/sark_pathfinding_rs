pub mod pathing_map;

use std::{collections::{binary_heap::BinaryHeap, HashMap}, hash::Hash};

use ahash::RandomState;
pub use pathing_map::PathingMap;

#[derive(Eq)]
struct Cell<T: Eq + Hash + Copy> {
    cost: usize,
    value: T,
}

impl<T: Eq + Hash + Copy> Cell<T> {
    pub fn new(value: T, cost: usize) -> Self {
        Self {
            value,
            cost,
        }
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
        //.then_with(||other.value.cmp(&self.value))
    }
}

impl<T: Eq + Hash + Copy> PartialEq for Cell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.value == other.value
    }
}

#[derive(Default)]
pub struct AStar<T: Eq + Hash + Copy> {
    frontier: BinaryHeap<Cell<T>>,
    parents: HashMap<T,T, RandomState>,
    costs: HashMap<T,usize, RandomState>,
    path: Option<Vec<T>>,
}

impl <T: Eq + Hash + Copy> AStar<T> {
    pub fn new(len: usize) -> Self {
        Self {
            frontier: BinaryHeap::with_capacity(len),
            parents: HashMap::with_capacity_and_hasher(len / 2, RandomState::default()),
            costs: HashMap::with_capacity_and_hasher(len / 2, RandomState::default()),
            path: None,
        }
    }

    pub fn from_size(size: [u32;2]) -> Self {
        Self::new(size[0] as usize * size[1] as usize)
    }

    /// Find a path on the map. Returns `None` if no path could be found.
    /// 
    /// A reference to the resulting path will be returned, the actual path will be stored in the [AStar] struct. 
    /// The reference can retrieved via the `path()` function or the struct can be converted into `Vec<T>` 
    /// to take ownership.
    pub fn find_path(&mut self, map: &impl PathingMap<T>, start: T, end: T) -> Option<&Vec<T>> {
        self.frontier.push(Cell::new(start, 0));

        self.costs.insert(start, 0);

        while self.frontier.len() > 0 {
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
    pub fn visited(&self) -> impl Iterator<Item=T> + '_ {
        self.parents.iter().map(|(k,_)|k).cloned()
    }

    /// Retrieve the result of the pathfinding operation. 
    /// 
    /// `None` if `find_path` hasn't been called yet or if no path was found.
    pub fn path(&self) -> Option<&Vec<T>> {
        self.path.as_ref()
    }
}

impl<T: Ord + Eq + Hash + Copy> From<AStar<T>> for Vec<T> {
    fn from(astar: AStar<T>) -> Self {
        astar.path.unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::pathing_map::{PathMap2d};

    use super::*;

    #[test]
    fn right_test() {
        let map = PathMap2d::new([10,10]);
        
        let mut astar = AStar::new(10 * 10);
        let path = astar.find_path(&map, [0,0], [5,0]).unwrap();

        assert_eq!(6, path.len());
        assert_eq!([0,0], path[0]);
    }

    #[test]
    fn down_test() {
        let map = PathMap2d::new([10,10]);
        
        let mut astar = AStar::new(10 * 10);
        let path = astar.find_path(&map, [5,5], [5,0]).unwrap();

        assert_eq!(6, path.len());
    }
    
    #[test]
    fn up_test() {
        let map = PathMap2d::new([10,10]);
        
        let mut astar = AStar::new(10 * 10);
        let path = astar.find_path(&map, [5,4], [5,9]).unwrap();

        assert_eq!(6, path.len());
    }

    #[test]
    fn left_test() {
        let map = PathMap2d::new([10,10]);
        
        let mut astar = AStar::new(10 * 10);
        let path = astar.find_path(&map, [9,5], [4,5]).unwrap();

        assert_eq!(6, path.len());
    }

}