[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/sark_pathfinding)](https://crates.io/crates/sark_pathfinding/)
[![docs](https://docs.rs/sark_pathfinding/badge.svg)](https://docs.rs/sark_pathfinding/)

A simple implementation of the [astar pathfinding algorithm](https://www.redblobgames.com/pathfinding/a-star/implementation.html) 
from red blob games as well as "Dijkstra Maps" as described in [Dijkstra Maps Visualized]https://www.roguebasin.com/index.php/Dijkstra_Maps_Visualized  

---

In order to use the astar pathfinder you must have a path map for it to navigate. You can define one by implementing the `PathMap` trait, or you can use the built-in `PathMap2d`.

# Example

```rust
use sark_pathfinding::*;

let mut pf = Pathfinder::new();
let mut map = PathMap2d::new([10, 10]);
map.add_obstacle([3,0]);
map.add_obstacle([3,1]);
let path = pf.astar(&map, [0, 0], [5, 0]).unwrap();
assert_eq!(6, path.len());
```

![](images/pathfind_demo.gif)
*From the "astar" example.*

---

A `DijkstraMap` can be generated using a `PathMap` as well. The pathmap defines the obstacles and movement costs, and the 'goals' are defined in the dijkstra map.

# Example

```rust
use sark_pathfinding::*;
let mut pathmap = PathMap2d::new([20,20]);
pathmap.add_obstacle([5,5]);
pathmap.add_obstacle([5,6]);
pathmap.add_obstacle([5,7]);

// Ensure the dijsktra map is defined with  the same size as your pathmap.
let mut goals = DijkstraMap::new([20,20]);
goals.add_goal([10,10], 0.0);
goals.add_goal([15,15], 5.0);
goals.recalculate(&pathmap);
let next_step = goals.next_lowest([13,13], &pathmap).unwrap();
// Lower value goals are considered 'closer'.
assert_eq!([12,12], next_step.to_array());
```
