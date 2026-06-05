pub mod bit_grid;
pub mod dijkstra_map;
pub mod float_grid;
pub mod grid;
pub mod min_heap;
pub mod pathfinder;
pub mod pathmap;

pub use dijkstra_map::DijkstraMap;
use glam::IVec2;
pub use min_heap::MinHeap;
pub use pathfinder::Pathfinder;
pub use pathmap::{PathMap, PathMap2d};

pub const UP: IVec2 = IVec2::from_array([0, 1]);
pub const DOWN: IVec2 = IVec2::from_array([0, -1]);
pub const LEFT: IVec2 = IVec2::from_array([-1, 0]);
pub const RIGHT: IVec2 = IVec2::from_array([1, 0]);
pub const UP_LEFT: IVec2 = IVec2::from_array([-1, 1]);
pub const UP_RIGHT: IVec2 = IVec2::from_array([1, 1]);
pub const DOWN_LEFT: IVec2 = IVec2::from_array([-1, -1]);
pub const DOWN_RIGHT: IVec2 = IVec2::from_array([1, -1]);

pub const DEFAULT_CARDINAL_COST: i32 = 2;
pub const DEFAULT_DIAGONAL_COST: i32 = 3;

/// Array of four orthogonal grid directions.
pub const DIR_4: &[IVec2] = &[UP, DOWN, LEFT, RIGHT];

/// Array of eight adjacent grid directions.
pub const DIR_8: &[IVec2] = &[
    UP, DOWN, LEFT, RIGHT, UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT,
];

/// The [taxicab distance](https://en.wikipedia.org/wiki/Taxicab_geometry)
/// between two points on a four-way grid.
pub fn taxi_dist(a: impl Into<IVec2>, b: impl Into<IVec2>) -> usize {
    let d = (a.into() - b.into()).abs();
    (d.x + d.y) as usize
}

/// Whether or not the difference between two points is along a cardinal direction
/// (not diagonal).
#[inline]
pub fn is_cardinal(a: impl Into<IVec2>, b: impl Into<IVec2>) -> bool {
    a.into().cmpeq(b.into()).any()
}

/// A heuristic function for pathfinding on a 4-way grid - aka Taxicab distance.
#[inline]
pub fn cardinal_heuristic(a: impl Into<IVec2>, b: impl Into<IVec2>) -> i32 {
    taxi_dist(a, b) as i32
}

/// A heuristic function for pathfinding on an 8 way grid.
///
/// From https://github.com/riscy/a_star_on_grids/blob/master/src/heuristics.cpp#L67
#[inline]
pub fn octile_heuristic(
    a: impl Into<IVec2>,
    b: impl Into<IVec2>,
    cardinal_cost: i32,
    diagonal_cost: i32,
) -> i32 {
    let tcmd: i32 = 2 * cardinal_cost - diagonal_cost;
    let d = (a.into() - b.into()).abs();
    (tcmd * (d.x - d.y).abs() + diagonal_cost * (d.x + d.y)) / 2
}
