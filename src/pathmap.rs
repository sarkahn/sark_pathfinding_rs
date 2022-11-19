use arrayvec::{ArrayVec, IntoIter};
use glam::IVec2;
use sark_grids::{Grid, GridPoint, Size2d};

pub trait PathMap {
    type ExitIterator: Iterator<Item = IVec2>;
    /// Returns an iterator of the valid exits from the given grid point.
    fn exits(&self, p: impl GridPoint) -> Self::ExitIterator;
    /// The cost of moving between two grid points.
    fn cost(&self, a: impl GridPoint, b: impl GridPoint) -> i32;
    /// The distance between two grid points.
    fn distance(&self, a: impl GridPoint, b: impl GridPoint) -> usize;
}

/// A pathmap represented as a 2d grid of [bool].
///
/// Note that a grid position is considered an obstacle if it is set to `true`.
///
/// # Example
/// ```rust
/// use sark_pathfinding::*;
///
/// let mut map = PathMap2d::new([50,50]);
/// let mut pf = Pathfinder::new();
///
/// // Set position [5,4] of the path map to be a pathfinding obstacle.
/// map[[5,4]] = true;
///
/// let path = pf.astar(&map, [4,4], [10,10]).unwrap();
/// ```
pub struct PathMap2d {
    grid: Grid<bool>,
}

impl PathMap2d {
    /// Create a new PathMap with all values set to false (no obstacles).
    pub fn new(size: impl Size2d) -> Self {
        Self {
            grid: Grid::default(size),
        }
    }

    pub fn is_obstacle(&self, p: impl GridPoint) -> bool {
        self[p]
    }

    pub fn set_obstacle(&mut self, p: impl GridPoint, v: bool) {
        self[p] = v;
    }
}

impl std::ops::DerefMut for PathMap2d {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.grid
    }
}

impl std::ops::Deref for PathMap2d {
    type Target = Grid<bool>;

    fn deref(&self) -> &Self::Target {
        &self.grid
    }
}

impl PathMap for PathMap2d {
    type ExitIterator = IntoIter<IVec2, 8>;
    fn exits(&self, p: impl GridPoint) -> Self::ExitIterator {
        let mut points = ArrayVec::new();
        for adj in p.adj_8() {
            if !self.in_bounds(adj) {
                continue;
            }

            if !self[adj] {
                points.push(adj);
            }
        }
        points.into_iter()
    }

    fn cost(&self, _: impl GridPoint, _: impl GridPoint) -> i32 {
        1
    }

    fn distance(&self, a: impl GridPoint, b: impl GridPoint) -> usize {
        a.taxi_dist(b)
    }
}
