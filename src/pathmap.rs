use arrayvec::{ArrayVec, IntoIter};
use glam::{IVec2, UVec2};
use sark_grids::{bit_grid::BitGrid, GridPoint, GridSize, SizedGrid};

pub const DEFAULT_MAX_EXITS: usize = 8;
pub const DEFAULT_CARDINAL_COST: i32 = 2;
pub const DEFAULT_DIAGONAL_COST: i32 = 3;

/// A trait for a map that defines pathing information across a 2d grid.
pub trait PathMap: SizedGrid {
    type ExitIterator: Iterator<Item = IVec2>;
    /// Returns an iterator of the valid exits from the given grid point.
    fn exits(&self, p: impl GridPoint) -> Self::ExitIterator;
    /// The cost of moving between two grid points.
    fn cost(&self, a: impl GridPoint, b: impl GridPoint) -> i32;
    /// The distance between two grid points.
    fn distance(&self, a: impl GridPoint, b: impl GridPoint) -> i32;
    fn is_obstacle(&self, p: impl GridPoint) -> bool;
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

    /// Remove an obstacle from one position and add an obstacle to another.
    /// Note this will ignore the current state of either position.
    pub fn move_obstacle(&mut self, old_pos: impl GridPoint, new_pos: impl GridPoint) {
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

impl PathMap for PathMap2d {
    type ExitIterator = IntoIter<IVec2, DEFAULT_MAX_EXITS>;
    fn exits(&self, p: impl GridPoint) -> Self::ExitIterator {
        let mut points = ArrayVec::new();
        let neighbours = match self.adjacency {
            Adjacency::Cardinal => p.adj_4(),
            _ => p.adj_8(),
        };
        for adj in neighbours {
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

    fn distance(&self, a: impl GridPoint, b: impl GridPoint) -> i32 {
        match self.adjacency {
            Adjacency::Cardinal => cardinal_heuristic(a, b),
            Adjacency::Octile {
                cardinal_cost,
                diagonal_cost,
            } => octile_heuristic(a, b, cardinal_cost, diagonal_cost),
        }
    }

    fn is_obstacle(&self, p: impl GridPoint) -> bool {
        self.obstacles.get(p)
    }
}

/// Whether or not the difference between two points is along a cardinal direction
/// (not diagonal).
#[inline]
pub fn is_cardinal(a: impl GridPoint, b: impl GridPoint) -> bool {
    a.to_ivec2().cmpeq(b.to_ivec2()).any()
}

/// A heuristic function for pathfinding on a 4-way grid - aka Taxicab distance.
#[inline]
pub fn cardinal_heuristic(a: impl GridPoint, b: impl GridPoint) -> i32 {
    a.taxi_dist(b) as i32
}

/// A heuristic function for pathfinding on an 8 way grid.
///
/// From https://github.com/riscy/a_star_on_grids/blob/master/src/heuristics.cpp#L67
#[inline]
pub fn octile_heuristic(
    a: impl GridPoint,
    b: impl GridPoint,
    cardinal_cost: i32,
    diagonal_cost: i32,
) -> i32 {
    let tcmd = 2 * cardinal_cost - diagonal_cost;
    let d = (a.to_ivec2() - b.to_ivec2()).abs();
    (tcmd * (d.x - d.y).abs() + diagonal_cost * (d.x + d.y)) / 2
}
