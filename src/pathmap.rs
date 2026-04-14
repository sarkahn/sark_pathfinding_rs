use arrayvec::{ArrayVec, IntoIter};
use glam::{IVec2, UVec2};

use crate::{bit_grid::BitGrid, grid::SizedGrid};

pub const DEFAULT_MAX_EXITS: usize = 8;
pub const DEFAULT_CARDINAL_COST: i32 = 2;
pub const DEFAULT_DIAGONAL_COST: i32 = 3;

/// A trait for a map that defines pathing information across a 2d grid.
pub trait PathMap {
    type ExitIterator: Iterator<Item = IVec2>;
    /// Returns an iterator of the valid exits from the given grid point.
    fn exits(&self, p: impl Into<IVec2>) -> Self::ExitIterator;
    /// The cost of moving between two grid points.
    fn cost(&self, a: impl Into<IVec2>, b: impl Into<IVec2>) -> i32;
    /// The distance between two grid points.
    fn distance(&self, a: impl Into<IVec2>, b: impl Into<IVec2>) -> i32;
    fn is_obstacle(&self, p: impl Into<IVec2>) -> bool;
}

/// A basic pathmap that tracks obstacles. When building the map you can specify
/// whether to allow 4-way or 8-way movement. The default is 8-way movement.
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
    pub adjacency: Adjacency,
    obstacles: BitGrid,
}

/// Defines how the grid handles movement between adjacent tiles.
pub enum Adjacency {
    /// Grid allows for 4-way movement.
    Cardinal,
    /// Grid allows for 8-way movement.
    Octile {
        cardinal_cost: i32,
        diagonal_cost: i32,
    },
}

impl Default for Adjacency {
    fn default() -> Self {
        Self::Octile {
            cardinal_cost: DEFAULT_CARDINAL_COST,
            diagonal_cost: DEFAULT_DIAGONAL_COST,
        }
    }
}

impl PathMap2d {
    /// Create a new PathMap with all values set to false (no obstacles).
    pub fn new(size: impl Into<UVec2>) -> Self {
        Self {
            obstacles: BitGrid::new(size),
            adjacency: Adjacency::default(),
        }
    }

    pub fn from_string(s: impl AsRef<str>, obstacle_char: char) -> Option<Self> {
        let width = s.as_ref().lines().map(|l| l.len()).max()?;
        let height = s.as_ref().lines().filter(|l| !l.is_empty()).count();
        if width == 0 || height == 0 {
            return None;
        }
        let size = UVec2::new(width as u32, height as u32);
        let (mut x, mut y) = (0, 0);
        let mut obstacles = BitGrid::new(size);
        for line in s.as_ref().lines().filter(|l| !l.is_empty()).rev() {
            for c in line.chars() {
                if c == obstacle_char {
                    obstacles.set([x, y], true);
                }
                x += 1;
            }
            y += 1;
            x = 0;
        }
        Some(Self {
            adjacency: Adjacency::default(),
            obstacles,
        })
    }

    pub fn is_obstacle(&self, p: impl Into<IVec2>) -> bool {
        self.obstacles.get(p)
    }

    pub fn add_obstacle(&mut self, p: impl Into<IVec2>) {
        self.set_obstacle(p, true);
    }

    pub fn remove_obstacle(&mut self, p: impl Into<IVec2>) {
        self.set_obstacle(p, false);
    }

    pub fn set_obstacle(&mut self, p: impl Into<IVec2>, v: bool) {
        self.obstacles.set(p, v);
    }

    pub fn toggle_obstacle(&mut self, p: impl Into<IVec2>) {
        self.obstacles.toggle(p);
    }

    /// Remove an obstacle from one position and add an obstacle to another.
    /// Note this will ignore the current state of either position.
    pub fn move_obstacle(&mut self, old_pos: impl Into<IVec2>, new_pos: impl Into<IVec2>) {
        self.obstacles.set(old_pos, false);
        self.obstacles.set(new_pos, true);
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

    pub fn print_grid(&self) {
        for y in (0..self.height()).rev() {
            for x in 0..self.width() {
                let p = IVec2::new(x as i32, y as i32);
                if self.obstacles.get(p) {
                    print!("#");
                } else {
                    print!(".");
                }
            }

            println!();
        }
        println!();
    }
}

impl SizedGrid for PathMap2d {
    fn size(&self) -> UVec2 {
        self.obstacles.size()
    }
}

pub const UP: IVec2 = IVec2::from_array([0, 1]);
pub const DOWN: IVec2 = IVec2::from_array([0, -1]);
pub const LEFT: IVec2 = IVec2::from_array([-1, 0]);
pub const RIGHT: IVec2 = IVec2::from_array([1, 0]);
pub const UP_LEFT: IVec2 = IVec2::from_array([-1, 1]);
pub const UP_RIGHT: IVec2 = IVec2::from_array([1, 1]);
pub const DOWN_LEFT: IVec2 = IVec2::from_array([-1, -1]);
pub const DOWN_RIGHT: IVec2 = IVec2::from_array([1, -1]);

/// Array of four orthogonal grid directions.
pub const DIR_4: &[IVec2] = &[UP, DOWN, LEFT, RIGHT];

/// Array of eight adjacent grid directions.
pub const DIR_8: &[IVec2] = &[
    UP, DOWN, LEFT, RIGHT, UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT,
];

impl PathMap for PathMap2d {
    type ExitIterator = IntoIter<IVec2, DEFAULT_MAX_EXITS>;
    fn exits(&self, p: impl Into<IVec2>) -> Self::ExitIterator {
        let p = p.into();
        let mut points = ArrayVec::new();
        let neighbours = match self.adjacency {
            Adjacency::Cardinal => DIR_4.iter().copied(),
            _ => DIR_8.iter().copied(),
        }
        .map(|adj| p + adj);
        for adj in neighbours {
            if !self.obstacles.contains_point(adj) {
                continue;
            }

            if !self.obstacles.get(adj) {
                points.push(adj);
            }
        }
        points.into_iter()
    }

    fn cost(&self, a: impl Into<IVec2>, b: impl Into<IVec2>) -> i32 {
        match self.adjacency {
            Adjacency::Cardinal => 1,
            Adjacency::Octile {
                cardinal_cost,
                diagonal_cost,
            } => {
                if is_cardinal(a, b) {
                    cardinal_cost
                } else {
                    diagonal_cost
                }
            }
        }
    }

    fn distance(&self, a: impl Into<IVec2>, b: impl Into<IVec2>) -> i32 {
        match self.adjacency {
            Adjacency::Cardinal => cardinal_heuristic(a, b),
            Adjacency::Octile {
                cardinal_cost,
                diagonal_cost,
            } => octile_heuristic(a, b, cardinal_cost, diagonal_cost),
        }
    }

    fn is_obstacle(&self, p: impl Into<IVec2>) -> bool {
        self.obstacles.get(p)
    }
}

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
    let tcmd = 2 * cardinal_cost - diagonal_cost;
    let d = (a.into() - b.into()).abs();
    (tcmd * (d.x - d.y).abs() + diagonal_cost * (d.x + d.y)) / 2
}
