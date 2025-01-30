use glam::IVec2;
use ordered_float::OrderedFloat;
use sark_grids::GridPoint;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A MinHeap for pathfinding that can store positions with a cost, either floats
/// or ints.
///
/// # Example
/// ```rust
/// use sark_pathfinding::*;
/// let mut heap = MinHeap::new();
/// heap.push([10,10], 5.0);
///
/// let mut heap = MinHeap::new();
/// heap.push([10,10], 5);
/// ```
#[derive(Debug, Default, Clone)]
pub struct MinHeap<T: Cost = i32> {
    heap: BinaryHeap<Cell<T::Wrapped>>,
}

impl<T: Cost> MinHeap<T> {
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

    pub fn push(&mut self, xy: impl GridPoint, cost: T) {
        self.heap.push(Cell {
            cost: cost.wrap(),
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
struct Cell<T> {
    cost: T,
    pos: IVec2,
}

impl<T: Ord> Ord for Cell<T> {
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

impl<T: Ord> PartialOrd for Cell<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Wrapper to allow the usage of floats or ints as costs in the MinHeap.
pub trait Cost: Sized {
    type Wrapped: Ord;
    fn wrap(self) -> Self::Wrapped;
}

// Implement for i32
impl Cost for i32 {
    type Wrapped = i32;

    fn wrap(self) -> Self::Wrapped {
        self
    }
}

// Implement for f32, using OrderedFloat
impl Cost for f32 {
    type Wrapped = OrderedFloat<f32>;

    fn wrap(self) -> Self::Wrapped {
        OrderedFloat(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floatheap() {
        let mut floatheap = MinHeap::new();
        floatheap.push([2, 2], 2.0);
        floatheap.push([-10, -10], -10.0);
        floatheap.push([1, 1], 1.0);
        floatheap.push([5, 5], 5.0);

        assert_eq!([-10, -10], floatheap.pop().unwrap().to_array());
        assert_eq!([1, 1], floatheap.pop().unwrap().to_array());
        assert_eq!([2, 2], floatheap.pop().unwrap().to_array());
        assert_eq!([5, 5], floatheap.pop().unwrap().to_array());
    }

    #[test]
    fn intheap() {
        let mut floatheap = MinHeap::new();
        floatheap.push([2, 2], 2);
        floatheap.push([-10, -10], -10);
        floatheap.push([1, 1], 1);
        floatheap.push([5, 5], 5);

        assert_eq!([-10, -10], floatheap.pop().unwrap().to_array());
        assert_eq!([1, 1], floatheap.pop().unwrap().to_array());
        assert_eq!([2, 2], floatheap.pop().unwrap().to_array());
        assert_eq!([5, 5], floatheap.pop().unwrap().to_array());
    }
}
