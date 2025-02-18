//! A simple implementation of a "Dijkstra Map" as described in https://www.roguebasin.com/index.php/Dijkstra_Maps_Visualized

use crate::{min_heap::MinHeap, PathMap};
use ahash::{HashSet, HashSetExt};
use arrayvec::{ArrayVec, IntoIter};
use glam::{IVec2, UVec2};
use sark_grids::{BitGrid, FloatGrid, GridPoint, GridSize, SizedGrid};

const INITIAL_VALUE: f32 = 1000.0;
const EXIT_CAP: usize = 8;

/// A simple implementation of a "Dijkstra Map" as described in [Dijsktra Maps Visualized]
///
/// A [PathMap] is used to define the obstacles and movement costs for the map.
///
/// Various 'goals' can be defined with specific values. Once the map is
/// recalculated using a [PathMap] a path to the nearest goal can be
/// followed by querying for the next lowest value exit from any given position.
///
/// # Example
///
/// ```
/// use sark_pathfinding::*;
/// let mut pathmap = PathMap2d::new([20,20]);
/// pathmap.add_obstacle([5,5]);
/// pathmap.add_obstacle([5,6]);
/// pathmap.add_obstacle([5,7]);
///
/// // Ensure the dijsktra map is defined with  the same size as your pathmap.
/// let mut goals = DijkstraMap::new([20,20]);
/// goals.add_goal([10,10], 0.0);
/// goals.add_goal([15,15], 5.0);
/// goals.recalculate(&pathmap);
/// let next_step = goals.next_lowest([13,13], &pathmap).unwrap();
/// // Lower value goals are considered 'closer'.
/// assert_eq!([12,12], next_step.to_array());
/// ```
/// [Dijsktra Maps Visualized]: https://www.roguebasin.com/index.php/Dijkstra_Maps_Visualized
#[derive(Debug, Default, Clone)]
pub struct DijkstraMap {
    values: FloatGrid,
    goals: HashSet<IVec2>,
    /// Obstacles populated during map recalculation. This is only used
    /// when iterating over map values on an already calculated map to skip
    /// 'obstacles'.
    obstacles: BitGrid,
    frontier: MinHeap,
    size: UVec2,
    initial_value: f32,
}

impl DijkstraMap {
    pub fn new(size: impl GridSize) -> Self {
        let mut values = FloatGrid::new(size.to_uvec2());
        values.set_all(INITIAL_VALUE);
        Self {
            values,
            goals: HashSet::new(),
            frontier: MinHeap::with_capacity(size.tile_count()),
            obstacles: BitGrid::new(size.to_uvec2()),
            size: size.to_uvec2(),
            initial_value: INITIAL_VALUE,
        }
    }

    pub fn with_initial_value(mut self, initial_value: f32) -> Self {
        self.initial_value = initial_value;
        self.values.set_all(initial_value);
        self
    }

    pub fn from_string(s: impl AsRef<str>) -> Option<Self> {
        let width = s.as_ref().lines().map(|l| l.len()).max()?;
        let height = s.as_ref().lines().filter(|l| !l.is_empty()).count();
        if width == 0 || height == 0 {
            return None;
        }
        let size = UVec2::new(width as u32, height as u32);
        let mut values = FloatGrid::new(size);
        values.set_all(INITIAL_VALUE);
        let mut goals = HashSet::new();
        let mut obstacles = BitGrid::new(size);
        for (y, line) in s
            .as_ref()
            .lines()
            .filter(|l| !l.is_empty())
            .rev()
            .enumerate()
        {
            for (x, c) in line.chars().enumerate() {
                match c {
                    '#' => {
                        obstacles.set([x, y], true);
                    }
                    _ => {
                        // Attempt to convert the char into a goal value
                        if let Some(v) = c.to_digit(10).map(|d| d as f32) {
                            values.set_value([x, y], v);
                            goals.insert(IVec2::from([x as i32, y as i32]));
                        }
                    }
                }
            }
        }

        Some(Self {
            values,
            goals,
            obstacles,
            frontier: MinHeap::with_capacity(size.tile_count()),
            size,
            initial_value: INITIAL_VALUE,
        })
    }

