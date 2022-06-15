use std::mem::transmute;
use std::mem::size_of;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Add;
use std::ops::Sub;
use std::ops::Shl;
use std::ops::Index;
use std::arch::x86_64::__m256i;
use std::arch::x86_64::_mm256_add_epi32;
use std::arch::x86_64::_mm256_sub_epi32;
use std::arch::x86_64::_mm256_set_epi32;
use std::arch::x86_64::_mm256_setr_epi32;
use std::arch::x86_64::_mm256_set1_epi32;
use std::arch::x86_64::_mm256_cmpeq_epi32;
use std::arch::x86_64::_mm256_cmpgt_epi32;
use std::arch::x86_64::_mm256_movemask_epi8;
use std::arch::x86_64::_mm256_setzero_si256;

#[derive(Clone, Copy)]
pub struct M256Epi32(pub __m256i);

impl PartialEq for M256Epi32
{
    fn eq(&self, other: &Self) -> bool
    {
        unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi32(self.0, other.0)) == -1 }
    }

    fn ne(&self, other: &Self) -> bool
    {
        !unsafe { _mm256_movemask_epi8(_mm256_cmpeq_epi32(self.0, other.0)) == -1 }
    }
}

impl PartialOrd for M256Epi32
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
    {
        if unsafe {_mm256_movemask_epi8(_mm256_cmpeq_epi32(self.0, other.0)) == -1}
        {
            Some(std::cmp::Ordering::Equal)
        }
        else if unsafe {_mm256_movemask_epi8(_mm256_cmpgt_epi32(self.0, other.0)) == -1}
        {
            Some(std::cmp::Ordering::Greater)
        }
        else if unsafe {_mm256_movemask_epi8(_mm256_cmpgt_epi32(other.0, self.0)) == -1}
        {
            Some(std::cmp::Ordering::Less)
        }
        else
        {
            None
        }
    }
}

impl Add for M256Epi32
{
    type Output = M256Epi32;
    fn add(self, rhs: Self) -> Self::Output
    {
        unsafe { M256Epi32(_mm256_add_epi32(self.0, rhs.0)) }
    }
}

impl Sub for M256Epi32
{
    type Output = M256Epi32;
    fn sub(self, rhs: Self) -> Self::Output
    {
        unsafe { M256Epi32(_mm256_sub_epi32(self.0, rhs.0)) }
    }
}

impl Shl<usize> for M256Epi32
{
    type Output = M256Epi32;
    fn shl(self, rhs: usize) -> Self::Output
    {
        if rhs > 7
        {
            panic!("Out of bound");
        }
        unsafe
        {
            let out = transmute::<*const __m256i, *mut i32>(&_mm256_setzero_si256() as *const __m256i);
            let ptr = transmute::<*const __m256i, *const i32>(&self.0 as *const __m256i).add(rhs);
            ptr.copy_to(out, (8 - rhs) * size_of::<i32>());
            M256Epi32(*transmute::<*mut i32, *const __m256i>(out))
        }
    }
}

impl Index<usize> for M256Epi32
{
    type Output = i32;
    fn index(&self, index: usize) -> &Self::Output
    {
        if index > 7
        {
            panic!("Out of bound");
        }
        let M256Epi32(arr) = self;
        unsafe
        {
            let ptr_arr = transmute::<*const __m256i, *const __m256i>(arr as *const __m256i);
            &*transmute::<*const __m256i, *const i32>(ptr_arr).add(index)
        }
    }
}

impl Display for M256Epi32
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "[{}, {}, {}, {}, {}, {}, {}, {}]", 
        self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7])
    }
}

impl Debug for M256Epi32
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.debug_list()
            .entry(&self[0])
            .entry(&self[1])
            .entry(&self[2])
            .entry(&self[3])
            .entry(&self[4])
            .entry(&self[5])
            .entry(&self[6])
            .entry(&self[7])
            .finish()
    }
}

#[allow(dead_code)]
impl M256Epi32
{
    pub fn zero() -> M256Epi32
    {
        unsafe { M256Epi32(_mm256_setzero_si256()) }
    }

