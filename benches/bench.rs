use criterion::{criterion_group, criterion_main, Criterion};

use sark_pathfinding::{PathMap2d, *};

fn test_std_heap(c: &mut Criterion) {
    let size = [500, 500];
    let map = PathMap2d::default(size);

    c.bench_function("std_heap", |b| {
        b.iter(|| {
            let mut astar = AStar::new(10);
            astar.find_path(&map, [0, 0], [499, 499]);
        })
    });
    // b.iter(|| {
    //     astar.find_path(&map, [0, 0], [499, 499]);
    // });
}

criterion_group!(benches, test_std_heap);
criterion_main!(benches);
