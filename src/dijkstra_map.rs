//! A simple implementation of a "Dijkstra Map" as described in https://www.roguebasin.com/index.php/Dijkstra_Maps_Visualized

use crate::{min_heap::MinHeap, PathMap};
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use arrayvec::{ArrayVec, IntoIter};
use glam::{IVec2, UVec2};
use sark_grids::{BitGrid, FloatGrid, GridPoint, GridSize, SizedGrid};

/// A simple implementation of a "Dijkstra Map" as described in https://www.roguebasin.com/index.php/Dijkstra_Maps_Visualized
///
/// Various 'goals' can be defined with specific values. Once the map is
/// recalculated, a path can be followed from any valid position to find the
/// most 'desired' goal from that position.
#[derive(Debug, Default, Clone)]
pub struct DijkstraMap {
    values: FloatGrid,
    goals: HashSet<IVec2>,
    obstacles: BitGrid,
    frontier: MinHeap<f32>,
    came_from: HashMap<IVec2, IVec2>,
    path: Option<Vec<IVec2>>,
}

impl DijkstraMap {
    pub fn new(size: impl GridSize) -> Self {
        let mut values = FloatGrid::new([0, 0], size.to_uvec2());
        values.set_all(f32::MAX);
        Self {
            values,
            goals: HashSet::new(),
            frontier: MinHeap::with_capacity(size.tile_count()),
            came_from: HashMap::with_capacity(size.tile_count()),
            obstacles: BitGrid::new(size),
            path: Some(Vec::new()),
        }
    }

    /// Add a 'goal' to a map position. `value` determines the desirability of
    /// the goal during pathfinding, where higher values goals are more desireable,
    /// higher value value goals could be preferred over lower value goals at
    /// the same or closer distances.
    ///
    /// If you add to an existing goal, the value will be added to the previously
    /// set value.
    pub fn add_goal(&mut self, xy: impl GridPoint, value: f32) {
        let xy = xy.to_ivec2();
        if self.goals.contains(&xy) {
            let v = self.values.value(xy);
            self.values.set_value(xy, v + value);
        } else {
            self.values[xy] = value;
            self.goals.insert(xy);
        }
    }

    /// Remove a goal. This will not affect any previously set value
    /// for that goal's tile.
    pub fn remove_goal(&mut self, xy: impl GridPoint) {
        self.goals.remove(&xy.to_ivec2());
    }

    /// An iterator over any goals set for the map, and the value for each goal.
    pub fn goals(&self) -> impl Iterator<Item = (IVec2, f32)> + '_ {
        self.goals.iter().map(|p| (*p, self.values.value(*p)))
    }

    /// Recalculate the map based on the given pathing.
    ///
    /// This will not clear any currently set map values and will overwrite previous
    /// values based on the current state of set goals. To recalculate a clean
    /// map based only on the currently set goals, call [DijkstraMap::clear_values]
    /// to clear any previously set non-goal tiles.
    pub fn recalculate(&mut self, pathing: &impl PathMap) {
        self.obstacles.set_all(true);
        self.came_from.clear();
        self.frontier.clear();

        for goal in self.goals.iter() {
            let v = self.values.value(*goal);
            self.frontier.push(*goal, v);
            self.came_from.insert(*goal, *goal);
        }

        while let Some(curr) = self.frontier.pop() {
            for next in pathing.exits(curr) {
                let new_cost = self.values.value(curr) + pathing.cost(curr, next) as f32;
                self.obstacles.set(next, false);
                if !self.came_from.contains_key(&next) || new_cost < self.values.value(next) {
                    self.values.set_value(next, new_cost);
                    self.frontier.push(next, new_cost);
                    self.came_from.insert(next, curr);
                }
            }
        }
    }

    /// Resets the value of all non-goal tiles.
    pub fn clear_values(&mut self) {
        for (p, v) in self.values.iter_grid_points().zip(self.values.values_mut()) {
            if !self.goals.contains(&p) {
                *v = 0.0;
            }
        }
    }

    /// Clears all goals and values from the map, resetting all tiles values to 0.
    pub fn clear_all(&mut self) {
        self.obstacles.set_all(true);
        self.goals.clear();
        self.values.clear();
    }

    /// Apply a mathematical operation to every value in the grid.
    pub fn apply_operation(&mut self, operation: impl Fn(f32) -> f32) {
        self.values.apply_operation(operation);
    }

    /// A reference to the [DijkstraMap]'s underlying [FloatGrid].
    pub fn float_grid(&self) -> &FloatGrid {
        &self.values
    }

    /// A mutable reference to the [DijkstraMap]'s underlying [FloatGrid].
    pub fn float_grid_mut(&mut self) -> &mut FloatGrid {
        &mut self.values
    }

    /// Retrieve an iterator over the valid exits from a given position on the [DijkstraMap].
    ///
    /// The exits are sorted by 'value', so the first exit should be considered
    /// the most 'valuable'. The actual values can be retrieved via [DijkstraMap::exit_values].
    pub fn exits(&self, xy: impl GridPoint) -> impl Iterator<Item = IVec2> {
        self.exit_values(xy).map(|pv| pv.0)
    }

    /// Retrieve an iterator over the valid exits from a given position on the [DijkstraMap].
    ///
    /// Returns an iterator of 2 element tuples, where each tuple contains a position
    /// and it's corresponding 'value' in the [DijkstraMap]. The exits
    /// are sorted by 'most valuable' first before being returned.
    pub fn exit_values(&self, xy: impl GridPoint) -> IntoIter<(IVec2, i32), 8> {
        let xy = xy.to_ivec2();
        let mut v = ArrayVec::new();
        for next in xy.adj_8() {
            let Some(i) = self.try_transform_lti(next) else {
                continue;
            };
            if self.obstacles.get_index(i) {
                continue;
            }
            v.push((next, self.values[i] as i32));
        }
        //v.sort_unstable_by_key(|f| f.1);
        v.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        v.into_iter()
    }

    pub fn values(&self) -> &[f32] {
        self.values.values()
    }
}

impl SizedGrid for DijkstraMap {
    fn size(&self) -> UVec2 {
        self.values.size()
    }
}

#[cfg(test)]
mod tests {
    use glam::{ivec2, IVec2};

    use crate::GridPoint;

    use super::DijkstraMap;

    struct PathFunctionHaver {
        fun: fn(IVec2, IVec2) -> bool,
    }

    #[test]
    fn pathing() {
        let mut grid = DijkstraMap::new([100, 100]);
        // let finder = PathFinder {
        //     path_func: |a: IVec2, b: IVec2| true,
        // };

        // assert!(finder.is_valid_move([0, 0], [10, 10]));
        // let can_path = |a: IVec2, b: IVec2| {
        //     let a = a.as_ivec2();
        //     let dir = b.as_ivec2() - a;
        //     if dir.x != 0 && dir.y != 0 {
        //         let x_obstacle = grid.is_obstacle(a + ivec2(dir.x, 0));
        //         let y_obstacle = grid.is_obstacle(a + ivec2(0, dir.y));
        //         return x_obstacle || y_obstacle;
        //     }
        //     true
        // };
    }
}
