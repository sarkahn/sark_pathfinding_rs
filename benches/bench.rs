#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use crate::*;
    use sark_pathfinding::{pathing_map::PathMap2d, *};
    use test::Bencher;

    #[bench]
    fn test_std_heap(b: &mut Bencher) {
        let size = [500, 500];
        let map = PathMap2d::new(size);

        let mut astar = AStar::from_size(size);
        b.iter(|| {
            astar.find_path(&map, [0, 0], [499, 499]);
        });
    }
}
