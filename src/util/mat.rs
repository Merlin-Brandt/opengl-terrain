
use super::NonZero;

/// Two-dimensional Vec with one variable and one fixed dimension
pub struct Mat<T, F: FixedDimension> {
    pub vec: Vec<T>,
    pub fixed_dim: F,
}

impl<T, F> Mat<T, F> where F: FixedDimension {
    pub fn get(&self, coords: [usize; 2]) -> Option<&T> {
        self.fixed_dim.to_index(coords)
            .and_then(|ind| self.vec.get(ind))
    }
}

#[derive(Copy, Clone)]
pub struct FixedWidth(NonZero<usize>);
#[derive(Copy, Clone)]
pub struct FixedHeight(NonZero<usize>);

pub trait FixedDimension {
    type CoordsIter;

    fn to_index(&self, coord: [usize; 2]) -> Option<usize>;
    fn to_coords(&self, i: usize) -> [usize; 2];
    /// Returns an infinite iterator over all valid coords for this fixed dimension
    fn coords_iter(&self) -> Self::CoordsIter;
}

impl FixedWidth {
    pub fn from_width(width: usize) -> Option<FixedWidth> {
        NonZero::new(width)
            .map(|nz_width| FixedWidth(nz_width))
    }

    pub fn from_non_zero(width: NonZero<usize>) -> FixedWidth {
        FixedWidth(width)
    }

    pub fn width(&self) -> usize {
        self.0.unwrap()
    }
}

impl FixedHeight {
    pub fn from_height(height: usize) -> Option<FixedHeight> {
        NonZero::new(height)
            .map(|nz_height| FixedHeight(nz_height))
    }

    pub fn from_non_zero(height: NonZero<usize>) -> FixedHeight {
        FixedHeight(height)
    }

    pub fn height(&self) -> usize {
        self.0.unwrap()
    }
}

impl FixedDimension for FixedWidth {
    type CoordsIter = CoordsIter<FixedWidth>;

    fn to_index(&self, coord: [usize; 2]) -> Option<usize> {
        let w = self.width();
        if coord[0] >= w {
            None
        } else {
            Some(coord[1] * w + coord[0])
        }
    }

    fn to_coords(&self, i: usize) -> [usize; 2] {
        let w = self.width();
        [i % w, i / w]
    }

    fn coords_iter(&self) -> CoordsIter<FixedWidth> {
        CoordsIter {
            fixed_dim: *self,
            coords: [0, 0],
        }
    }
}

impl FixedDimension for FixedHeight {
    type CoordsIter = CoordsIter<FixedHeight>;

    fn to_index(&self, coord: [usize; 2]) -> Option<usize> {
        let h = self.height();
        if coord[0] >= h {
            None
        } else {
            Some(coord[0] * h + coord[1])
        }
    }

    fn to_coords(&self, i: usize) -> [usize; 2] {
        let h = self.height();
        [i / h, i % h]
    }

    fn coords_iter(&self) -> CoordsIter<FixedHeight> {
        CoordsIter {
            fixed_dim: *self,
            coords: [0, 0],
        }
    }
}

pub struct CoordsIter<FD> {
    fixed_dim: FD,
    coords: [usize; 2],
}

impl Iterator for CoordsIter<FixedWidth> {
    type Item = [usize; 2];

    fn next(&mut self) -> Option<[usize; 2]> {
        let ret = self.coords;

        self.coords[0]+= 1;

        if self.coords[0] == self.fixed_dim.width() {
            self.coords[0] = 0;
            self.coords[1]+= 1;
        }

        Some(ret)
    }
}

impl Iterator for CoordsIter<FixedHeight> {
    type Item = [usize; 2];

    fn next(&mut self) -> Option<[usize; 2]> {
        let ret = self.coords;

        self.coords[1]+= 1;

        if self.coords[1] == self.fixed_dim.height() {
            self.coords[1] = 0;
            self.coords[0]+= 1;
        }

        Some(ret)
    }
}
