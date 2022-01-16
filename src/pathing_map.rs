use glam::{IVec2, UVec2};
use smallvec::SmallVec;

/// Trait for defining how the pathfinding algorithm reads your map.
pub trait PathingMap<T: Eq> {
    /// Returns the list of valid exits from a given cell.
    fn get_available_exits(&self, p: T) -> SmallVec<[T;8]>;
    /// The cost of moving between two adjacent points.
    fn get_cost(&self, a: T, b: T) -> usize;
    /// The distance between two points.
    fn get_distance(&self, a: T, b: T) -> usize;
}

pub const ADJACENT_4_WAY: [[i32;2];4] = [
    [ 0,-1],
    [ 1, 0],
    [ 0, 1],
    [-1, 0],
];

pub const ADJACENT_8_WAY: [[i32;2];8] = [
    [ 0,-1],
    [ 1, 0],
    [ 0, 1],
    [-1, 0],
    [-1,-1],
    [ 1,-1],
    [-1, 1],
    [ 1, 1],
];  

/// A simple 2d path map.
pub struct PathMap2d {
    tiles: Vec<bool>,
    size: UVec2,
}

impl PathingMap<[i32;2]> for PathMap2d {
    fn get_available_exits(&self, p: [i32;2]) -> smallvec::SmallVec<[[i32;2];8]> {
        let mut output = SmallVec::new();
        let xy = IVec2::from(p);

        for dir in ADJACENT_8_WAY {
            let dir = IVec2::from(dir);
            let next = xy + dir;

            if !self.in_bounds(next.into()) {
                continue;
            }

            if !self.is_obstacle([next.x, next.y]) {
                output.push(next.into());
            }
        }
        output
    }

    fn get_cost(&self, _a: [i32;2], _b: [i32;2]) -> usize {
        1
    }

    fn get_distance(&self, a: [i32;2], b: [i32;2]) -> usize {
        // Manhattan distance
        ((a[0] - b[0]).abs() + (a[1] - b[1]).abs()) as usize
    }
}

impl PathMap2d {
    pub fn new(size: [u32;2]) -> Self {
        Self {
            tiles: vec![false; (size[0] * size[1]) as usize],
            size: UVec2::from(size),
        }
    }

    pub fn to_index(&self, xy: [i32;2]) -> usize {
        xy[1] as usize * self.width() + xy[0] as usize
    }

    pub fn to_xy(&self, index: usize) -> IVec2 {
        let x = index % self.width();
        let y = index / self.width();
        IVec2::new(x as i32,y as i32)

    }
    pub fn in_bounds(&self, xy: [i32;2]) -> bool {
        let [x,y] = xy;
        x >= 0 && x < self.width() as i32 && y >= 0 && y < self.height() as i32
    }

    pub fn is_obstacle(&self, xy: [i32;2]) -> bool {
        self.tiles[self.to_index(xy)]
    }

    pub fn width(&self) -> usize {
        self.size.x as usize
    }

    pub fn height(&self) -> usize {
        self.size.y as usize
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn is_obstacle_index(&self, i: usize) -> bool {
        self.tiles[i]
    }

    pub fn toggle_obstacle_index(&mut self, i: usize) {
        self.tiles[i] = !self.tiles[i]
    }

    pub fn iter(&self) -> impl Iterator<Item=bool> + '_ {
        self.tiles.iter().cloned()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut bool> {
        self.tiles.iter_mut()
    }
}
