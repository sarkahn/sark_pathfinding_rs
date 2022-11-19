//! A simple implementation of the [astar pathfinding algorithm](https://www.redblobgames.com/pathfinding/a-star/implementation.html)
//! from red blob games.
//!
//! In order to use the pathfinder you must have a path map for it to navigate.
//! You can define one by implementing the [PathMap] trait, or you can use the
//! built-in [PathMap2d].
//!
//! # Example
//!
//! ```rust
//! use sark_pathfinding::*;
//!
//! let mut map = PathMap2d::new([50,50]);
//! let mut pf = Pathfinder::new();
//!
//! // Set position [5,4] of the path map to be a pathfinding obstacle.
//! map[5,4] = true;
//!
//! let path = pf.astar(&map, [4,4], [10,10]).unwrap();
//! ```
pub mod pathfinder;
pub mod pathmap;

pub use pathfinder::Pathfinder;
pub use pathmap::{PathMap, PathMap2d};
pub use sark_grids::{GridPoint, Size2d};