    pub fn set(e0: i32, e1: i32, e2: i32, e3: i32, e4: i32, e5: i32, e6: i32, e7: i32) -> M256Epi32
    {
        unsafe { M256Epi32(_mm256_set_epi32(e0, e1, e2, e3, e4, e5, e6, e7)) }
    }

    pub fn setr(e0: i32, e1: i32, e2: i32, e3: i32, e4: i32, e5: i32, e6: i32, e7: i32) -> M256Epi32
    {
        unsafe { M256Epi32(_mm256_setr_epi32(e0, e1, e2, e3, e4, e5, e6, e7)) }
    }

    pub fn fill(item: i32) -> M256Epi32
    {
        unsafe { M256Epi32(_mm256_set1_epi32(item)) }
    }

    pub fn from_arr(arr: &[i32; 8]) -> M256Epi32
    {
        unsafe { M256Epi32(*transmute::<*const i32, *const __m256i>(arr.as_ptr())) }
    }

    pub fn to_arr(&self) -> [i32; 8]
    {
        unsafe
        {
            let mut arr = [0; 8];
            let ptr = transmute::<*const __m256i, *const i32>(&(self.0) as *const __m256i);
            ptr.copy_to(arr.as_mut_ptr(), 8);
            arr
        }
    }

    pub fn get_max_m256_i32(&self) -> i32
    {
        let mut arr = self.to_arr();
        arr.sort();
        *(arr.last().unwrap())
    }
}

#[macro_export]
macro_rules! max_epi32
{
    ( $ ( $arr: expr ), * ) => 
    { 
        {
            let mut max = M256Epi32::zero();
            use std::arch::x86_64::_mm256_max_epi32;
            let max_arr = |a: M256Epi32, b: M256Epi32| 
            {
                let (M256Epi32(x), M256Epi32(y)) = (a, b);
                unsafe { _mm256_max_epi32(x, y) }
            };
            $( max = M256Epi32(max_arr(max, $arr)); )*
            max
        }
    };
}

#[cfg(test)]
mod test
{
    use super::*;
    #[test]
    fn add()
    {
        let a = M256Epi32::set(1, 2, 3, -9, 0, 8, 4, -19);
        let b = M256Epi32::set(1, 23, 8, -92, 0, 1, 1, -9);
        let res = M256Epi32::set(2, 25, 11, -101, 0, 9, 5, -28);
        assert_eq!(a+b, res);
    }

    #[test]
    fn sub()
    {
        let a = M256Epi32::set(1, 2, 3, -9, 0, 8, 4, -19);
        let b = M256Epi32::set(1, 23, 8, -92, 0, 1, 1, -9);
        let res = M256Epi32::set(0, -21, -5, 83, 0, 7, 3, -10);
        assert_eq!(a-b, res);
    }

    #[test]
    fn shl()
    {
        let a = M256Epi32::set(1, 2, 3, 4, 5, 6, 7, 8);   
        let res = M256Epi32::set(0, 1, 2, 3, 4, 5, 6, 7);
        assert_eq!(a << 1, res);
    }

    #[test]
    fn index()
    {
        let a = M256Epi32::set(1, 2, 3, 4, 5, 6, 7, 8);
        assert_eq!(a[0], 8);
        assert_eq!(a[1], 7);
        assert_eq!(a[2], 6);
        assert_eq!(a[3], 5);
        assert_eq!(a[4], 4);
        assert_eq!(a[5], 3);
        assert_eq!(a[6], 2);
        assert_eq!(a[7], 1);
    }

    #[test]
    fn to_arr()
    {
        let a = M256Epi32::set(1, 2, 3, 4, 5, 6, 7, 8);
        assert_eq!(a.to_arr(), [8, 7, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn get_max()
    {
        let a = M256Epi32::set(9899, 27, 53, -9, 5, 6, 17, 8123);
        assert_eq!(a.get_max_m256_i32(), 9899, "max?");
    }
}