    /// Add a 'goal' to a map position. `value` determines the desirability of
    /// the goal during pathfinding, where lower value goals are seen as 'closer' than
    /// higher value goals.
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

    /// Set the goal value for a position.
    ///
    /// Lower value goals are considered 'closer' during pathfinding.
    pub fn set_goal(&mut self, xy: impl GridPoint, value: f32) {
        self.values[xy] = value;
        self.goals.insert(xy.to_ivec2());
    }

    /// Recalculate the map based on the given pathing.
    ///
    /// This will not clear any currently set map values and will overwrite previous
    /// values. To recalculate a clean map based only on the currently set goals,
    /// call [DijkstraMap::clear_values] to clear any previously set non-goal tiles.
    pub fn recalculate(&mut self, pathing: &impl PathMap) {
        self.obstacles.set_all(true);
        self.frontier.clear();

        for i in 0..self.size.tile_count() {
            let xy = self.transform_itl(i);
            if !self.goals.contains(&xy) && pathing.is_obstacle(xy) {
                continue;
            }
            self.frontier.push(xy, self.values[i] as i32);
        }

        while let Some(curr) = self.frontier.pop() {
            for next in pathing.exits(curr) {
                let new_cost = self.values.value(curr) + pathing.cost(curr, next) as f32;
                self.obstacles.set(next, false);
                if new_cost < self.values.value(next) {
                    self.values.set_value(next, new_cost);
                    self.frontier.push(next, new_cost as i32);
                }
            }
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

    /// Resets the value of all non-goal tiles.
    pub fn clear_values(&mut self) {
        for (p, v) in self.values.iter_grid_points().zip(self.values.values_mut()) {
            if !self.goals.contains(&p) {
                *v = self.initial_value;
            }
        }
    }

    /// Clears all goals and values from the map, resetting all tiles values to 0.
    pub fn clear_all(&mut self) {
        self.clear_values();
        self.obstacles.set_all(true);
        self.goals.clear();
    }

    /// Apply a mathematical operation to every value in the map.
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
    /// The [DijkstraMap] does not store pathing information so a [PathMap] must be provided.
    ///
    /// The exits are sorted by 'value', so the first exit should be considered
    /// the most 'valuable'. The actual values can be retrieved via [DijkstraMap::exit_values].
    pub fn exits(&self, xy: impl GridPoint, pathing: &impl PathMap) -> impl Iterator<Item = IVec2> {
        self.exit_values(xy, pathing).map(|pv| pv.0)
    }

    /// Retrieve an iterator over the valid exits from a given position on the [DijkstraMap].
    ///
    /// The [DijkstraMap] does not store pathing information so a [PathMap] must be provided.
    ///
    /// Returns an iterator of 2 element tuples, where each tuple contains a position
    /// and it's corresponding 'value' in the [DijkstraMap]. The exits
    /// are sorted by 'most valuable' first before being returned.
    pub fn exit_values(
        &self,
        xy: impl GridPoint,
        pathing: &impl PathMap,
    ) -> IntoIter<(IVec2, i32), EXIT_CAP> {
        let xy = xy.to_ivec2();
        let mut v = ArrayVec::new();
        for next in pathing.exits(xy) {
            let Some(i) = next.get_index(self.size()) else {
                continue;
            };

            v.push((next, self.values[i] as i32));
        }
        v.sort_unstable_by(|a, b| a.1.cmp(&b.1));

        v.into_iter()
    }

    /// Returns the lowest value exit from a position if there is one.
    ///
    /// The [DijkstraMap] does not store pathing information so a [PathMap] must be provided.
    pub fn next_lowest(&self, xy: impl GridPoint, pathing: &impl PathMap) -> Option<IVec2> {
        let xy = xy.to_ivec2();
        let mut v: ArrayVec<(IVec2, i32), EXIT_CAP> = ArrayVec::new();
        for next in pathing.exits(xy) {
            let Some(i) = next.get_index(self.size()) else {
                continue;
            };

            v.push((next, self.values[i] as i32));
        }
        v.sort_unstable_by(|a, b| a.1.cmp(&b.1));

        v.into_iter().next().map(|pv| pv.0)
    }

    /// Returns the highest value exit from a position if there is one.
    ///
    /// The [DijkstraMap] does not store pathing information so a [PathMap] must be provided.
    pub fn next_highest(&self, xy: impl GridPoint, pathing: &impl PathMap) -> Option<IVec2> {
        let xy = xy.to_ivec2();
        let mut v: ArrayVec<(IVec2, i32), EXIT_CAP> = ArrayVec::new();
        for next in pathing.exits(xy) {
            let Some(i) = next.get_index(self.size()) else {
                continue;
            };

            v.push((next, self.values[i] as i32));
        }
        v.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        v.into_iter().next().map(|pv| pv.0)
    }

    pub fn values(&self) -> &[f32] {
        self.values.values()
    }

    /// Iterate over all the values in the map, skipping any obstacles.
    pub fn iter_xy(&self) -> impl Iterator<Item = (IVec2, f32)> + '_ {
        self.values
            .iter_grid_points()
            .zip(self.values.values().iter().copied())
            .filter(|(p, _)| !self.obstacles.get(*p))
    }

    /// Iterate over all the values in the map, skipping any obstacles.
    pub fn iter_xy_mut(&mut self) -> impl Iterator<Item = (IVec2, &mut f32)> + '_ {
        self.values
            .iter_grid_points()
            .zip(self.values.values_mut().iter_mut())
            .filter(|(p, _)| !self.obstacles.get(*p))
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn print_grid_values(&self) {
        for y in (0..self.size.y).rev() {
            for x in 0..self.size.x {
                let p = IVec2::new(x as i32, y as i32);
                let v = self.values.value(p);
                if v.abs() >= 999.0 {
                    print!("  ###");
                } else {
                    print!("{:5.1}", v);
                }
            }

            println!();
        }
        println!();
    }
}

impl SizedGrid for DijkstraMap {
    fn size(&self) -> UVec2 {
        self.values.size()
    }
}

#[cfg(test)]
mod tests {
    use glam::UVec2;

