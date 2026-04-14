//! A rectangular grid of float values with utility functions for performing operations
//! across the grid.

use glam::{IVec2, UVec2};

use crate::grid::{Grid, SizedGrid};

/// A rectangular grid of floating point values.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FloatGrid {
    data: Vec<f32>,
    size: UVec2,
}

impl FloatGrid {
    pub fn new(size: impl Into<UVec2>) -> Self {
        let size = size.into();
        Self {
            data: vec![0.0; size.element_product() as usize],
            size,
        }
    }

    pub fn values(&self) -> &[f32] {
        &self.data
    }
    pub fn values_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Apply a mathematical operation on all values in the grid.
    pub fn apply_operation(&mut self, operation: impl Fn(f32) -> f32) {
        for v in self.data.iter_mut() {
            *v = operation(*v);
        }
    }

    /// Reset all values in the [FloatGrid] to 0.
    pub fn clear(&mut self) {
        self.data.fill(0.0);
    }
}

impl SizedGrid for FloatGrid {
    fn size(&self) -> UVec2 {
        self.size
    }
}

impl Grid<f32> for FloatGrid {
    fn get_from_index(&self, index: usize) -> Option<&f32> {
        self.data.get(index)
    }

    fn get_mut_from_index(&mut self, index: usize) -> Option<&mut f32> {
        self.data.get_mut(index)
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a f32>
    where
        f32: 'a,
    {
        self.data.iter()
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut f32>
    where
        f32: 'a,
    {
        self.data.iter_mut()
    }
}

impl<P: Into<IVec2>> std::ops::Index<P> for FloatGrid {
    type Output = f32;

    fn index(&self, xy: P) -> &Self::Output {
        let i = self.xy_to_index(xy);
        &self.data[i]
    }
}

impl<P: Into<IVec2>> std::ops::IndexMut<P> for FloatGrid {
    fn index_mut(&mut self, xy: P) -> &mut Self::Output {
        let i = self.xy_to_index(xy);
        &mut self.data[i]
    }
}
