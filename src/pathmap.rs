use arrayvec::{ArrayVec, IntoIter};
use glam::IVec2;
use sark_grids::{bit_grid::BitGrid, GridPoint, GridSize, SizedGrid};

/// A trait for a map that defines pathing information across a 2d grid.
pub trait PathMap {
    type ExitIterator: Iterator<Item = IVec2>;
    /// Returns an iterator of the valid exits from the given grid point.
    fn exits(&self, p: impl GridPoint) -> Self::ExitIterator;
    /// The cost of moving between two grid points.
    fn cost(&self, a: impl GridPoint, b: impl GridPoint) -> i32;
    /// The distance between two grid points.
    fn distance(&self, a: impl GridPoint, b: impl GridPoint) -> i32;
}

/// A basic pathmap that tracks obstacles and allows movement in 8
/// adjacent grid directions.
///
/// # Example
/// ```rust
/// use sark_pathfinding::*;
///
/// let mut map = PathMap2d::new([50,50]);
/// let mut pf = Pathfinder::new();
///
/// map.add_obstacle([5,4]);
///
/// let path = pf.astar(&map, [4,4], [10,10]).unwrap();
/// ```
pub struct PathMap2d {
    obstacles: BitGrid,
}

impl SizedGrid for PathMap2d {
    fn size(&self) -> glam::UVec2 {
        self.obstacles.size()
    }
}

impl PathMap2d {
    /// Create a new PathMap with all values set to false (no obstacles).
    pub fn new(size: impl GridSize) -> Self {
        Self {
            obstacles: BitGrid::new(size),
        }
    }

    pub fn is_obstacle(&self, p: impl GridPoint) -> bool {
        self.obstacles.get(p)
    }

    pub fn add_obstacle(&mut self, p: impl GridPoint) {
        self.set_obstacle(p, true);
    }

    pub fn remove_obstacle(&mut self, p: impl GridPoint) {
        self.set_obstacle(p, false);
    }

    pub fn set_obstacle(&mut self, p: impl GridPoint, v: bool) {
        self.obstacles.set(p, v);
    }

    pub fn toggle_obstacle(&mut self, p: impl GridPoint) {
        self.obstacles.toggle(p);
    }

    /// A reference to the underlying bit grid that stores the [PathMap2d]'s
    /// obstacle data.
    pub fn obstacle_grid(&self) -> &BitGrid {
        &self.obstacles
    }

    /// A mutable reference to the underlying bit grid that stores the
    /// [PathMap2d]'s obstacle data.
    pub fn obstacle_grid_mut(&mut self) -> &mut BitGrid {
        &mut self.obstacles
    }
}

pub const CARDINAL_COST: i32 = 2;
pub const DIAGONAL_COST: i32 = 3;

impl PathMap for PathMap2d {
    type ExitIterator = IntoIter<IVec2, 8>;
    fn exits(&self, p: impl GridPoint) -> Self::ExitIterator {
        let mut points = ArrayVec::new();
        for adj in p.adj_8() {
            if !self.obstacles.in_bounds(adj) {
                continue;
            }

            if !self.obstacles.get(adj) {
                points.push(adj);
            }
        }
        points.into_iter()
    }

    fn cost(&self, a: impl GridPoint, b: impl GridPoint) -> i32 {
        if is_cardinal(a, b) {
            CARDINAL_COST
        } else {
            DIAGONAL_COST
        }
    }

    fn distance(&self, a: impl GridPoint, b: impl GridPoint) -> i32 {
        octile_heuristic(a, b)
    }
}

/// Whether or not the difference between two points is along a cardinal direction
/// (not diagonal).
#[inline]
pub fn is_cardinal(a: impl GridPoint, b: impl GridPoint) -> bool {
    a.to_ivec2().cmpeq(b.to_ivec2()).any()
}

/// A heuristic function for pathfinding on a 4-way grid - aka Manhattan Distance.
#[inline]
pub fn cardinal_heuristic(a: impl GridPoint, b: impl GridPoint) -> i32 {
    a.taxi_dist(b) as i32
}

/// A heuristic function for pathfinding on an 8 way grid.
///
/// From https://github.com/riscy/a_star_on_grids/blob/master/src/heuristics.cpp#L67
#[inline]
pub fn octile_heuristic(a: impl GridPoint, b: impl GridPoint) -> i32 {
    let tcmd = 2 * CARDINAL_COST - DIAGONAL_COST;
    let d = (a.to_ivec2() - b.to_ivec2()).abs();
    (tcmd * (d.x - d.y).abs() + DIAGONAL_COST * (d.x + d.y)) / 2
}