    use super::DijkstraMap;
    use crate::PathMap2d;

    #[test]
    #[ignore]
    fn open() {
        let size = UVec2::splat(10);
        let mut map = DijkstraMap::new(size);
        let pathing = PathMap2d::new(size);
        map.add_goal([4, 4], 1.0);
        map.recalculate(&pathing);
        map.print_grid_values();
    }

    #[test]
    #[ignore]
    fn obstacles() {
        let size = UVec2::splat(7);
        let mut map = DijkstraMap::new(size);
        let mut pathing = PathMap2d::new(size);
        let goal = [3, 3];
        pathing.add_obstacle([3, 4]);
        pathing.add_obstacle([4, 3]);
        pathing.add_obstacle([3, 2]);
        map.add_goal(goal, 1.0);
        map.recalculate(&pathing);
        map.print_grid_values();
    }

    #[test]
    #[ignore]
    fn goals() {
        let size = UVec2::splat(9);
        let mut map = DijkstraMap::new(size);
        let pathing = PathMap2d::new(size);
        map.add_goal([2, 2], 1.0);
        map.add_goal([8, 8], 3.0);
        map.recalculate(&pathing);
        map.print_grid_values();
    }

    #[test]
    #[ignore]
    fn flip() {
        let size = UVec2::splat(9);
        let mut map = DijkstraMap::new(size);
        let pathing = PathMap2d::new(size);
        map.add_goal([2, 2], 5.0);
        map.add_goal([8, 8], -3.0);
        map.recalculate(&pathing);
        map.print_grid_values();
        map.apply_operation(|f| f * -1.2);
        map.print_grid_values();
        map.recalculate(&pathing);
        map.print_grid_values();
    }

    #[test]
    #[ignore]
    fn string_map() {
        let map_string = "
###############
#             #
#             ###########
#                       #
#      2      #######   #
#             #     #   #
#             #     #####
###############
        ";
        let pathing = PathMap2d::from_string(map_string, '#').unwrap();
        let mut map = DijkstraMap::from_string(map_string).unwrap();
        map.recalculate(&pathing);
        map.apply_operation(|f| f * -1.2);
        map.recalculate(&pathing);
        map.print_grid_values();
    }
}
