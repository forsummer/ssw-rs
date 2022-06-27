use std::ops::Index;
use std::ops::IndexMut;

pub struct Mat<T>
where
    T: Copy + Clone + Sized + Default
{
    mat: Vec<Vec<T>>,
    pub shape: (usize, usize),
}

impl<T> Mat<T>
where
    T: Copy + Clone + Sized + Default
{
    pub fn init(shape: (usize, usize)) -> Mat<T>
    {
        let mat = vec![vec![T::default(); shape.1]; shape.0];
        Mat { mat, shape }
    }
}

impl<T> Index<usize> for Mat<T>
where
    T: Copy + Clone + Sized + Default
{
    type Output = Vec<T>;
    fn index(&self, index: usize) -> &Self::Output
    {
        &self.mat[index]
    }
}

impl<T> IndexMut<usize> for Mat<T>
where
    T: Copy + Clone + Sized + Default
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output
    {
        &mut self.mat[index]
    }
}

#[macro_export]
macro_rules! max
{
    ($x:expr) => ( $x );
    ($x:expr, $($xs:expr),+) =>
    {
        std::cmp::max($x, max!( $($xs),+ ))
    };
}