#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use crate::*;
    use sark_pathfinding::*;
    use test::Bencher;

    #[bench]
    fn test_std_heap(b: &mut Bencher) {
        let size = [500, 500];
        let map = PathMap2d::new(size);

        let mut astar = Pathfinder::new();
        b.iter(|| {
            astar.astar(&map, [0, 0], [499, 499]);
        });
    }
}
