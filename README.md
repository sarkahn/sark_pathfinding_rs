[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/sark_pathfinding)](https://crates.io/crates/sark_pathfinding/)
[![docs](https://docs.rs/sark_pathfinding/badge.svg)](https://docs.rs/sark_pathfinding/)

A simple implementation of the [astar pathfinding algorithm](https://www.redblobgames.com/pathfinding/a-star/implementation.html) 
from red blob games.

In order to use the pathfinder you must have a path map for it to navigate. You can
define one by implementing the `PathingMap` trait, or you can use the built-in
`PathMap2d`.

# Example

```rust
use sark_pathfinding::*;

let mut map = PathMap2d::new([50,50]);
let mut pf = Pathfinder::new();

// Set position [5,4] of the path map to be a pathfinding obstacle.
map[5,4] = true;

let path = pf.astar(&map, [4,4], [10,10]).unwrap();
```

![](images/pathfind_demo.gif)

*From the "terminal" example.*//! A simple implementation of the [astar pathfinding algorithm](https://www.redblobgames.com/pathfinding/a-star/implementation.html)
