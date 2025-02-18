//! A minheap that stores positions with a cost.

use glam::IVec2;
use sark_grids::GridPoint;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A min heap for pathfinding that can store positions with a cost.
///
/// # Example
/// ```rust
/// use sark_pathfinding::*;
/// let mut heap = MinHeap::new();
/// heap.push([10,10], 5);
/// heap.push([1,1], 2);
/// heap.push([15,15], -1);
/// heap.push([33,33], 6);;
/// heap.push([7,7], 1);
/// assert_eq!(heap.pop().unwrap().to_array(), [15,15]);
/// ```
#[derive(Debug, Default, Clone)]
pub struct MinHeap {
    heap: BinaryHeap<Cell>,
}

impl MinHeap {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.heap.clear();
    }

    pub fn push(&mut self, xy: impl GridPoint, cost: i32) {
        self.heap.push(Cell {
            cost,
            pos: xy.to_ivec2(),
        });
    }

    pub fn pop(&mut self) -> Option<IVec2> {
        self.heap.pop().map(|c| c.pos)
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

/// A cell for our min heap.
#[derive(Eq, PartialEq, Debug, Default, Clone, Copy)]
struct Cell {
    cost: i32,
    pos: IVec2,
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        // order by cost, then y, then x
        other.cost.cmp(&self.cost).then_with(|| {
            self.pos
                .y
                .cmp(&other.pos.y)
                .then_with(|| self.pos.x.cmp(&other.pos.x))
        })
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap() {
        let mut heap = MinHeap::new();
        heap.push([2, 2], 2);
        heap.push([-10, -10], -10);
        heap.push([1, 1], 1);
        heap.push([5, 5], 5);

        assert_eq!([-10, -10], heap.pop().unwrap().to_array());
        assert_eq!([1, 1], heap.pop().unwrap().to_array());
        assert_eq!([2, 2], heap.pop().unwrap().to_array());
        assert_eq!([5, 5], heap.pop().unwrap().to_array());
    }
}
