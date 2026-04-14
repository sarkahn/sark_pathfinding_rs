//! A rectangular grid of bit values for representing simple state across a large grid.

use bit_vec::BitVec;
use glam::{IVec2, UVec2};

use crate::grid::SizedGrid;

/// A rectangular grid with it's underlying data defined as a [BitVec].
#[derive(Default, Clone)]
pub struct BitGrid {
    bits: BitVec,
    size: UVec2,
}

impl BitGrid {
    /// Create a new BitGrid with all bits set to false.
    pub fn new(size: impl Into<UVec2>) -> Self {
        let size = size.into();
        Self {
            bits: BitVec::from_elem(size.element_product() as usize, false),
            size,
        }
    }

    /// Set the initial value for all bits.
    pub fn with_value(mut self, value: bool) -> Self {
        self.set_all(value);
        self
    }

    /// Retrieve the value of a bit at the given 2d index.
    #[inline]
    pub fn get(&self, xy: impl Into<IVec2>) -> bool {
        let i = self.xy_to_index(xy);
        self.get_index(i)
    }

    /// Retrieve the value of the bit at the given index.
    #[inline]
    pub fn get_index(&self, i: usize) -> bool {
        self.bits.get(i).unwrap()
    }

    #[inline]
    pub fn set_true(&mut self, xy: impl Into<IVec2>) {
        self.set(xy, true);
    }

    #[inline]
    pub fn set_false(&mut self, xy: impl Into<IVec2>) {
        self.set(xy, false);
    }

    /// Set the bit at the given 2d index.
    #[inline]
    pub fn set(&mut self, xy: impl Into<IVec2>, value: bool) {
        let i = self.xy_to_index(xy);
        self.bits.set(i, value);
    }

    /// Set the bit at the given 1d index.
    #[inline]
    pub fn set_index(&mut self, i: usize, value: bool) {
        self.bits.set(i, value);
    }

    /// Set the bit at the given 2d index to true.
    #[inline]
    pub fn set_index_true(&mut self, i: usize) {
        self.set_index(i, true);
    }

    /// Set the bit at the given 2d index to false.
    #[inline]
    pub fn set_index_false(&mut self, i: usize) {
        self.set_index(i, false);
    }

    /// Toggle the value of the given bit.
    #[inline]
    pub fn toggle(&mut self, xy: impl Into<IVec2>) {
        let i = self.xy_to_index(xy);
        self.toggle_index(i)
    }

    /// Toggle the value of the bit at the given index.
    #[inline]
    pub fn toggle_index(&mut self, i: usize) {
        let v = self.bits.get(i).unwrap();
        self.bits.set(i, !v);
    }

    /// Set the value for all bits.
    pub fn set_all(&mut self, value: bool) {
        self.bits.fill(value);
    }

    /// A reference to the underlying bit data.
    pub fn bits(&self) -> &BitVec {
        &self.bits
    }

    /// A mutable reference to the underlying bit data.
    pub fn bits_mut(&mut self) -> &mut BitVec {
        &mut self.bits
    }

    /// Returns true if any bits in the grid are set.
    pub fn any(&self) -> bool {
        self.bits.any()
    }

    /// Returns true if none of the bits in the grid are set.
    pub fn none(&self) -> bool {
        self.bits.none()
    }

    /// Negate the bits in the grid.
    pub fn negate_all(&mut self) {
        self.bits.negate();
    }

    /// Unset all bits in the grid.
    pub fn clear(&mut self) {
        self.bits.fill(false);
    }

    pub fn iter_xy(&self) -> impl Iterator<Item = (IVec2, bool)> + '_ {
        let w = self.width() as i32;
        self.bits
            .iter()
            .enumerate()
            .map(move |(i, b)| (IVec2::new(i as i32 % w, i as i32 / w), b))
    }
}

impl SizedGrid for BitGrid {
    fn size(&self) -> UVec2 {
        self.size
    }
}

impl std::fmt::Debug for BitGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = format!("{:?}", self.bits);
        for y in (0..self.height()).rev() {
            let start = self.xy_to_index([0, y as i32]);
            let end = start + self.width();
            writeln!(f, "{}", &a[start..end])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::SizedGrid;

    use super::BitGrid;

    #[test]
    fn iter() {
        let mut grid = BitGrid::new([10, 5]);

        grid.set_true([0, 0]);
        grid.set_true([0, 1]);
        grid.set_true([9, 2]);
        grid.set_true([9, 4]);

        let points: Vec<_> = grid.iter_xy().collect();
        assert!(points[grid.xy_to_index([0, 0])].1);
        assert!(points[grid.xy_to_index([0, 1])].1);
        assert!(points[grid.xy_to_index([9, 2])].1);
        assert!(points[grid.xy_to_index([9, 4])].1);
    }
}
