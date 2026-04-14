use glam::{IVec2, UVec2};

/// A rectangular grid with a defined size.
pub trait SizedGrid {
    fn size(&self) -> UVec2;

    fn width(&self) -> usize {
        self.size().x as usize
    }

    fn height(&self) -> usize {
        self.size().y as usize
    }

    fn area(&self) -> usize {
        self.width() * self.height()
    }

    fn contains_point(&self, xy: impl Into<IVec2>) -> bool {
        let xy = xy.into();
        xy.cmpge(IVec2::ZERO).all() && xy.cmplt(self.size().as_ivec2()).all()
    }

    fn try_xy_to_index(&self, xy: impl Into<IVec2>) -> Option<usize> {
        let xy = xy.into();
        if !self.contains_point(xy) {
            return None;
        }
        Some(self.xy_to_index(xy))
    }

    fn xy_to_index(&self, xy: impl Into<IVec2>) -> usize {
        let xy = xy.into();
        xy.y as usize * self.width() + xy.x as usize
    }

    fn index_to_xy(&self, i: usize) -> IVec2 {
        let w = self.width() as i32;
        IVec2::new(i as i32 % w, i as i32 / w)
    }
}

pub trait Grid<T>: SizedGrid {
    fn get_from_index(&self, index: usize) -> Option<&T>;
    fn get_mut_from_index(&mut self, index: usize) -> Option<&mut T>;

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a;
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a;

    fn iter_xy<'a>(&'a self) -> impl Iterator<Item = (IVec2, &'a T)>
    where
        T: 'a,
    {
        let w = self.width() as i32;
        self.iter()
            .enumerate()
            .map(move |(i, v)| (IVec2::new(i as i32 % w, i as i32 / w), v))
    }

    fn iter_xy_mut<'a>(&'a mut self) -> impl Iterator<Item = (IVec2, &'a mut T)>
    where
        T: 'a,
    {
        let w = self.width() as i32;
        self.iter_mut()
            .enumerate()
            .map(move |(i, v)| (IVec2::new(i as i32 % w, i as i32 / w), v))
    }

    fn get(&self, xy: impl Into<IVec2>) -> Option<&T> {
        let xy = xy.into();

        if xy.cmplt(IVec2::ZERO).any() || xy.cmpge(self.size().as_ivec2()).any() {
            return None;
        }

        let i = self.xy_to_index(xy);
        self.get_from_index(i)
    }

    /// Returns false if the input position was out of bounds
    fn set(&mut self, xy: impl Into<IVec2>, value: T) -> bool {
        let Some(v) = self.get_mut(xy) else {
            return false;
        };
        *v = value;
        true
    }

    /// Returns false if the input position was out of bounds
    fn set_from_index(&mut self, index: usize, value: T) -> bool {
        let Some(v) = self.get_mut_from_index(index) else {
            return false;
        };
        *v = value;
        true
    }

    fn get_mut(&mut self, xy: impl Into<IVec2>) -> Option<&mut T> {
        let xy = xy.into();
        if xy.cmplt(IVec2::ZERO).any() || xy.cmpge(self.size().as_ivec2()).any() {
            return None;
        }
        let i = self.xy_to_index(xy);
        self.get_mut_from_index(i)
    }

    fn set_all(&mut self, value: T)
    where
        T: Copy,
    {
        self.iter_mut().for_each(|v| *v = value);
    }
}
