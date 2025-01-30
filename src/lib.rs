//! A simple implementation of the [A* pathfinding algorithm](https://www.redblobgames.com/pathfinding/a-star/introduction.html#astar)
//! from Red Blob Games, along with a few other pathfinding utilities.
//!
//! In order to use the pathfinder you must have a [PathMap] which defines the
//! rules for movement across a grid. You can define one by implementing the
//! [PathMap] trait, or you can use the built-in [PathMap2d].
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
//! map.add_obstacle([5,4]);
//!
//! let path = pf.astar(&map, [4,4], [10,10]).unwrap();
//! ```
pub mod dijkstra_map;
pub mod min_heap;
pub mod pathfinder;
pub mod pathmap;

pub use dijkstra_map::DijkstraMap;
pub use pathfinder::Pathfinder;
pub use pathmap::{PathMap, PathMap2d};
pub use sark_grids::{GridPoint, SizedGrid};
