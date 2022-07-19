use glam::IVec2;

pub use arrayvec::{ArrayVec, IntoIter};

use sark_grids::{directions::DIR_8, Grid, GridPoint};

/// Trait for defining how the pathfinding algorithm navigates your map.
///
/// `Neighbours` is the iterator returned by the `get_available_exits` function, 
/// which the pathfinder uses to find neighbours for a given cell.
pub trait PathingMap {
    type Neighbours: Iterator<Item = IVec2>;
    /// Returns the list of valid exits from a given cell.
    fn get_available_exits(&self, p: IVec2) -> Self::Neighbours;
    /// The cost of moving between two adjacent points.
    fn get_cost(&self, a: IVec2, b: IVec2) -> usize;
    /// The distance between two points.
    fn get_distance(&self, a: IVec2, b: IVec2) -> usize;
}

impl PathingMap for Grid<bool> {
    type Neighbours = IntoIter<IVec2, 8>;

    fn get_available_exits(&self, xy: IVec2) -> Self::Neighbours {
        let mut v = ArrayVec::<_, 8>::new();

        for dir in DIR_8.iter().cloned() {
            let next = xy + dir;

            if !self.in_bounds(next) {
                continue;
            }

            if !self[next] {
                v.push(next);
            }
        }
        v.into_iter()
    }

    fn get_cost(&self, _a: IVec2, _b: IVec2) -> usize {
        1
    }

    fn get_distance(&self, a: IVec2, b: IVec2) -> usize {
        a.taxi_dist(b)
    }
}
